use crate::errors::msgpack_error::MessagePackError;
use bytes::{Buf, Bytes, BytesMut};
use futures::Stream;
use futures_lite::StreamExt;
use log::info;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Clone, PartialEq)]
pub struct KeyedBytes {
    pub key: String,
    pub length: u64,
    pub data: Bytes,
}

#[derive(Debug, PartialEq, Clone)]
enum ParserState {
    Initial,
    KeyHeader,
    KeyData {
        length: usize,
        collected: usize,
    },
    ValueHeader {
        key: String,
    },
    ValueLength {
        key: String,
    },
    ValueData {
        key: String,
        length: usize,
        collected: usize,
    },
    Complete,
}

// Function to handle larger MessagePack maps (uint32 size)
pub fn parse_map_size(byte: u8, buffer: &mut BytesMut) -> Option<Result<usize, MessagePackError>> {
    match byte {
        // Small map (up to 15 items)
        b if (b & 0xF0) == 0x80 => Some(Ok((b & 0x0F) as usize)),
        // Map 16 format (0xDE)
        0xDE => {
            if buffer.len() < 2 {
                None // Need more bytes
            } else {
                let size = ((buffer[0] as usize) << 8) | (buffer[1] as usize);
                buffer.advance(2);
                Some(Ok(size))
            }
        }
        // Map 32 format (0xDF)
        0xDF => {
            if buffer.len() < 4 {
                None // Need more bytes
            } else {
                let size = ((buffer[0] as usize) << 24)
                    | ((buffer[1] as usize) << 16)
                    | ((buffer[2] as usize) << 8)
                    | (buffer[3] as usize);
                buffer.advance(4);
                Some(Ok(size))
            }
        }
        // Not a map
        _ => Some(Err(MessagePackError::UnexpectedFormat(format!(
            "Expected map marker, got: {:#x}",
            byte
        )))),
    }
}

// Enhanced implementation for larger MessagePack streams

pub struct MessagePackByteStream {
    state: ParserState,
    buffer: BytesMut,
    current_key_buffer: Vec<u8>,
    remaining_pairs: usize,
}

impl MessagePackByteStream {
    pub fn new() -> Self {
        Self {
            state: ParserState::Initial,
            buffer: BytesMut::new(),
            current_key_buffer: Vec::new(),
            remaining_pairs: 0,
        }
    }

    pub fn extend_buffer(&mut self, new_bytes: Bytes) {
        self.buffer.extend_from_slice(&new_bytes);
    }

    pub fn process_bytes(&mut self) -> Option<Result<KeyedBytes, MessagePackError>> {
        while !self.buffer.is_empty() {
            match self.state.clone() {
                ParserState::Initial => {
                    if self.buffer.is_empty() {
                        return None;
                    }

                    let header = self.buffer[0];
                    self.buffer.advance(1);

                    // Handle different map size formats
                    match parse_map_size(header, &mut self.buffer) {
                        Some(Ok(size)) => {
                            self.remaining_pairs = size;
                            if size == 0 {
                                self.state = ParserState::Complete;
                            } else {
                                self.state = ParserState::KeyHeader;
                            }
                        }
                        Some(Err(e)) => return Some(Err(e)),
                        None => return None, // Need more bytes
                    }
                }

                ParserState::KeyHeader => {
                    if self.buffer.is_empty() {
                        return None;
                    }

                    let header = self.buffer[0];
                    self.buffer.advance(1);

                    // Handle different string formats
                    if (header & 0xE0) == 0xA0 {
                        // fixstr format
                        let length = (header & 0x1F) as usize;
                        self.current_key_buffer.clear();
                        self.state = ParserState::KeyData {
                            length,
                            collected: 0,
                        };
                    } else if header == 0xD9 {
                        // str 8 format
                        if self.buffer.is_empty() {
                            return None;
                        }
                        let length = self.buffer[0] as usize;
                        self.buffer.advance(1);
                        self.current_key_buffer.clear();
                        self.state = ParserState::KeyData {
                            length,
                            collected: 0,
                        };
                    } else if header == 0xDA {
                        // str 16 format
                        if self.buffer.len() < 2 {
                            return None;
                        }
                        let length = ((self.buffer[0] as usize) << 8) | (self.buffer[1] as usize);
                        self.buffer.advance(2);
                        self.current_key_buffer.clear();
                        self.state = ParserState::KeyData {
                            length,
                            collected: 0,
                        };
                    } else if header == 0xDB {
                        // str 32 format
                        if self.buffer.len() < 4 {
                            return None;
                        }
                        let length = ((self.buffer[0] as usize) << 24)
                            | ((self.buffer[1] as usize) << 16)
                            | ((self.buffer[2] as usize) << 8)
                            | (self.buffer[3] as usize);
                        self.buffer.advance(4);
                        self.current_key_buffer.clear();
                        self.state = ParserState::KeyData {
                            length,
                            collected: 0,
                        };
                    } else {
                        return Some(Err(MessagePackError::UnexpectedFormat(
                            "Expected string marker for key".to_string(),
                        )));
                    }
                }

                ParserState::KeyData { length, collected } => {
                    let remaining = length - collected;
                    let bytes_available = self.buffer.len().min(remaining);

                    if bytes_available == 0 {
                        return None;
                    }

                    self.current_key_buffer
                        .extend_from_slice(&self.buffer[..bytes_available]);
                    self.buffer.advance(bytes_available);

                    let new_collected = collected + bytes_available;
                    if new_collected == length {
                        match String::from_utf8(self.current_key_buffer.clone()) {
                            Ok(key) => {
                                self.state = ParserState::ValueHeader { key };
                            }
                            Err(e) => {
                                return Some(Err(e.into()));
                            }
                        }
                    } else {
                        self.state = ParserState::KeyData {
                            length,
                            collected: new_collected,
                        };
                        return None;
                    }
                }

                ParserState::ValueHeader { key } => {
                    if self.buffer.is_empty() {
                        return None;
                    }

                    let header = self.buffer[0];
                    self.buffer.advance(1);
                    // Handle different binary formats
                    if header == 0xC4 {
                        // bin 8 format
                        self.state = ParserState::ValueLength { key: key.clone() };
                    } else if header == 0xC5 {
                        // bin 16 format
                        if self.buffer.len() < 2 {
                            return None;
                        }
                        let length = ((self.buffer[0] as usize) << 8) | (self.buffer[1] as usize);
                        self.buffer.advance(2);
                        self.state = ParserState::ValueData {
                            key: key.clone(),
                            length,
                            collected: 0,
                        };
                    } else if header == 0xC6 {
                        // bin 32 format
                        if self.buffer.len() < 4 {
                            return None;
                        }
                        let length = ((self.buffer[0] as usize) << 24)
                            | ((self.buffer[1] as usize) << 16)
                            | ((self.buffer[2] as usize) << 8)
                            | (self.buffer[3] as usize);
                        self.buffer.advance(4);
                        self.state = ParserState::ValueData {
                            key: key.clone(),
                            length,
                            collected: 0,
                        };
                    } else {
                        return Some(Err(MessagePackError::UnexpectedFormat(
                            "Expected binary marker for value".to_string(),
                        )));
                    }
                }

                ParserState::ValueLength { key } => {
                    if self.buffer.is_empty() {
                        return None;
                    }

                    let length = self.buffer[0] as usize;
                    self.buffer.advance(1);

                    self.state = ParserState::ValueData {
                        key: key.clone(),
                        length,
                        collected: 0,
                    };
                }

                ParserState::ValueData {
                    key,
                    length,
                    collected,
                } => {
                    let remaining = length - collected;
                    let bytes_available = self.buffer.len().min(remaining);

                    if bytes_available == 0 {
                        return None;
                    }

                    // Extract the bytes to emit without copying the entire buffer
                    let data = self.buffer.split_to(bytes_available).freeze();

                    let new_collected = collected + bytes_available;
                    if new_collected == length {
                        self.remaining_pairs -= 1;
                        if self.remaining_pairs == 0 {
                            self.state = ParserState::Complete;
                        } else {
                            self.state = ParserState::KeyHeader;
                        }
                    } else {
                        self.state = ParserState::ValueData {
                            key: key.clone(),
                            length,
                            collected: new_collected,
                        };
                    }

                    // Return the bytes with their key immediately
                    return Some(Ok(KeyedBytes {
                        key: key.clone(),
                        length: length as u64,
                        data,
                    }));
                }

                ParserState::Complete => {
                    return None;
                }
            }
        }

        None
    }
}

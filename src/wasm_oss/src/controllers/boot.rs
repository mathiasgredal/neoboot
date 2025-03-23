use crate::commands::CommandDispatcher;
use crate::executor::Executor;
use crate::ffi;
use crate::utils::{parse_int, sys_get_env};
use crate::{asyncio::get_keypress, errors::lwip_error::LwipError};
use bytes::Bytes;
use futures::lock::Mutex;
use futures::Stream;
use futures::{
    future::{select, Either},
    FutureExt,
};
use futures_lite::future::yield_now;
use log::info;
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;
use std::{cell::RefCell, future::Future, pin::Pin, rc::Rc};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PayloadType {
    Kernel,
    Devicetree,
    Ramdisk,
}

impl PayloadType {
    fn as_str(&self) -> &'static str {
        match self {
            PayloadType::Kernel => "kernel_addr_r",
            PayloadType::Devicetree => "fdt_addr_r",
            PayloadType::Ramdisk => "ramdisk_addr_r",
        }
    }
}

impl FromStr for PayloadType {
    type Err = LwipError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "kernel_addr_r" => PayloadType::Kernel,
            "fdt_addr_r" => PayloadType::Devicetree,
            "ramdisk_addr_r" => PayloadType::Ramdisk,
            _ => return Err(LwipError::IllegalArgument),
        })
    }
}

pub struct Payload {
    payload_type: PayloadType,
    address: u64,
    offset: u64,
    length: u64,
}

pub struct BootController {
    payloads: Vec<Payload>,
}

impl BootController {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self { payloads: vec![] }))
    }

    pub fn get_payload(&self, payload_type: &PayloadType) -> Option<&Payload> {
        self.payloads
            .iter()
            .find(|p| p.payload_type == *payload_type)
    }

    pub fn remove_payload(&mut self, payload_type: &PayloadType) {
        self.payloads.retain(|p| p.payload_type != *payload_type);
    }

    fn get_payload_mut(&mut self, payload_type: &PayloadType) -> Option<&mut Payload> {
        self.payloads
            .iter_mut()
            .find(|p| p.payload_type == *payload_type)
    }

    pub fn set_payload_address(&mut self, payload_type: PayloadType, address: u64) {
        if let Some(payload) = self.get_payload_mut(&payload_type) {
            payload.address = address;
        } else {
            self.payloads.push(Payload {
                payload_type,
                address,
                offset: 0,
                length: 0,
            });
        }
    }

    pub async fn put_payload_bytes(
        &mut self,
        payload_type: PayloadType,
        payload_size: u64,
        bytes: Bytes,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let payload = self.get_payload(&payload_type);

        // If the payload is not found, add it
        if payload.is_none() {
            let payload_address = sys_get_env(payload_type.as_str())?;
            let payload_address = parse_int(&payload_address)?;
            self.payloads.push(Payload {
                payload_type: payload_type.clone(),
                address: payload_address,
                offset: 0,
                length: payload_size,
            });
        }

        // TODO: Check size

        // Copy the bytes to the payload
        let payload = self.get_payload_mut(&payload_type).unwrap();
        let result = unsafe {
            ffi::env_memcpy(
                bytes.as_ptr(),
                payload.address + payload.offset,
                bytes.len() as u32,
            )
        };

        if result < 0 {
            return Err(Box::new(LwipError::Buffer));
        }

        // Increment the offset
        payload.offset += bytes.len() as u64;

        Ok(())
    }

    pub fn boot(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Booting...");
        if self.payloads.is_empty() {
            info!("No payloads set");
            return Ok(());
        }

        info!("Payloads: {:?}", self.payloads.len());

        let mut device_tree = match self.get_payload(&PayloadType::Devicetree) {
            Some(device_tree) => device_tree,
            None => {
                let fdt_address = sys_get_env("fdt_addr");
                if fdt_address.is_err() {
                    return Err("No devicetree payload set".into());
                }
                let fdt_address = parse_int(&fdt_address.unwrap());
                if fdt_address.is_err() {
                    return Err("No devicetree payload set".into());
                }
                self.set_payload_address(PayloadType::Devicetree, fdt_address.unwrap());
                self.get_payload(&PayloadType::Devicetree).unwrap()
            }
        };

        let kernel_address = self.get_payload(&PayloadType::Kernel);
        if kernel_address.is_none() {
            return Err("No kernel payload set".into());
        }

        let ramdisk_address = self.get_payload(&PayloadType::Ramdisk);
        if ramdisk_address.is_none() {
            return Err("No ramdisk payload set".into());
        }

        info!("Booting...");
        let cmd_str = format!(
            "booti {:x} {:x}:{:x} {:x}",
            kernel_address.unwrap().address,
            ramdisk_address.unwrap().address,
            ramdisk_address.unwrap().length,
            device_tree.address,
        );
        info!("Executing command: '{}'", cmd_str);
        unsafe { ffi::env_execute_cmd(cmd_str.as_ptr(), cmd_str.len() as u32) };
        Ok(())
    }
}

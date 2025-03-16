use futures::io::{AsyncRead, AsyncWrite};
use futures_rustls::client::TlsStream;
use hyper::rt::{self};
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

pub enum AnyHttpStream<T>
where
    T: AsyncRead + AsyncWrite,
{
    Http(T),
    Https(TlsStream<T>),
}

impl<T: AsyncRead + AsyncWrite> From<T> for AnyHttpStream<T> {
    fn from(inner: T) -> Self {
        Self::Http(inner)
    }
}

impl<T: AsyncRead + AsyncWrite> From<TlsStream<T>> for AnyHttpStream<T> {
    fn from(inner: TlsStream<T>) -> Self {
        Self::Https(inner)
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> rt::Read for AnyHttpStream<T> {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        mut buf: rt::ReadBufCursor<'_>,
    ) -> Poll<Result<(), io::Error>> {
        let mut buf_size = 512;
        if buf_size > buf.remaining() {
            buf_size = buf.remaining();
        }
        let mut ibuf = vec![0u8; buf_size];

        match Pin::get_mut(self) {
            Self::Http(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_read(cx, &mut ibuf).map_ok(|n| {
                    buf.put_slice(&ibuf[..n]);
                    ()
                })
            }
            Self::Https(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_read(cx, &mut ibuf).map_ok(|n| {
                    buf.put_slice(&ibuf[..n]);
                    ()
                })
            }
        }
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> rt::Write for AnyHttpStream<T> {
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match Pin::get_mut(self) {
            Self::Http(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_write(cx, buf)
            }
            Self::Https(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_write(cx, buf)
            }
        }
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match Pin::get_mut(self) {
            Self::Http(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_flush(cx)
            }
            Self::Https(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_flush(cx)
            }
        }
    }

    #[inline]
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match Pin::get_mut(self) {
            Self::Http(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_close(cx)
            }
            Self::Https(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_close(cx)
            }
        }
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        return false;
    }

    #[inline]
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        _: &[io::IoSlice<'_>],
    ) -> Poll<Result<usize, io::Error>> {
        todo!()
    }
}

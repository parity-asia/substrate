
use futures::prelude::*;
use log::trace;
use pin_project::pin_project;

use std::{
    error,
    fmt::{self, Write},
    io,
    io::Error as IoError,
    num::ParseIntError,
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};

/// A pre-shared key, consisting of 32 bytes of random data.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CA(Vec<u8>);

impl CA {
    /// Create a new pre shared key from raw bytes
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
}

/// Private network configuration
#[derive(Debug, Clone)]
pub struct CaExchangeConfig {
    /// the PreSharedKey to use for encryption
    anchor: CA,
    cert: CA,
}

impl CaExchangeConfig {
    pub fn new(anchor: CA, cert: CA) -> Self {
        Self { anchor: anchor, cert: cert }
    }

    /// upgrade a connection to use pre shared key encryption.
    ///
    /// the upgrade works by both sides exchanging 24 byte nonces and then encrypting
    /// subsequent traffic with XSalsa20
    pub async fn handshake<TSocket>(
        self,
        mut socket: TSocket,
    ) -> Result<CaExchangeOutput<TSocket>, CaExchangeError>
    where
        TSocket: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        trace!("exchanging nonces");
        let mut local_ca: CA;
        // let mut remote_der = vec!(); // 3072 max length of ca with der format
        let mut remote_ca = [0u8; 3];

        // let mut remote_vec = vec!();
        // rand::thread_rng().fill_bytes(&mut local_nonce);
        let ca_len = self.cert.0.len() as u16;
        let mut ca_len_arr = [0u8; 2];
        ca_len_arr[0] = (ca_len & 0xFF) as u8;
        ca_len_arr[1] = ((ca_len >> 8)  & 0xFF) as u8;
        socket
            .write_all(&ca_len_arr)
            .await
            .map_err(CaExchangeError::HandshakeError)?;
        socket
            .write_all(&self.cert.0[..])
            .await
            .map_err(CaExchangeError::HandshakeError)?;

        let mut remote_len_bytes = [0u8; 2];
        socket.read_exact(&mut remote_len_bytes)
            .await
            .map_err(CaExchangeError::HandshakeError)?;
        trace!("remote nonce is {:?}", remote_len_bytes);
        let mut remote_len: usize = 0;
        remote_len = (remote_len_bytes[0] as usize) + (((remote_len_bytes[1] as u16) << 8) as usize);

        while remote_len > 8 {
            let mut read_bytes = [0u8; 8];

            socket.read_exact(&mut read_bytes)
                .await
                .map_err(CaExchangeError::HandshakeError)?;
            trace!("remote nonce is {:?}", read_bytes);
            remote_len -= 8;
        }

        while remote_len > 0 {
            let mut read_byte = [0u8; 1];

            socket.read_exact(&mut read_byte)
                .await
                .map_err(CaExchangeError::HandshakeError)?;
            trace!("remote nonce is {:?}", read_byte);
            remote_len -= 1;
        }

        Ok(CaExchangeOutput::new(socket))
    }
}

/// The result of a handshake. This implements AsyncRead and AsyncWrite and can therefore
/// be used as base for additional upgrades.
#[pin_project]
pub struct CaExchangeOutput<S> {
    #[pin]
    inner: S,
    buf: Vec<u8>,
}

impl<S: AsyncRead + AsyncWrite> CaExchangeOutput<S> {
    fn new(inner: S) -> Self {
        Self {
            inner: inner,
            buf: Vec::new(),
        }
    }
}

impl<S: AsyncRead + AsyncWrite> AsyncRead for CaExchangeOutput<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        self.project().inner.poll_read(cx, buf)
    }
}

impl<S: AsyncRead + AsyncWrite> AsyncWrite for CaExchangeOutput<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        self.project().inner.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.project().inner.poll_close(cx)
    }
}

/// Error when writing or reading private swarms
#[derive(Debug)]
pub enum CaExchangeError {
    /// Error during handshake.
    HandshakeError(IoError),
    /// I/O error.
    IoError(IoError),
}

impl From<IoError> for CaExchangeError {
    #[inline]
    fn from(err: IoError) -> CaExchangeError {
        CaExchangeError::IoError(err)
    }
}

impl error::Error for CaExchangeError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            CaExchangeError::HandshakeError(ref err) => Some(err),
            CaExchangeError::IoError(ref err) => Some(err),
        }
    }
}

impl fmt::Display for CaExchangeError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            CaExchangeError::HandshakeError(e) => write!(f, "Handshake error: {}", e),
            CaExchangeError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}


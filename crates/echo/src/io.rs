use std::future::Future;
use std::io;
use std::pin::Pin;

type IoFuture = Pin<Box<dyn Future<Output = Result<usize, io::Error>>>>;

pub trait AsyncRead {
    /// Reads data into the provided buffer, returning the number of bytes read.
    fn read(&mut self, buf: &mut [u8]) -> IoFuture;
}

pub trait AsyncWrite {
    /// Writes data from the provided buffer, returning the number of bytes written.
    fn write(&mut self, buf: &[u8]) -> IoFuture;
}

type Reader = Box<dyn crate::io::AsyncRead>;
type Writer = Box<dyn crate::io::AsyncWrite>;

pub struct Echo {
    buf: Vec<u8>,
    r: Reader,
    w: Writer,
}

impl Echo {
    fn new(buf_size: usize, read: Reader, write: Writer) -> Self {
        Self {
            buf: vec![0u8; buf_size],
            r: read,
            w: write,
        }
    }

    async fn run(&mut self) -> Result<(), std::io::Error> {
        loop {
            let bytes_read = self.r.read(&mut self.buf).await?;
            let mut bytes_written = 0;
            while bytes_written < bytes_read {
                let written = self.w.write(&self.buf[bytes_written..bytes_read]).await?;
                bytes_written += written;
            }
        }
    }
}

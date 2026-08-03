use std::io::Read;

use bytes::{Bytes, BytesMut};

pub trait ReadExt {
    /// Reads all bytes from the `Read` into a byte buffer.
    fn read_all(self, size: Option<usize>) -> std::io::Result<Bytes>;

    /// Reads all bytes from the `Read` into a byte buffer.
    /// Calls the `progress` callback after reading a chunk.
    fn read_progress<F>(self, size: Option<usize>, progress: F) -> std::io::Result<Bytes>
    where
        F: FnMut(usize);
}

impl<R> ReadExt for R
where
    R: Read,
{
    fn read_all(mut self, size: Option<usize>) -> std::io::Result<Bytes> {
        // Use 1MB if no size is specified
        let size = size.unwrap_or(1 * 1024 * 1024) as usize;
        let mut bytes = Vec::with_capacity(size);

        self.read_to_end(&mut bytes)?;
        Ok(bytes.into())
    }

    fn read_progress<F>(mut self, size: Option<usize>, mut progress: F) -> std::io::Result<Bytes>
    where
        F: FnMut(usize),
    {
        // Use 1MB if no size is specified
        let size = size.unwrap_or(1 * 1024 * 1024) as usize;
        let mut bytes = BytesMut::with_capacity(size);
        let mut buffer = [0; 32 * 1024];

        loop {
            // Read data into buffer, retry on interrupted
            let n = match self.read(&mut buffer) {
                Ok(n) => n,
                Err(e) if matches!(e.kind(), std::io::ErrorKind::Interrupted) => continue,
                Err(e) => return Err(e),
            };

            // Stop reading if end of stream is reached
            if n == 0 {
                break;
            }

            // Add read buffer to final buffer
            bytes.extend_from_slice(&buffer[..n]);

            // Call progress callback
            progress(bytes.len());
        }

        Ok(bytes.freeze())
    }
}

use embedded_io::blocking::{Read, ReadExactError, Seek, Write};
use embedded_io::SeekFrom;
use embedded_io::{Error, Io};

use crate::sd::{BlockIo, Lba};

#[derive(Debug)]
pub enum ErrorKind {
    ReadExactError,
}

impl Error for ErrorKind {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl<E> From<ReadExactError<E>> for ErrorKind {
    fn from(value: ReadExactError<E>) -> Self {
        Self::ReadExactError
    }
}

/// BS: Block Size, PS: Page Size
/// optimally PS would be in terms of BS, but const generics don't allow that yet
/// BS and PS must be powers of two, and PS must be as large or larger than BS
pub struct BufferedIo<const BS: usize, const PS: usize, IO: BlockIo<BS>> {
    io: IO,
    /// current stream position
    pos: usize,
    /// track the current page in the buffer
    page: Option<(Lba, [u8; PS])>,
}

impl<const BS: usize, const PS: usize, IO: BlockIo<BS>> BufferedIo<BS, PS, IO> {
    pub fn new(io: IO) -> Self {
        Self {
            io,
            pos: 0,
            page: None,
        }
    }

    fn lba(&self) -> Lba {
        // floor to PS, but in terms of BS
        (self.pos / PS * PS / BS) as Lba
    }
}

impl<const BS: usize, const PS: usize, IO: BlockIo<BS>> Io for BufferedIo<BS, PS, IO> {
    type Error = ErrorKind;
}

impl<const BS: usize, const PS: usize, IO: BlockIo<BS>> Read for BufferedIo<BS, PS, IO> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        // ensure the page at self.pos is loaded
        let (lba, page) = self
            .page
            .filter(|(lba, _)| *lba == self.lba())
            .unwrap_or_else(|| {
                let lba = self.lba();
                let mut buf = [0; PS];
                self.io.read_blocks(lba, &mut buf).unwrap();
                (lba, buf)
            });
        self.page = Some((lba, page));

        // offset inside page
        let offset = self.pos - (lba as usize * BS);
        let end = PS.min(offset + buf.len());
        let len = end - offset;

        buf[..len].copy_from_slice(&page[offset..end]);
        self.pos += len;

        Ok(len)
    }
}

impl<const BS: usize, const PS: usize, IO: BlockIo<BS>> Write for BufferedIo<BS, PS, IO> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.pos += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<const BS: usize, const PS: usize, IO: BlockIo<BS>> Seek for BufferedIo<BS, PS, IO> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorKind> {
        match pos {
            SeekFrom::Start(addr) => {
                self.pos = addr as usize;
            }
            SeekFrom::End(addr) => {
                self.pos = usize::MAX - addr as usize;
            }
            SeekFrom::Current(addr) => {
                self.pos += addr as usize;
            }
        }

        Ok(self.pos as u64)
    }
}

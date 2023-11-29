use std::{
    cmp::Ordering,
    io::{Read, Result, Seek, SeekFrom},
};

struct BoxReader<R> {
    reader: R,
    len: u64,
}

impl<R: Seek + Read> BoxReader<R> {
    pub fn new(mut r: R) -> Result<BoxReader<R>> {
        let len = r.seek(SeekFrom::End(0))?;
        r.seek(SeekFrom::Start(0))?;
        Ok(BoxReader { reader: r, len })
    }
}
/// Multiple stream readers
#[derive(Default)]
pub struct StreamReaders<R> {
    buf: Vec<BoxReader<R>>,
    index: usize,
    seek: u64,
    len: u64,
}

impl<R: Read + Seek> StreamReaders<R> {
    /// Create a empty `StreamReaders`
    pub fn new() -> StreamReaders<R> {
        Self {
            buf: Vec::new(),
            index: 0,
            seek: 0,
            len: 0,
        }
    }
    /// Appends an element.
    pub fn push(&mut self, value: R) -> Result<()> {
        let reader = BoxReader::new(value)?;
        if reader.len > 0 {
            self.len += reader.len;
            self.buf.push(reader);
        }
        Ok(())
    }
    /// Return `true` if no element
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
    /// Return the length of the stream
    ///
    /// # Examples
    /// ```
    /// use readers::*;
    /// let bytes = b"hello world";
    /// let mut reader = StreamReaders::new();
    /// reader.push(BytesReader::new(bytes)).unwrap();
    /// assert_eq!(reader.len(), 11)
    /// ```
    pub fn len(&self) -> u64 {
        self.len
    }
    /// Return the position
    pub fn pos(&self) -> u64 {
        let mut pos = self.seek;
        for r in &self.buf[..self.index] {
            pos += r.len;
        }
        pos
    }
    fn add_offset(&mut self, offset: u64) -> Result<u64> {
        if self.len > offset + self.pos() {
            let remain = self.buf[self.index].len - self.seek - 1;
            if remain >= offset {
                self.seek = self.buf[self.index]
                    .reader
                    .seek(SeekFrom::Current(offset as i64))?;
            } else {
                self.index += 1;
                self.seek = offset - remain - 1;
                while self.seek > self.buf[self.index].len {
                    self.seek -= self.buf[self.index].len;
                    self.index += 1;
                }
                self.buf[self.index]
                    .reader
                    .seek(SeekFrom::Start(self.seek))?;
            }
            Ok(self.pos())
        } else {
            self.seek_end()?;
            Ok(if self.is_empty() { 0 } else { self.len - 1 })
        }
    }
    fn sub_offset(&mut self, offset: u64) -> Result<u64> {
        if self.pos() >= offset {
            if self.seek >= offset {
                self.seek = self.buf[self.index]
                    .reader
                    .seek(SeekFrom::Current(-(offset as i64)))?;
            } else {
                self.index -= 1;
                let mut n = offset as i64 - self.seek as i64 - 1;
                while n < 0 {
                    n += self.buf[self.index].len as i64;
                    self.index -= 1;
                }
                self.buf[self.index].reader.seek(SeekFrom::End(n.abs()))?;
            }
            Ok(self.pos())
        } else {
            self.seek_start()?;
            Ok(0)
        }
    }
    fn seek_start(&mut self) -> Result<()> {
        self.index = 0;
        self.seek = 0;
        for r in &mut self.buf {
            r.reader.rewind()?;
        }
        Ok(())
    }

    fn seek_end(&mut self) -> Result<()> {
        if self.buf.is_empty() {
            return Ok(());
        }
        for r in &mut self.buf {
            r.reader.rewind()?;
        }
        self.index = self.buf.len() - 1;
        self.seek = self.buf[self.index].reader.seek(SeekFrom::End(0))?;
        Ok(())
    }
}

impl<R: Read + Seek> Read for StreamReaders<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.pos() >= self.len {
            return Ok(0);
        }
        let len = self.buf[self.index].reader.read(buf)?;
        self.seek += len as u64;
        if len < buf.len() {
            self.index += 1;
            self.seek = 0;
            Ok(self.read(&mut buf[len..])? + len)
        } else {
            if self.seek >= self.buf[self.index].len {
                self.index += 1;
                self.seek = 0;
            }
            Ok(len)
        }
    }
}

impl<R: Read + Seek> Seek for StreamReaders<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Current(i) => match i.cmp(&0) {
                Ordering::Equal => Ok(self.pos()),
                Ordering::Greater => self.add_offset(i.unsigned_abs()),
                Ordering::Less => self.sub_offset(i.unsigned_abs()),
            }
            .map_err(Into::into),
            SeekFrom::End(end) => {
                if end >= 0 {
                    self.seek_end()?;
                    Ok(self.len)
                } else {
                    self.seek_end()?;
                    self.sub_offset(end.unsigned_abs()).map_err(Into::into)
                }
            }
            SeekFrom::Start(start) => {
                self.seek_start()?;
                self.add_offset(start).map_err(Into::into)
            }
        }
    }
}

#[allow(unused)]
mod test {

    use super::*;
    use std::io::BufReader;
    #[test]
    fn test() -> std::io::Result<()> {
        std::fs::write("1", b"Hello,")?;
        std::fs::write("2", b"Rust!")?;
        let f1 = std::fs::File::open("1")?;
        let f2 = std::fs::File::open("2")?;
        let mut readers = StreamReaders::new();
        readers.push(f1)?;
        readers.push(f2)?;
        let mut reader = BufReader::new(readers);
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        assert_eq!("Hello,Rust!", buf.as_str());
        Ok(())
    }
}

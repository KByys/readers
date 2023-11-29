use std::io::{Read, Seek, SeekFrom};


/// A byte slice wrapper and implements [`Read`][std::io::Read] and [Seek][std::io::Seek]
/// # Usage
/// ```
/// use std::io::BufReader;
/// use readers::BytesReader;
/// use std::io::Read;
/// let bytes = b"hello world";
/// let mut reader = BufReader::new(BytesReader::new(bytes));
/// let mut buf = String::new();
/// reader.read_to_string(&mut buf).unwrap();
/// 
/// assert_eq!(buf.as_str(), "hello world");
/// ```
/// 
#[derive(Debug)]
pub struct BytesReader<'a> {
    buf: &'a [u8],
    index: usize,
}
impl<'a> BytesReader<'a> {
    /// Create a `BytesReader` from the byte slice
    pub fn new(buf: &'a [u8]) -> BytesReader<'a> {
        Self { buf, index: 0 }
    }
    /// Returns `true` if the bytes slice has a length of 0.
    /// 
    /// # Examples
    /// ```
    /// use readers::BytesReader;
    /// 
    /// let bytes = [];
    /// let reader = BytesReader::new(&bytes);
    /// assert!(reader.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
    /// Returns the number of elements in the bytes slice.
    /// 
    /// # Examples
    /// ```
    /// use readers::BytesReader;
    /// 
    /// let bytes = [1, 2];
    /// let reader = BytesReader::new(&bytes);
    /// assert_eq!(reader.len(), 2)
    /// ```
    pub fn len(&self) -> usize {
        self.buf.len()
    }
    /// Returns the position in the bytes slice
    /// 
    /// # Examples
    /// ```
    /// use std::io::Read;
    /// use readers::BytesReader;
    /// 
    /// let bytes = [1, 2, 3];
    /// let mut reader = BytesReader::new(&bytes);
    /// let mut buf = [0; 2];
    /// assert_eq!(2, reader.read(&mut buf).unwrap());
    /// assert_eq!(2, reader.pos());
    /// ```
    pub fn pos(&self) -> usize {
        self.index
    }
}

impl<'a> Seek for BytesReader<'a> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Current(i) => {
                if i >= 0 {
                    self.index += i.unsigned_abs() as usize;
                    if self.index >= self.buf.len() {
                        self.index = self.len();
                    }
                } else {
                    let u = i.unsigned_abs() as usize;
                    if u >= self.index {
                        self.index = 0;
                    } else {
                        self.index -= u;
                    }
                }
            }
            SeekFrom::End(e) => {
                if e >= 0 {
                    self.index = self.len();
                } else {
                    let e = e.unsigned_abs() as usize;
                    if e > self.buf.len() {
                        self.index = 0;
                    } else {
                        self.index -= e;
                    }
                }
            }
            SeekFrom::Start(s) => {
                let s = s as usize;
                self.index = if s >= self.buf.len() {
                    self.len()
                } else {
                    s
                };
            }
        }
        Ok(self.index as u64)
    }
}

impl<'a> Read for BytesReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.index >= self.buf.len() {
            return Ok(0);
        }
        let remain = self.buf.len() - self.index;
        if remain >= buf.len() {
            buf.copy_from_slice(&self.buf[self.index..self.index + buf.len()]);
            self.index += buf.len();
            Ok(buf.len())
        } else {
            buf[..remain].copy_from_slice(&self.buf[self.index..]);
            self.index = self.len();
            Ok(remain)
        }
    }
}

#[allow(unused)]
mod test {
    use std::io::{Seek, SeekFrom, Read, Result};
    
    use super::BytesReader;    
    #[test]
    fn test_seek() -> Result<()> {
        let bytes = vec![0, 1, 2, 3, 4, 5];
        let mut seek = BytesReader::new(&bytes);
        seek.seek(SeekFrom::Start(1))?;
        assert_eq!(seek.pos(), 1);
        seek.seek(SeekFrom::Start(100))?;
        assert_eq!(seek.pos(), seek.len());


        seek.seek(SeekFrom::End(0))?;
        assert_eq!(seek.pos(), seek.len());
        seek.seek(SeekFrom::End(100))?;
        assert_eq!(seek.pos(), seek.len());
        seek.seek(SeekFrom::End(-1))?;
        assert_eq!(seek.pos(), seek.len() - 1);

        seek.seek(SeekFrom::Current(-3))?;
        assert_eq!(seek.pos(), 2);
        seek.seek(SeekFrom::Current(3))?;
        assert_eq!(seek.pos(), seek.len() - 1);


        Ok(())
    }
    #[test]
    fn test_read() -> Result<()> {
        let bytes = vec![0, 1, 2, 3, 4];
        let mut reader = BytesReader::new(&bytes);
        let mut buf = [0; 3];
        assert_eq!(3, reader.read(&mut buf)?);
        assert_eq!([0, 1, 2], buf);

        assert_eq!(2, reader.read(&mut buf)?);
        assert_eq!([3, 4, 2], buf);

        assert_eq!(0, reader.read(&mut buf)?);
        Ok(())
    }
}
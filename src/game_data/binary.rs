use std::io;
pub(super) fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
pub(super) struct Reader<'a> {
    pub bytes: &'a [u8],
    pub pos: usize,
}
impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    pub fn take(&mut self, len: usize) -> io::Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| invalid("binary offset overflow"))?;
        let bytes = self
            .bytes
            .get(self.pos..end)
            .ok_or_else(|| invalid(format!("truncated data at {}", self.pos)))?;
        self.pos = end;
        Ok(bytes)
    }
    pub fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    pub fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    pub fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    pub fn count(&mut self, limit: usize) -> io::Result<usize> {
        let count = self.u32()? as usize;
        if count > limit {
            return Err(invalid("binary count exceeds limit"));
        }
        Ok(count)
    }
    pub fn flag(&mut self) -> io::Result<bool> {
        match self.u32()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid("invalid binary flag")),
        }
    }
    pub fn string(&mut self) -> io::Result<String> {
        let count = self.i32()?;
        if count == 0 {
            return Ok(String::new());
        }
        let count_bytes = (count.unsigned_abs() as usize)
            .checked_mul(if count < 0 { 2 } else { 1 })
            .filter(|n| *n <= 1024 * 1024)
            .ok_or_else(|| invalid("string exceeds limit"))?;
        let bytes = self.take(count_bytes)?;
        let value = if count < 0 {
            if bytes[bytes.len() - 2..] != [0, 0] {
                return Err(invalid("unterminated UTF-16 string"));
            }
            let words: Vec<_> = bytes[..bytes.len() - 2]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u16::from_le_bytes(*b))
                .collect();
            String::from_utf16(&words).map_err(|_| invalid("invalid UTF-16"))?
        } else {
            if bytes.last() != Some(&0) {
                return Err(invalid("unterminated UTF-8 string"));
            }
            std::str::from_utf8(&bytes[..bytes.len() - 1])
                .map_err(|_| invalid("invalid UTF-8"))?
                .to_owned()
        };
        if value.contains('\0') {
            return Err(invalid("embedded NUL"));
        }
        Ok(value)
    }
}

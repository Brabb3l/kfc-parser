use std::io::{Error, ErrorKind, Read, Result, Seek, Write};

pub trait ReadExt: Read {

    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    #[inline]
    fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    #[inline]
    fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    #[inline]
    fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    #[inline]
    fn read_i8(&mut self) -> Result<i8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0] as i8)
    }

    #[inline]
    fn read_i16(&mut self) -> Result<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    #[inline]
    fn read_i32(&mut self) -> Result<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    #[inline]
    fn read_i64(&mut self) -> Result<i64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(i64::from_le_bytes(buf))
    }

    #[inline]
    fn read_f32(&mut self) -> Result<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    #[inline]
    fn read_f64(&mut self) -> Result<f64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }

    #[inline]
    fn read_string(&mut self, len: usize) -> Result<String> {
        let mut buf = vec![0; len];
        self.read_exact(&mut buf)?;
        String::from_utf8(buf)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid utf-8"))
    }

    #[inline]
    fn read_exact_n(&mut self, len: usize, buf: &mut Vec<u8>) -> Result<()> {
        let mut chunk = self.take(len as u64);
        assert_eq!(chunk.read_to_end(buf)?, len);
        Ok(())
    }

    #[inline]
    fn padding(&mut self, n: usize) -> Result<()> {
        let mut buf = vec![0; n];
        self.read_exact(&mut buf)?;

        for byte in buf {
            if byte != 0 {
                return Err(Error::new(ErrorKind::InvalidData, "padding is not 0"));
            }
        }

        Ok(())
    }
}

impl<T: Read> ReadExt for T {}

pub trait ReadSeekExt: Read + Seek + Sized {

    #[inline]
    fn read_u32_offset(&mut self) -> Result<u64> {
        let pos = self.stream_position()?;
        let offset = self.read_u32()? as u64;

        Ok(pos + offset)
    }

    #[inline]
    fn align(&mut self, alignment: usize) -> Result<usize> {
        let pos = self.stream_position()? as usize;
        let padding = (alignment - (pos % alignment)) % alignment;
        self.padding(padding)?;
        Ok(padding)
    }
}

impl<T: Read + Seek> ReadSeekExt for T {}

pub trait WriteExt: Write {

    #[inline]
    fn write_u8(&mut self, n: u8) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_u16(&mut self, n: u16) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_u32(&mut self, n: u32) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_u64(&mut self, n: u64) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_i8(&mut self, n: i8) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_i16(&mut self, n: i16) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_i32(&mut self, n: i32) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_i64(&mut self, n: i64) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_f32(&mut self, n: f32) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_f64(&mut self, n: f64) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_string(&mut self, s: &str, len: usize) -> Result<()> {
        let s = s.as_bytes();
        let s = &s[..len];
        self.write_all(s)
    }

    #[inline]
    fn padding(&mut self, n: u64) -> Result<()> {
        self.write_all(&vec![0; n as usize])
    }

}

impl<T: Write> WriteExt for T {}

pub trait WriteSeekExt: Write + Seek + Sized {

    #[inline]
    fn write_offset(&mut self, offset: u64) -> Result<()> {
        if offset == 0 {
            self.write_u32(0)?;
            return Ok(());
        }

        let pos = self.stream_position()?;
        let relative_offset = offset - pos;

        self.write_u32(relative_offset as u32)?;

        Ok(())
    }

    #[inline]
    fn align(&mut self, alignment: usize) -> Result<usize> {
        let pos = self.stream_position()? as usize;
        let padding = (alignment - (pos % alignment)) % alignment;
        self.write_all(&vec![0; padding])?;
        Ok(padding)
    }

    #[inline]
    fn align_with(&mut self, alignment: usize, value: u8) -> Result<usize> {
        let pos = self.stream_position()? as usize;
        let padding = (alignment - (pos % alignment)) % alignment;
        self.write_all(&vec![value; padding])?;
        Ok(padding)
    }

}

impl<T: Write + Seek> WriteSeekExt for T {}

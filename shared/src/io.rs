use std::io::{Read, Seek, SeekFrom, Write};

pub struct Reader<T> {
    src: T
}

impl<T> Reader<T> {
    pub fn new(src: T) -> Self {
        Self {
            src
        }
    }

    pub fn get_ref(&self) -> &T {
        &self.src
    }

    pub fn into_inner(self) -> T {
        self.src
    }
}

impl<T: Read> Reader<T> {
    pub fn read_u8(&mut self) -> std::io::Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    pub fn read_u16(&mut self) -> std::io::Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub fn read_u32(&mut self) -> std::io::Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_u64(&mut self) -> std::io::Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    pub fn read_i8(&mut self) -> std::io::Result<i8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0] as i8)
    }

    pub fn read_i16(&mut self) -> std::io::Result<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    pub fn read_i32(&mut self) -> std::io::Result<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    pub fn read_i64(&mut self) -> std::io::Result<i64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(i64::from_le_bytes(buf))
    }

    pub fn read_f32(&mut self) -> std::io::Result<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    pub fn read_f64(&mut self) -> std::io::Result<f64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }

    pub fn read_string(&mut self, len: usize) -> std::io::Result<String> {
        let mut buf = vec![0; len];
        self.read_exact(&mut buf)?;
        Ok(String::from_utf8(buf).unwrap())
    }
}

impl<T: Read + Seek> Reader<T> {
    pub fn read_u32_offset(&mut self) -> std::io::Result<u64> {
        Ok(self.stream_position()? + self.read_u32()? as u64)
    }
}

impl<T: Read> Read for Reader<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.src.read(buf)
    }
}

impl<T: Seek> Seek for Reader<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.src.seek(pos)
    }
}

pub struct Writer<T> {
    dst: T
}

impl<T> Writer<T> {
    pub fn new(dst: T) -> Self {
        Self {
            dst
        }
    }

    pub fn get_ref(&self) -> &T {
        &self.dst
    }

    pub fn into_inner(self) -> T {
        self.dst
    }
}

impl<T: Write> Writer<T> {
    pub fn write_u8(&mut self, n: u8) -> std::io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    pub fn write_u16(&mut self, n: u16) -> std::io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    pub fn write_u32(&mut self, n: u32) -> std::io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    pub fn write_u64(&mut self, n: u64) -> std::io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    pub fn write_i8(&mut self, n: i8) -> std::io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    pub fn write_i16(&mut self, n: i16) -> std::io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    pub fn write_i32(&mut self, n: i32) -> std::io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    pub fn write_i64(&mut self, n: i64) -> std::io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    pub fn write_f32(&mut self, n: f32) -> std::io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    pub fn write_f64(&mut self, n: f64) -> std::io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    pub fn write_string(&mut self, s: &str, len: usize) -> std::io::Result<()> {
        let s = s.as_bytes();
        let s = &s[..len];
        self.write_all(s)
    }
}

impl<T: Write> Write for Writer<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.dst.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.dst.flush()
    }
}

impl<T: Seek> Seek for Writer<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.dst.seek(pos)
    }
}

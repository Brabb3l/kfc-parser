use half::f16;

use super::{ByteOrder, Buffer, Result};

impl Buffer {

    pub fn write_bool(&mut self, value: bool) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(1)?;

        self.data[self.position] = if value { 1 } else { 0 };
        self.position += 1;

        Ok(())
    }

    pub fn write_u8(&mut self, value: u8) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(1)?;

        self.data[self.position] = value;
        self.position += 1;

        Ok(())
    }

    pub fn write_i8(&mut self, value: i8) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(1)?;

        self.data[self.position] = value as u8;
        self.position += 1;

        Ok(())
    }

    pub fn write_u16(&mut self, value: u16) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(2)?;

        match self.order {
            ByteOrder::BigEndian => self.data[self.position..self.position + 2]
                .copy_from_slice(&value.to_be_bytes()),
            ByteOrder::LittleEndian => self.data[self.position..self.position + 2]
                .copy_from_slice(&value.to_le_bytes()),
        }
        self.position += 2;

        Ok(())
    }

    pub fn write_i16(&mut self, value: i16) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(2)?;

        match self.order {
            ByteOrder::BigEndian => self.data[self.position..self.position + 2]
                .copy_from_slice(&value.to_be_bytes()),
            ByteOrder::LittleEndian => self.data[self.position..self.position + 2]
                .copy_from_slice(&value.to_le_bytes()),
        }
        self.position += 2;

        Ok(())
    }

    pub fn write_u32(&mut self, value: u32) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(4)?;

        match self.order {
            ByteOrder::BigEndian => self.data[self.position..self.position + 4]
                .copy_from_slice(&value.to_be_bytes()),
            ByteOrder::LittleEndian => self.data[self.position..self.position + 4]
                .copy_from_slice(&value.to_le_bytes()),
        }
        self.position += 4;

        Ok(())
    }

    pub fn write_i32(&mut self, value: i32) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(4)?;

        match self.order {
            ByteOrder::BigEndian => self.data[self.position..self.position + 4]
                .copy_from_slice(&value.to_be_bytes()),
            ByteOrder::LittleEndian => self.data[self.position..self.position + 4]
                .copy_from_slice(&value.to_le_bytes()),
        }
        self.position += 4;

        Ok(())
    }

    pub fn write_u64(&mut self, value: u64) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(8)?;

        match self.order {
            ByteOrder::BigEndian => self.data[self.position..self.position + 8].copy_from_slice(&value.to_be_bytes()),
            ByteOrder::LittleEndian => self.data[self.position..self.position + 8].copy_from_slice(&value.to_le_bytes()),
        }
        self.position += 8;

        Ok(())
    }

    pub fn write_i64(&mut self, value: i64) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(8)?;

        match self.order {
            ByteOrder::BigEndian => self.data[self.position..self.position + 8]
                .copy_from_slice(&value.to_be_bytes()),
            ByteOrder::LittleEndian => self.data[self.position..self.position + 8]
                .copy_from_slice(&value.to_le_bytes()),
        }
        self.position += 8;

        Ok(())
    }

    pub fn write_f16(&mut self, value: f16) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(2)?;

        match self.order {
            ByteOrder::BigEndian => self.data[self.position..self.position + 2]
                .copy_from_slice(&value.to_be_bytes()),
            ByteOrder::LittleEndian => self.data[self.position..self.position + 2]
                .copy_from_slice(&value.to_le_bytes()),
        }
        self.position += 2;

        Ok(())
    }

    pub fn write_f32(&mut self, value: f32) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(4)?;

        match self.order {
            ByteOrder::BigEndian => self.data[self.position..self.position + 4]
                .copy_from_slice(&value.to_be_bytes()),
            ByteOrder::LittleEndian => self.data[self.position..self.position + 4]
                .copy_from_slice(&value.to_le_bytes()),
        }
        self.position += 4;

        Ok(())
    }

    pub fn write_f64(&mut self, value: f64) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(8)?;

        match self.order {
            ByteOrder::BigEndian => self.data[self.position..self.position + 8]
                .copy_from_slice(&value.to_be_bytes()),
            ByteOrder::LittleEndian => self.data[self.position..self.position + 8]
                .copy_from_slice(&value.to_le_bytes()),
        }
        self.position += 8;

        Ok(())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(bytes.len())?;

        self.data[self.position..self.position + bytes.len()].copy_from_slice(bytes);
        self.position += bytes.len();

        Ok(())
    }

}

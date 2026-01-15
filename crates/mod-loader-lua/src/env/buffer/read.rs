use half::f16;

use super::{ByteOrder, Buffer, Result};

impl Buffer {

    pub fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_read_bounds(1)?;

        let value = self.data[self.head];
        self.head += 1;

        Ok(value)
    }

    pub fn read_i8(&mut self) -> Result<i8> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_read_bounds(1)?;

        let value = self.data[self.head] as i8;
        self.head += 1;

        Ok(value)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_read_bounds(2)?;

        let value = match self.order {
            ByteOrder::BigEndian => u16::from_be_bytes(self.data[self.head..self.head + 2].try_into().unwrap()),
            ByteOrder::LittleEndian => u16::from_le_bytes(self.data[self.head..self.head + 2].try_into().unwrap()),
        };
        self.head += 2;

        Ok(value)
    }

    pub fn read_i16(&mut self) -> Result<i16> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_read_bounds(2)?;

        let value = match self.order {
            ByteOrder::BigEndian => i16::from_be_bytes(self.data[self.head..self.head + 2].try_into().unwrap()),
            ByteOrder::LittleEndian => i16::from_le_bytes(self.data[self.head..self.head + 2].try_into().unwrap()),
        };
        self.head += 2;

        Ok(value)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_read_bounds(4)?;

        let value = match self.order {
            ByteOrder::BigEndian => u32::from_be_bytes(self.data[self.head..self.head + 4].try_into().unwrap()),
            ByteOrder::LittleEndian => u32::from_le_bytes(self.data[self.head..self.head + 4].try_into().unwrap()),
        };
        self.head += 4;

        Ok(value)
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_read_bounds(4)?;

        let value = match self.order {
            ByteOrder::BigEndian => i32::from_be_bytes(self.data[self.head..self.head + 4].try_into().unwrap()),
            ByteOrder::LittleEndian => i32::from_le_bytes(self.data[self.head..self.head + 4].try_into().unwrap()),
        };
        self.head += 4;

        Ok(value)
    }

    pub fn read_u64(&mut self) -> Result<u64> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_read_bounds(8)?;

        let value = match self.order {
            ByteOrder::BigEndian => u64::from_be_bytes(self.data[self.head..self.head + 8].try_into().unwrap()),
            ByteOrder::LittleEndian => u64::from_le_bytes(self.data[self.head..self.head + 8].try_into().unwrap()),
        };
        self.head += 8;

        Ok(value)
    }

    pub fn read_i64(&mut self) -> Result<i64> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_read_bounds(8)?;

        let value = match self.order {
            ByteOrder::BigEndian => i64::from_be_bytes(self.data[self.head..self.head + 8].try_into().unwrap()),
            ByteOrder::LittleEndian => i64::from_le_bytes(self.data[self.head..self.head + 8].try_into().unwrap()),
        };
        self.head += 8;

        Ok(value)
    }

    pub fn read_f16(&mut self) -> Result<f16> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_read_bounds(2)?;

        let value = match self.order {
            ByteOrder::BigEndian => f16::from_be_bytes(self.data[self.head..self.head + 2].try_into().unwrap()),
            ByteOrder::LittleEndian => f16::from_le_bytes(self.data[self.head..self.head + 2].try_into().unwrap()),
        };
        self.head += 2;

        Ok(value)
    }

    pub fn read_f32(&mut self) -> Result<f32> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_read_bounds(4)?;

        let value = match self.order {
            ByteOrder::BigEndian => f32::from_be_bytes(self.data[self.head..self.head + 4].try_into().unwrap()),
            ByteOrder::LittleEndian => f32::from_le_bytes(self.data[self.head..self.head + 4].try_into().unwrap()),
        };
        self.head += 4;

        Ok(value)
    }

    pub fn read_f64(&mut self) -> Result<f64> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_read_bounds(8)?;

        let value = match self.order {
            ByteOrder::BigEndian => f64::from_be_bytes(self.data[self.head..self.head + 8].try_into().unwrap()),
            ByteOrder::LittleEndian => f64::from_le_bytes(self.data[self.head..self.head + 8].try_into().unwrap()),
        };
        self.head += 8;

        Ok(value)
    }

    pub fn read_bytes(&mut self, size: usize) -> Result<&[u8]> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_read_bounds(size)?;

        let bytes = &self.data[self.head..self.head + size];
        self.head += size;

        Ok(bytes)
    }

}

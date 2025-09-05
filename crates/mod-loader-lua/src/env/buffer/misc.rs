use super::{BufferError, ByteOrder, Buffer, BufferState, Result};

#[allow(unused)]
impl Buffer {

    fn readable(&self) -> bool {
        matches!(self.state, BufferState::ReadWrite | BufferState::Read)
    }

    pub fn set_readable(&mut self, state: bool) -> Result<()> {
        if state {
            match self.state {
                BufferState::ReadWrite | BufferState::Read => {},
                BufferState::Write => self.state = BufferState::Read,
                BufferState::Closed => return Err(BufferError::Closed),
            }
        } else {
            match self.state {
                BufferState::ReadWrite | BufferState::Write => self.state = BufferState::Write,
                BufferState::Read => self.state = BufferState::Write,
                BufferState::Closed => return Err(BufferError::Closed),
            }
        }

        Ok(())
    }

    fn writable(&self) -> bool {
        matches!(self.state, BufferState::ReadWrite | BufferState::Write)
    }

    pub fn set_writable(&mut self, state: bool) -> Result<()> {
        if state {
            match self.state {
                BufferState::ReadWrite | BufferState::Write => {},
                BufferState::Read => self.state = BufferState::ReadWrite,
                BufferState::Closed => return Err(BufferError::Closed),
            }
        } else {
            match self.state {
                BufferState::ReadWrite | BufferState::Read => self.state = BufferState::Read,
                BufferState::Write => self.state = BufferState::Write,
                BufferState::Closed => return Err(BufferError::Closed),
            }
        }

        Ok(())
    }

    pub fn close(&mut self) {
        self.state = BufferState::Closed;
    }

    pub fn position(&self) -> Result<usize> {
        self.check_not_closed()?;

        Ok(self.position)
    }

    pub fn set_position(&mut self, position: usize) -> Result<()> {
        self.check_not_closed()?;

        if position > self.limit {
            return Err(BufferError::Overflow {
                position,
                limit: self.limit,
            });
        }

        self.position = position;

        Ok(())
    }

    pub fn limit(&self) -> Result<usize> {
        self.check_not_closed()?;

        Ok(self.limit)
    }

    pub fn set_limit(&mut self, limit: usize) -> Result<()> {
        self.check_not_closed()?;

        if limit > self.data.len() {
            return Err(BufferError::LimitOverflow {
                limit,
                capacity: self.data.len(),
            });
        }

        self.limit = limit;

        Ok(())
    }

    pub fn remaining(&self) -> Result<usize> {
        self.check_not_closed()?;

        if self.position > self.limit {
            Ok(0)
        } else {
            Ok(self.limit - self.position)
        }
    }

    pub fn capacity(&self) -> Result<usize> {
        self.check_not_closed()?;

        Ok(self.data.len())
    }

    pub fn flip(&mut self) -> Result<()> {
        self.check_not_closed()?;

        self.limit = self.position;
        self.position = 0;

        Ok(())
    }

    pub fn rewind(&mut self) -> Result<()> {
        self.check_not_closed()?;

        self.position = 0;

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        self.check_not_closed()?;

        self.position = 0;
        self.limit = 0;

        Ok(())
    }

    pub fn reserve(&mut self, additional: usize) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;

        let (new_capacity, overflowing) = self.position.overflowing_add(additional);

        if overflowing {
            return Err(BufferError::CapacityOverflow);
        }

        if self.data.len() < new_capacity {
            self.data.reserve_exact(new_capacity - self.data.len());
            self.data.resize(new_capacity, 0);
        }

        Ok(())
    }

    pub fn order(&self) -> Result<ByteOrder> {
        self.check_not_closed()?;

        Ok(self.order)
    }

    pub fn set_order(&mut self, order: ByteOrder) -> Result<()> {
        self.check_not_closed()?;

        self.order = order;

        Ok(())
    }

    pub fn skip(&mut self, size: usize) -> Result<()> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_read_bounds(size)?;

        self.position += size;

        Ok(())
    }

    pub fn copy(&mut self, src: &[u8]) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(src.len())?;

        self.data[self.position..self.position + src.len()].copy_from_slice(src);
        self.position += src.len();

        Ok(())
    }

    pub fn copy_within(&mut self, pos: usize, len: usize) -> Result<()> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_writable()?;
        self.check_read_bounds_at(pos, len)?;

        self.reserve(len)?;
        self.data.copy_within(pos..pos + len, self.position);
        self.position += len;

        Ok(())
    }

    pub fn data(&self) -> Result<&[u8]> {
        self.check_not_closed()?;

        if self.position >= self.limit {
            return Ok(&[]);
        }

        Ok(&self.data[self.position..self.limit])
    }

    pub(super) fn check_not_closed(&self) -> Result<()> {
        if self.state == BufferState::Closed {
            return Err(BufferError::Closed);
        }

        Ok(())
    }

    pub(super) fn check_readable(&self) -> Result<()> {
        if !self.readable() {
            return Err(BufferError::NotReadable);
        }

        Ok(())
    }

    pub(super) fn check_writable(&self) -> Result<()> {
        if !self.writable() {
            return Err(BufferError::NotWritable);
        }

        Ok(())
    }

    pub(super) fn check_read_bounds(&self, size: usize) -> Result<()> {
        let n = self.position.saturating_add(size);

        if n > self.limit {
            return Err(BufferError::Overflow {
                position: n,
                limit: self.limit,
            });
        }

        Ok(())
    }

    pub(super) fn check_read_bounds_at(&self, pos: usize, size: usize) -> Result<()> {
        let n = pos.saturating_add(size);

        if n > self.limit {
            return Err(BufferError::Overflow {
                position: n,
                limit: self.limit,
            });
        }

        Ok(())
    }

}

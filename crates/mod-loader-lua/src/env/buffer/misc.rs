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

    pub fn head(&self) -> Result<usize> {
        self.check_not_closed()?;

        Ok(self.head)
    }

    pub fn set_head(&mut self, position: usize) -> Result<()> {
        self.check_not_closed()?;

        if position > self.tail {
            return Err(BufferError::Overflow {
                head: position,
                tail: self.tail,
            });
        }

        self.head = position;

        Ok(())
    }

    pub fn tail(&self) -> Result<usize> {
        self.check_not_closed()?;

        Ok(self.tail)
    }

    pub fn set_tail(&mut self, position: usize) -> Result<()> {
        self.check_not_closed()?;

        if position > self.data.len() {
            return Err(BufferError::TailOverflow {
                tail: position,
                capacity: self.data.len(),
            });
        }

        self.tail = position;

        Ok(())
    }

    pub fn remaining(&self) -> Result<usize> {
        self.check_not_closed()?;

        Ok(self.tail.saturating_sub(self.head))
    }

    pub fn capacity(&self) -> Result<usize> {
        self.check_not_closed()?;

        Ok(self.data.len())
    }

    pub fn reset(&mut self) -> Result<()> {
        self.check_not_closed()?;

        self.head = 0;
        self.tail = 0;

        Ok(())
    }

    pub fn reserve(&mut self, additional: usize) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;

        let (new_capacity, overflowing) = self.tail.overflowing_add(additional);

        if overflowing || new_capacity > isize::MAX as usize {
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

        self.head += size;

        Ok(())
    }

    pub fn copy(&mut self, src: &[u8]) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(src.len())?;

        self.data[self.tail..self.tail + src.len()].copy_from_slice(src);
        self.tail += src.len();

        Ok(())
    }

    // TODO: this is currently very unintuitive, needs to be reworked
    pub fn copy_within(&mut self, pos: usize, len: usize) -> Result<()> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_writable()?;
        self.check_read_bounds(pos.saturating_add(len))?;

        let pos = self.head.saturating_add(pos);

        self.reserve(len)?;
        self.data.copy_within(pos..pos + len, self.tail);
        self.tail += len;

        Ok(())
    }

    pub fn data(&self) -> Result<&[u8]> {
        self.check_not_closed()?;

        if self.head >= self.tail {
            return Ok(&[]);
        }

        Ok(&self.data[self.head..self.tail])
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
        let n = self.head.saturating_add(size);

        if n > self.tail {
            return Err(BufferError::Overflow {
                head: n,
                tail: self.tail,
            });
        }

        Ok(())
    }

    pub(super) fn check_read_bounds_at(&self, pos: usize, size: usize) -> Result<()> {
        let n = pos.saturating_add(size);

        if n > self.tail {
            return Err(BufferError::Overflow {
                head: n,
                tail: self.tail,
            });
        }

        Ok(())
    }

}

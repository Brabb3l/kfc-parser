use std::io::Write;
use shared::io::WriteExt;

use super::{BlobGuid, DescriptorGuid};

impl BlobGuid {
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.data)
    }
}

impl DescriptorGuid {
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.data)?;
        writer.write_u32(self.type_hash)?;
        writer.write_u32(self.part_number)?;
        writer.padding(8)?;

        Ok(())
    }
}

mod read;
mod write;

use std::fmt::{Debug, Display};
use std::io::{Cursor, Write};
use std::num::ParseIntError;
use std::str::FromStr;

use shared::hash::{crc64, fnv};
use shared::io::WriteExt;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlobGuid {
    pub data: [u8; 16],
}

impl BlobGuid {
    
    pub const NONE: BlobGuid = BlobGuid {
        data: [0; 16],
    };
    
    pub fn new(data: [u8; 16]) -> Self {
        Self {
            data
        }
    }

    pub fn hash(&self) -> u64 {
        crc64(self.to_string())
    }

    pub fn fnv_hash(&self) -> u32 {
        fnv(self.data)
    }

    pub fn is_none(&self) -> bool {
        self.data == [0; 16]
    }

    pub fn as_descriptor_guid(&self, type_hash: u32, part_number: u32) -> DescriptorGuid {
        DescriptorGuid {
            data: self.data,
            type_hash,
            part_number,
            reserved: 0,
        }
    }
    
    pub fn size(&self) -> u32 {
        let data: [u8; 4] = self.data[0..4].try_into().unwrap();
        u32::from_le_bytes(data)
    }
}

impl FromStr for BlobGuid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 32 {
            return Err(format!("Invalid length: got {}, expected 32", s.len()));
        }
        
        let mut data = [0; 16];

        for i in 0..16 {
            data[15 - i] = u8::from_str_radix(&s[(i * 2)..(i * 2 + 2)], 16)
                .map_err(|e: ParseIntError| format!("Failed to parse byte: {}", e))?;
        }

        Ok(BlobGuid {
            data,
        })
    }
}

impl Debug for BlobGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\" ({})", self, self.hash())
    }
}

impl Display for BlobGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in (0..16).rev() {
            write!(f, "{:0>2x}", self.data[i])?;
        }

        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DescriptorGuid {
    pub data: [u8; 16],
    pub type_hash: u32,
    pub part_number: u32,
    pub reserved: u64,
}

impl DescriptorGuid {
    pub const NONE: DescriptorGuid = DescriptorGuid {
        data: [0; 16],
        type_hash: 0,
        part_number: 0,
        reserved: 0,
    };
    
    pub fn hash(&self) -> u64 {
        crc64(self.to_string())
    }
    
    pub fn fnv_hash(&self) -> u32 {
        fnv(self.to_string())
    }

    pub fn is_none(&self) -> bool {
        self.data == [0; 16]
    }

    pub fn as_blob_guid(&self) -> BlobGuid {
        BlobGuid {
            data: self.data,
        }
    }
}

impl FromStr for DescriptorGuid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX_XXXXXXXX_N
        
        if s.len() < 47 {
            return Err(format!("Invalid length: got {}, expected at least 47", s.len()));
        }
        
        let mut data = [0; 16];
        
        data[0..4].copy_from_slice(
            &u32::from_str_radix(&s[0..8], 16)
                .map_err(|e: ParseIntError| format!("Failed to parse byte: {}", e))?
                .to_le_bytes()
        );
        
        data[4..6].copy_from_slice(
            &u16::from_str_radix(&s[9..13], 16)
                .map_err(|e: ParseIntError| format!("Failed to parse byte: {}", e))?
                .to_le_bytes()
        );
        
        data[6..8].copy_from_slice(
            &u16::from_str_radix(&s[14..18], 16)
                .map_err(|e: ParseIntError| format!("Failed to parse byte: {}", e))?
                .to_le_bytes()
        );
        
        data[8..10].copy_from_slice(
            &u16::from_str_radix(&s[19..23], 16)
                .map_err(|e: ParseIntError| format!("Failed to parse byte: {}", e))?
                .to_be_bytes()
        );
        
        for i in 0..6 {
            data[10 + i] = u8::from_str_radix(&s[24 + (i * 2)..26 + (i * 2)], 16)
                .map_err(|e: ParseIntError| format!("Failed to parse byte: {}", e))?;
        }
        
        let type_hash = u32::from_str_radix(&s[37..45], 16)
            .map_err(|e: ParseIntError| format!("Failed to parse byte: {}", e))?;
        
        let part_number = u32::from_str(&s[46..])
            .map_err(|e: ParseIntError| format!("Failed to parse byte: {}", e))?;
        
        Ok(DescriptorGuid {
            data,
            type_hash,
            part_number,
            reserved: 0
        })
    }
}

impl Debug for DescriptorGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\" ({:0X})", self, self.hash())
    }
}

impl Display for DescriptorGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let part_0 = u32::from_le_bytes([self.data[0], self.data[1], self.data[2], self.data[3]]);
        let part_1 = u16::from_le_bytes([self.data[4], self.data[5]]);
        let part_2 = u16::from_le_bytes([self.data[6], self.data[7]]);
        let part_3 = u16::from_be_bytes([self.data[8], self.data[9]]);
        let part_4 = u32::from_be_bytes([self.data[10], self.data[11], self.data[12], self.data[13]]);
        let part_5 = u16::from_be_bytes([self.data[14], self.data[15]]);

        let part_6 = self.type_hash;

        write!(f, "{:0>8x}-{:0>4x}-{:0>4x}-{:0>4x}-{:0>8x}{:0>4x}_{:0>8x}_{}", part_0, part_1, part_2, part_3, part_4, part_5, part_6, self.part_number)
    }
}

mod test {
    use std::str::FromStr;

    const GUID: &str = "40e6ba42-a397-5790-a5c9-a4151fffe1c5_647628d6_420";
    
    #[test]
    fn test_guid_string() {
        let guid = super::DescriptorGuid::from_str(GUID).unwrap();
        
        assert_eq!(guid.to_string(), GUID);
    }
}
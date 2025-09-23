use std::{
    fmt::Display,
    io::{Cursor, Seek, SeekFrom, Write},
    ops::{Deref, DerefMut},
};

use indexmap::IndexMap;
use kfc::{
    guid::Guid,
    io::WriteExt,
    reflection::{LookupKey, PrimitiveType, TypeIndex, TypeMetadata, TypeRegistry},
};

use crate::value::{Value, WriteError, WriteErrorInfo};

impl Value {
    pub fn to_bytes(
        &self,
        type_registry: &TypeRegistry,
        r#type: &TypeMetadata,
    ) -> Result<Vec<u8>, WriteError> {
        let mut cursor = Cursor::new(Vec::with_capacity(r#type.size as usize));

        self.write(type_registry, r#type, &mut cursor)?;

        Ok(cursor.into_inner())
    }

    pub fn write_into(
        &self,
        type_registry: &TypeRegistry,
        r#type: &TypeMetadata,
        dst: &mut Vec<u8>,
    ) -> Result<(), WriteError> {
        let mut cursor = Cursor::new(dst);

        self.write(type_registry, r#type, &mut cursor)?;

        Ok(())
    }

    pub fn write<W: Write + Seek>(
        &self,
        type_registry: &TypeRegistry,
        r#type: &TypeMetadata,
        writer: &mut W,
    ) -> Result<(), WriteError> {
        let mut writer = Writer::new(writer, type_registry);

        writer.add_blob_offset(r#type.size as u64);

        match self.write_internal(r#type, &mut writer, 0) {
            Ok(_) => Ok(()),
            Err(error) => Err(WriteError::new(writer.path.to_string(), error)),
        }
    }

    /// Writes the value to the writer based on the type metadata.
    ///
    /// # Layout
    ///
    /// The layout of the individual values can be found in the documentation of the respective `write_*` methods.
    /// After the value is written, it fills the remaining space with zero to ensure the total size matches [TypeMetadata::size].
    ///
    /// ### Blobs
    ///
    /// Whenever a type is written as a blob, it means that the value is written outside of the
    /// type's main structure (meaning it is not part of `TypeMetadata::size`).
    /// Each blob's position is aligned to either its own `TypeMetadata::alignment` or the
    /// alignment of the blob before, whichever is larger.
    fn write_internal<W: Write + Seek>(
        &self,
        r#type: &TypeMetadata,
        writer: &mut Writer<W>,
        base_offset: u64,
    ) -> Result<(), WriteErrorInfo> {
        writer.seek(SeekFrom::Start(base_offset))?;

        match &r#type.primitive_type {
            PrimitiveType::None => {}
            PrimitiveType::Bool => self.write_bool(writer)?,
            PrimitiveType::UInt8 => self.write_u8(writer)?,
            PrimitiveType::Bitmask8 => self.write_bitmask8(r#type, writer)?,
            PrimitiveType::SInt8 => self.write_i8(writer)?,
            PrimitiveType::UInt16 => self.write_u16(writer)?,
            PrimitiveType::Bitmask16 => self.write_bitmask16(r#type, writer)?,
            PrimitiveType::SInt16 => self.write_i16(writer)?,
            PrimitiveType::UInt32 => self.write_u32(writer)?,
            PrimitiveType::Bitmask32 => self.write_bitmask32(r#type, writer)?,
            PrimitiveType::SInt32 => self.write_i32(writer)?,
            PrimitiveType::UInt64 => self.write_u64(writer)?,
            PrimitiveType::Bitmask64 => self.write_bitmask64(r#type, writer)?,
            PrimitiveType::SInt64 => self.write_i64(writer)?,
            PrimitiveType::Float32 => self.write_f32(writer)?,
            PrimitiveType::Float64 => self.write_f64(writer)?,
            PrimitiveType::Enum => self.write_enum(r#type, writer)?,
            PrimitiveType::Typedef => self.write_typedef(r#type, writer, base_offset)?,
            PrimitiveType::Struct => self.write_struct(r#type, writer, base_offset)?,
            PrimitiveType::StaticArray => self.write_static_array(r#type, writer, base_offset)?,
            PrimitiveType::DsArray => unreachable!(),
            PrimitiveType::DsString => unreachable!(),
            PrimitiveType::DsOptional => unreachable!(),
            PrimitiveType::DsVariant => unreachable!(),
            PrimitiveType::BlobArray => self.write_blob_array(r#type, writer)?,
            PrimitiveType::BlobString => self.write_blob_string(writer)?,
            PrimitiveType::BlobOptional => self.write_blob_optional(r#type, writer)?,
            PrimitiveType::BlobVariant => self.write_blob_variant(r#type, writer)?,
            PrimitiveType::ObjectReference | PrimitiveType::Guid => self.write_guid(writer)?,
        }

        // fill remaining space with zeroes
        let end_offset = base_offset + r#type.size as u64;

        if writer.stream_position()? < end_offset {
            writer.seek(SeekFrom::Start(end_offset - 1))?;
            writer.write_u8(0)?;
        }

        Ok(())
    }

    /// Writes a boolean value to the writer.
    ///
    /// Accepts the following values:
    /// - `Bool`
    ///
    /// # Layout
    ///
    /// The layout is a single byte (`u8`), where `0` represents `false` and `1` represents `true`.
    #[inline]
    fn write_bool<W: Write + Seek>(&self, writer: &mut Writer<W>) -> Result<(), WriteErrorInfo> {
        if let Some(value) = self.as_bool() {
            writer.write_u8(value as u8)?;
            return Ok(());
        }

        Err(WriteErrorInfo::IncompatibleType {
            got: self.to_string(),
            expected: "bool".to_string(),
        })
    }

    /// Writes an 8 bit unsigned integer to the writer.
    ///
    /// Accepts the following values:
    /// - `UInt` - must be within the range of a `u8`.
    /// - `SInt` - converts the value to a `UInt` if it fits within the range of a `u8`.
    #[inline]
    fn write_u8<W: Write + Seek>(&self, writer: &mut Writer<W>) -> Result<(), WriteErrorInfo> {
        if let Some(value) = self.as_u64() {
            if value <= u8::MAX as u64 {
                writer.write_u8(value as u8)?;
                return Ok(());
            }
        }

        Err(WriteErrorInfo::IncompatibleType {
            got: self.to_string(),
            expected: "u8".to_string(),
        })
    }

    /// Writes an 8 bit signed integer to the writer.
    ///
    /// Accepts the following values:
    /// - `SInt` - must be within the range of an `i8`.
    /// - `UInt` - converts the value to an `SInt` if it fits within the range of an `i8`.
    #[inline]
    fn write_i8<W: Write + Seek>(&self, writer: &mut Writer<W>) -> Result<(), WriteErrorInfo> {
        if let Some(value) = self.as_i64() {
            if value >= i8::MIN as i64 && value <= i8::MAX as i64 {
                writer.write_i8(value as i8)?;
                return Ok(());
            }
        }

        Err(WriteErrorInfo::IncompatibleType {
            got: self.to_string(),
            expected: "i8".to_string(),
        })
    }

    /// Writes a 16 bit unsigned integer to the writer.
    ///
    /// Accepts the following values:
    /// - `UInt` - must be within the range of a `u16`.
    /// - `SInt` - converts the value to a `UInt` if it is non-negative and fits within the range of a `u16`.
    #[inline]
    fn write_u16<W: Write + Seek>(&self, writer: &mut Writer<W>) -> Result<(), WriteErrorInfo> {
        if let Some(value) = self.as_u64() {
            if value <= u16::MAX as u64 {
                writer.write_u16(value as u16)?;
                return Ok(());
            }
        }

        Err(WriteErrorInfo::IncompatibleType {
            got: self.to_string(),
            expected: "u16".to_string(),
        })
    }

    /// Writes a 16 bit signed integer to the writer.
    ///
    /// Accepts the following values:
    /// - `SInt` - must be within the range of an `i16`.
    /// - `UInt` - converts the value to an `SInt` if it fits within the positive range of an `i16`.
    #[inline]
    fn write_i16<W: Write + Seek>(&self, writer: &mut Writer<W>) -> Result<(), WriteErrorInfo> {
        if let Some(value) = self.as_i64() {
            if value >= i16::MIN as i64 && value <= i16::MAX as i64 {
                writer.write_i16(value as i16)?;
                return Ok(());
            }
        }

        Err(WriteErrorInfo::IncompatibleType {
            got: self.to_string(),
            expected: "i16".to_string(),
        })
    }

    /// Writes a 32 bit unsigned integer to the writer.
    ///
    /// Accepts the following values:
    /// - `UInt` - must be within the range of a `u32`.
    /// - `SInt` - converts the value to a `UInt` if it is non-negative.
    #[inline]
    fn write_u32<W: Write + Seek>(&self, writer: &mut Writer<W>) -> Result<(), WriteErrorInfo> {
        if let Some(value) = self.as_u64() {
            if value <= u32::MAX as u64 {
                writer.write_u32(value as u32)?;
                return Ok(());
            }
        }

        Err(WriteErrorInfo::IncompatibleType {
            got: self.to_string(),
            expected: "u32".to_string(),
        })
    }

    /// Writes a 32 bit signed integer to the writer.
    ///
    /// Accepts the following values:
    /// - `SInt` - must be within the range of an `i32`.
    /// - `UInt` - converts the value to an `SInt` if it fits within the positive range of an `i32`.
    #[inline]
    fn write_i32<W: Write + Seek>(&self, writer: &mut Writer<W>) -> Result<(), WriteErrorInfo> {
        if let Some(value) = self.as_i64() {
            if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
                writer.write_i32(value as i32)?;
                return Ok(());
            }
        }

        Err(WriteErrorInfo::IncompatibleType {
            got: self.to_string(),
            expected: "i32".to_string(),
        })
    }

    /// Writes a 64 bit unsigned integer to the writer.
    ///
    /// Accepts the following values:
    /// - `UInt`
    /// - `SInt` - converts the value to a `UInt` if the value is non-negative.
    #[inline]
    fn write_u64<W: Write + Seek>(&self, writer: &mut Writer<W>) -> Result<(), WriteErrorInfo> {
        if let Some(value) = self.as_u64() {
            writer.write_u64(value)?;
            Ok(())
        } else {
            Err(WriteErrorInfo::IncompatibleType {
                got: self.to_string(),
                expected: "u64".to_string(),
            })
        }
    }

    /// Writes a 64 bit signed integer to the writer.
    ///
    /// Accepts the following values:
    /// - `SInt`
    /// - `UInt` - converts the value to an `SInt` if it fits within the positive range of an `i64`.
    #[inline]
    fn write_i64<W: Write + Seek>(&self, writer: &mut Writer<W>) -> Result<(), WriteErrorInfo> {
        if let Some(value) = self.as_i64() {
            writer.write_i64(value)?;
            Ok(())
        } else {
            Err(WriteErrorInfo::IncompatibleType {
                got: self.to_string(),
                expected: "i64".to_string(),
            })
        }
    }

    /// Writes a 32 bit floating point number to the writer.
    ///
    /// Accepts the following values:
    /// - `Float`
    /// - `UInt` - converts the value to a floating point number.
    /// - `SInt` - converts the value to a floating point number.
    /// - `String` - parses the string as a floating point number.
    #[inline]
    fn write_f32<W: Write + Seek>(&self, writer: &mut Writer<W>) -> Result<(), WriteErrorInfo> {
        if let Some(value) = self.as_f64() {
            writer.write_f32(value as f32)?;
            Ok(())
        } else {
            Err(WriteErrorInfo::IncompatibleType {
                got: self.to_string(),
                expected: "f32".to_string(),
            })
        }
    }

    /// Writes a 64 bit floating point number to the writer.
    ///
    /// Accepts the following values:
    /// - `Float`
    /// - `UInt` - converts the value to a floating point number.
    /// - `SInt` - converts the value to a floating point number.
    /// - `String` - parses the string as a floating point number.
    #[inline]
    fn write_f64<W: Write + Seek>(&self, writer: &mut Writer<W>) -> Result<(), WriteErrorInfo> {
        if let Some(value) = self.as_f64() {
            writer.write_f64(value)?;
            Ok(())
        } else {
            Err(WriteErrorInfo::IncompatibleType {
                got: self.to_string(),
                expected: "f64".to_string(),
            })
        }
    }

    /// Writes an 8 bit bitmask to the writer.
    ///
    /// Accepts the following values:
    /// - `UInt` - truncates the higher-order bits
    /// - `SInt` - converts the value to a `UInt` if the value is non-negative.
    /// - `Array` - converts an array of enum values to a bitmask.
    ///   See [Self::write_enum] what enum values are accepted.
    ///
    /// # Layout
    ///
    /// The layout is a `u8` representing the bitmask value.
    #[inline]
    fn write_bitmask8<W: Write + Seek>(
        &self,
        r#type: &TypeMetadata,
        writer: &mut Writer<W>,
    ) -> Result<(), WriteErrorInfo> {
        let value = self.resolve_bitmask_value(writer.type_registry, r#type)?;
        writer.write_u8(value as u8)?;
        Ok(())
    }

    /// Writes a 16 bit bitmask to the writer.
    ///
    /// Accepts the following values:
    /// - `UInt` - truncates the higher-order bits
    /// - `SInt` - converts the value to a `UInt` if the value is non-negative.
    /// - `Array` - converts an array of enum values to a bitmask.
    ///   See [Self::write_enum] what enum values are accepted.
    ///
    /// # Layout
    ///
    /// The layout is a `u16` representing the bitmask value.
    #[inline]
    fn write_bitmask16<W: Write + Seek>(
        &self,
        r#type: &TypeMetadata,
        writer: &mut Writer<W>,
    ) -> Result<(), WriteErrorInfo> {
        let value = self.resolve_bitmask_value(writer.type_registry, r#type)?;
        writer.write_u16(value as u16)?;
        Ok(())
    }

    /// Writes a 32 bit bitmask to the writer.
    ///
    /// Accepts the following values:
    /// - `UInt` - truncates the higher-order bits
    /// - `SInt` - converts the value to a `UInt` if the value is non-negative.
    /// - `Array` - converts an array of enum values to a bitmask.
    ///   See [Self::write_enum] what enum values are accepted.
    ///
    /// # Layout
    ///
    /// The layout is a `u32` representing the bitmask value.
    #[inline]
    fn write_bitmask32<W: Write + Seek>(
        &self,
        r#type: &TypeMetadata,
        writer: &mut Writer<W>,
    ) -> Result<(), WriteErrorInfo> {
        let value = self.resolve_bitmask_value(writer.type_registry, r#type)?;
        writer.write_u32(value as u32)?;
        Ok(())
    }

    /// Writes a 64 bit bitmask to the writer.
    ///
    /// Accepts the following values:
    /// - `UInt`
    /// - `SInt` - converts the value to a `UInt` if the value is non-negative.
    /// - `Array` - converts an array of enum values to a bitmask.
    ///   See [Self::write_enum] what enum values are accepted.
    ///
    /// # Layout
    ///
    /// The layout is a `u64` representing the bitmask value.
    #[inline]
    fn write_bitmask64<W: Write + Seek>(
        &self,
        r#type: &TypeMetadata,
        writer: &mut Writer<W>,
    ) -> Result<(), WriteErrorInfo> {
        let value = self.resolve_bitmask_value(writer.type_registry, r#type)?;
        writer.write_u64(value)?;
        Ok(())
    }

    #[inline]
    fn resolve_bitmask_value(
        &self,
        type_registry: &TypeRegistry,
        r#type: &TypeMetadata,
    ) -> Result<u64, WriteErrorInfo> {
        if let Some(value) = self.as_u64() {
            Ok(value)
        } else if let Some(values) = self.as_array() {
            let mut bitmask = 0u64;
            let bitmask_type = type_registry
                .get_inner_type(r#type)
                .expect("invalid bitmask type");

            for value in values {
                let bit_index = value.resolve_enum_value(bitmask_type, true)?;

                if bit_index >= 64 {
                    // TODO: maybe return an error here?
                    continue;
                }

                bitmask |= 1 << bit_index;
            }

            Ok(bitmask)
        } else {
            Err(WriteErrorInfo::IncompatibleType {
                got: self.to_string(),
                expected: "bitmask".to_string(),
            })
        }
    }

    /// Writes an enum value to the writer.
    ///
    /// Accepts the following values:
    /// - `UInt` - the associated enum value
    /// - `SInt` - same as `UInt`, but only if the value is non-negative.
    /// - `String` - converts the string to the associated enum value.
    ///
    /// # Layout
    ///
    /// The layout is equivalent to the primitive type of the enum value,
    /// which is either a signed or unsigned integer type.
    #[inline]
    fn write_enum<W: Write + Seek>(
        &self,
        r#type: &TypeMetadata,
        writer: &mut Writer<W>,
    ) -> Result<(), WriteErrorInfo> {
        let value = self.resolve_enum_value(r#type, false)?;
        let enum_value_type = writer
            .type_registry
            .get_inner_type(r#type)
            .expect("invalid enum type");

        match enum_value_type.primitive_type {
            PrimitiveType::SInt8 => writer.write_i8(value as i8)?,
            PrimitiveType::SInt16 => writer.write_i16(value as i16)?,
            PrimitiveType::SInt32 => writer.write_i32(value as i32)?,
            PrimitiveType::SInt64 => writer.write_i64(value as i64)?,
            PrimitiveType::UInt8 => writer.write_u8(value as u8)?,
            PrimitiveType::UInt16 => writer.write_u16(value as u16)?,
            PrimitiveType::UInt32 => writer.write_u32(value as u32)?,
            PrimitiveType::UInt64 => writer.write_u64(value)?,
            _ => panic!("Unsupported enum value type: {enum_value_type:?}"),
        }

        Ok(())
    }

    fn resolve_enum_value(
        &self,
        r#type: &TypeMetadata,
        allow_unknown_integers: bool,
    ) -> Result<u64, WriteErrorInfo> {
        if let Some(value) = self.as_u64() {
            if allow_unknown_integers || r#type.enum_fields.iter().any(|(_, v)| v.value == value) {
                Ok(value)
            } else {
                Err(WriteErrorInfo::InvalidEnumValue {
                    got: value.to_string(),
                    expected: r#type
                        .enum_fields
                        .iter()
                        .map(|(_, v)| v.name.clone())
                        .chain(r#type.enum_fields.iter().map(|(_, v)| v.value.to_string()))
                        .collect(),
                })
            }
        } else if let Some(value) = self.as_string() {
            if let Some(enum_value) = r#type.enum_fields.get(value) {
                Ok(enum_value.value)
            } else {
                Err(WriteErrorInfo::InvalidEnumValue {
                    got: value.clone(),
                    expected: r#type
                        .enum_fields
                        .iter()
                        .map(|(_, v)| v.name.clone())
                        .chain(r#type.enum_fields.iter().map(|(_, v)| v.value.to_string()))
                        .collect(),
                })
            }
        } else {
            Err(WriteErrorInfo::IncompatibleType {
                got: self.to_string(),
                expected: "enum".to_string(),
            })
        }
    }

    /// Writes a typedef to the writer.
    /// This function purely resolves the inner type and calls [Self::write_internal] with it.
    #[inline]
    fn write_typedef<W: Write + Seek>(
        &self,
        r#type: &TypeMetadata,
        writer: &mut Writer<W>,
        base_offset: u64,
    ) -> Result<(), WriteErrorInfo> {
        let inner_type = writer
            .type_registry
            .get_inner_type(r#type)
            .expect("invalid typedef type");

        self.write_internal(inner_type, writer, base_offset)
    }

    /// Writes a struct to the writer.
    ///
    /// Accepts the following values:
    /// - `Struct` - writes the struct fields including all parent fields.
    ///
    /// # Layout
    ///
    /// See [Self::write_struct_fields] for the layout of the struct fields.
    #[inline]
    fn write_struct<W: Write + Seek>(
        &self,
        r#type: &TypeMetadata,
        writer: &mut Writer<W>,
        base_offset: u64,
    ) -> Result<(), WriteErrorInfo> {
        if let Self::Struct(fields) = self {
            Self::write_struct_fields(fields, r#type, writer, base_offset)?;

            return Ok(());
        }

        Err(WriteErrorInfo::IncompatibleType {
            got: self.to_string(),
            expected: "struct".to_string(),
        })
    }

    /// Writes the entries of the given map to the writer with the given type.
    /// Each entry is considered a field of a struct and is written according to its metadata.
    ///
    /// # Layout
    ///
    /// Each field value is written at the given [StructFieldMetadata::data_offset](kfc::reflection::StructFieldMetadata::data_offset).
    #[inline]
    fn write_struct_fields<W: Write + Seek>(
        fields: &IndexMap<String, Self>,
        r#type: &TypeMetadata,
        writer: &mut Writer<W>,
        base_offset: u64,
    ) -> Result<(), WriteErrorInfo> {
        if let Some(parent_type) = writer.type_registry.get_inner_type(r#type) {
            Self::write_struct_fields(fields, parent_type, writer, base_offset)?;
        }

        for field in r#type.struct_fields.values() {
            let field_value = fields
                .get(&field.name)
                .ok_or_else(|| WriteErrorInfo::MissingField(field.name.clone()))?;

            let field_type = writer
                .type_registry
                .get(field.r#type)
                .expect("invalid field type");

            writer.path.push(&field.name);
            field_value.write_internal(field_type, writer, base_offset + field.data_offset)?;
            writer.path.pop();
        }

        Ok(())
    }

    /// Writes a static array to the writer.
    ///
    /// Accepts the following values:
    /// - `Array` - writes the array as a static array.
    ///
    /// # Layout
    ///
    /// The layout is equivalent to `[T; N]`, where `T` is the element type and `N` is the number of elements.
    /// The number of elements `N` is determined by [TypeMetadata::field_count].
    #[inline]
    fn write_static_array<W: Write + Seek>(
        &self,
        r#type: &TypeMetadata,
        writer: &mut Writer<W>,
        base_offset: u64,
    ) -> Result<(), WriteErrorInfo> {
        let element_type = writer
            .type_registry
            .get_inner_type(r#type)
            .expect("invalid static array type");

        if let Self::Array(values) = self {
            if values.len() != r#type.field_count as usize {
                return Err(WriteErrorInfo::IncompatibleType {
                    got: format!("array of length {}", values.len()),
                    expected: format!("static array of length {}", r#type.field_count),
                });
            }

            for (i, value) in values.iter().enumerate() {
                let offset = (i * element_type.size as usize) as u64;

                writer.path.push_index(i);
                value.write_internal(element_type, writer, base_offset + offset)?;
                writer.path.pop();
            }

            return Ok(());
        }

        Err(WriteErrorInfo::IncompatibleType {
            got: self.to_string(),
            expected: "array".to_string(),
        })
    }

    /// Writes a blob array to the writer.
    ///
    /// Accepts the following values:
    /// - `Array` - writes the array as a blob.
    ///
    /// # Layout
    ///
    /// ```no_run
    /// struct BlobArray {
    ///    offset: u32, // relative offset to the array data
    ///    length: u32, // length of the array (in elements)
    /// }
    /// ```
    #[inline]
    fn write_blob_array<W: Write + Seek>(
        &self,
        r#type: &TypeMetadata,
        writer: &mut Writer<W>,
    ) -> Result<(), WriteErrorInfo> {
        let element_type = writer
            .type_registry
            .get_inner_type(r#type)
            .expect("invalid blob array type");

        if let Self::Array(values) = self {
            let blob_size = values.len() as u64 * element_type.size as u64;
            let alignment = element_type.alignment as u64;

            if values.is_empty() {
                writer.write_u32(0)?;
                writer.write_u32(0)?;

                return Ok(());
            }

            let mut offset = writer.write_blob_header(
                blob_size,
                alignment,
                Some(values.len() as u32)
            )?;

            for (i, value) in values.iter().enumerate() {
                writer.path.push_index(i);
                value.write_internal(element_type, writer, offset)?;
                writer.path.pop();

                offset += element_type.size as u64;
            }

            return Ok(());
        }

        Err(WriteErrorInfo::IncompatibleType {
            got: self.to_string(),
            expected: "array".to_string(),
        })
    }

    /// Writes a string to the writer.
    ///
    /// Accepts the following values:
    /// - `None` - writes a zero string.
    /// - `String` - writes the string as a blob.
    ///
    /// **Note:** If the string is empty, it will do the same as `None`.
    ///
    /// # Layout
    ///
    /// ```no_run
    /// struct BlobString {
    ///     offset: u32, // relative offset to the string data
    ///     length: u32, // length of the string (in bytes)
    /// }
    /// ```
    #[inline]
    fn write_blob_string<W: Write + Seek>(
        &self,
        writer: &mut Writer<W>,
    ) -> Result<(), WriteErrorInfo> {
        if self.is_none() || self.as_string().map(String::is_empty).unwrap_or(false) {
            writer.write_u32(0)?;
            writer.write_u32(0)?;

            return Ok(());
        }

        if let Some(value) = self.as_string() {
            let offset = writer.write_blob_header(
                value.len() as u64,
                1,
                Some(value.len() as u32)
            )?;

            writer.seek(SeekFrom::Start(offset))?;
            writer.write_all(value.as_bytes())?;

            return Ok(());
        }

        Err(WriteErrorInfo::IncompatibleType {
            got: self.to_string(),
            expected: "string".to_string(),
        })
    }

    /// Writes an optional value to the writer.
    ///
    /// Accepts the following values:
    /// - `None` - writes a zero optional/null
    /// - otherwise, writes the inner value as a blob.
    ///
    /// # Layout
    ///
    /// ```no_run
    /// struct BlobOptional {
    ///     offset: u32, // relative offset to the inner value
    /// }
    /// ```
    #[inline]
    fn write_blob_optional<W: Write + Seek>(
        &self,
        r#type: &TypeMetadata,
        writer: &mut Writer<W>,
    ) -> Result<(), WriteErrorInfo> {
        let inner_type = writer
            .type_registry
            .get_inner_type(r#type)
            .expect("invalid optional type");

        if self.is_none() {
            writer.write_u32(0)?;
            return Ok(());
        }

        let offset = writer.write_blob_header(
            inner_type.size as u64,
            inner_type.alignment as u64,
            None
        )?;

        self.write_internal(inner_type, writer, offset)
    }

    /// Writes a variant to the writer.
    ///
    /// Accepts the following values:
    /// - `None` - writes a zero variant.
    /// - `Variant` - writes the variant data as a blob.
    ///
    /// # Layout
    ///
    /// ```no_run
    /// struct BlobVariant {
    ///     qualified_hash: u32, // hash of the variant type
    ///     offset: u32, // relative offset to the variant data
    ///     size: u32, // size of the variant data (size of the variant type)
    /// }
    /// ```
    #[inline]
    fn write_blob_variant<W: Write + Seek>(
        &self,
        r#type: &TypeMetadata,
        writer: &mut Writer<W>,
    ) -> Result<(), WriteErrorInfo> {
        if self.is_none() {
            writer.write_u32(0)?;
            writer.write_u32(0)?;
            writer.write_u32(0)?;

            return Ok(());
        }

        let type_registry = writer.type_registry;
        let base_type = type_registry
            .get_inner_type(r#type)
            .expect("invalid variant type");

        let (variant_value, variant_type) = if let Some(variant) = self.as_variant() {
            let variant_type = type_registry
                .get(variant.type_index)
                .ok_or(WriteErrorInfo::InvalidType(variant.type_index))?;

            if variant_type == base_type && variant_type.field_count == 0 {
                writer.write_u32(0)?;
                writer.write_u32(0)?;
                writer.write_u32(0)?;

                return Ok(());
            }

            (&variant.value, variant_type)
        } else if let Some(value) = self.as_struct() {
            // Format: { $type: "qualified_name", $value: ... }

            // parse $type

            writer.path.push("$type");

            let variant_type = value
                .get("$type")
                .map(|v| {
                    if let Some(value) = v.as_string() {
                        type_registry
                            .get_by_name(LookupKey::Qualified(value))
                            .ok_or_else(|| WriteErrorInfo::InvalidTypeName(value.clone()))
                    } else if let Some(v) = v.as_u64() {
                        let raw_index = TypeIndex::new(v as usize);

                        type_registry
                            .get(raw_index)
                            .ok_or(WriteErrorInfo::InvalidType(raw_index))
                    } else {
                        Err(WriteErrorInfo::IncompatibleType {
                            got: v.to_string(),
                            expected: "string".to_string(),
                        })
                    }
                })
                .transpose()?
                .ok_or_else(|| WriteErrorInfo::MissingField("$type".to_string()))?;

            writer.path.pop();

            if variant_type == base_type && variant_type.field_count == 0 {
                writer.write_u32(0)?;
                writer.write_u32(0)?;
                writer.write_u32(0)?;

                return Ok(());
            }

            // parse $value

            writer.path.push("$value");

            let variant_value = value
                .get("$value")
                .map(|v| {
                    v.as_struct().ok_or_else(|| WriteErrorInfo::IncompatibleType {
                        got: v.to_string(),
                        expected: "struct".to_string(),
                    })
                })
                .transpose()?
                .ok_or_else(|| WriteErrorInfo::MissingField("$value".to_string()))?;

            writer.path.pop();

            (variant_value, variant_type)
        } else {
            return Err(WriteErrorInfo::IncompatibleType {
                got: self.to_string(),
                expected: "variant".to_string(),
            });
        };

        let parent_type = type_registry
            .get_inner_type(r#type)
            .expect("invalid variant type");

        if !type_registry.is_sub_type(parent_type, variant_type) {
            return Err(WriteErrorInfo::VariantTypeNotSubType(
                variant_type.qualified_name.clone(),
                r#type.qualified_name.clone(),
            ));
        }

        writer.write_u32(variant_type.qualified_hash)?;

        let offset = writer.write_blob_header(
            variant_type.size as u64,
            variant_type.alignment as u64,
            Some(variant_type.size),
        )?;

        if self.is_struct() {
            writer.path.push("$value");
        }

        Self::write_struct_fields(variant_value, variant_type, writer, offset)?;

        if self.is_struct() {
            writer.path.pop();
        }

        // fill remaining space with zeroes
        let end_offset = offset + variant_type.size as u64;

        if writer.stream_position()? < end_offset {
            writer.seek(SeekFrom::Start(end_offset - 1))?;
            writer.write_u8(0)?;
        }

        Ok(())
    }

    /// Writes a GUID to the writer.
    ///
    /// Accepts the following values:
    /// - `None` - writes a zero GUID.
    /// - `Guid` - writes the GUID directly.
    /// - `String` - parses the string into a `Guid` and writes it.
    ///
    /// # Layout
    /// See [Guid::write].
    #[inline]
    fn write_guid<W: Write + Seek>(&self, writer: &mut Writer<W>) -> Result<(), WriteErrorInfo> {
        if self.is_none() {
            Guid::NONE.write(writer.deref_mut())?;
            return Ok(());
        }

        if let Some(value) = self.as_guid() {
            value.write(writer.deref_mut())?;
            return Ok(());
        } else if let Some(value) = self.as_string() {
            Guid::parse(value)
                .ok_or_else(|| WriteErrorInfo::MalformedGuid(value.clone()))?
                .write(writer.deref_mut())?;

            return Ok(());
        }

        Err(WriteErrorInfo::IncompatibleType {
            got: self.to_string(),
            expected: "guid".to_string(),
        })
    }
}

struct Writer<'a, W> {
    writer: W,
    path: TreePath,
    blob_offset: u64,
    type_registry: &'a TypeRegistry,
}

impl<'a, W: Write + Seek> Writer<'a, W> {
    #[inline]
    fn new(writer: W, type_registry: &'a TypeRegistry) -> Self {
        Self {
            writer,
            path: TreePath::new(),
            blob_offset: 0,
            type_registry,
        }
    }

    #[inline]
    fn offset(&mut self) -> std::io::Result<u64> {
        self.writer.stream_position()
    }

    #[inline]
    fn blob_offset(&self) -> u64 {
        self.blob_offset
    }

    #[inline]
    fn align_blob_offset(&mut self, alignment: u64) {
        self.blob_offset += (alignment - (self.blob_offset % alignment)) % alignment;
    }

    #[inline]
    fn add_blob_offset(&mut self, offset: u64) {
        self.blob_offset += offset;
    }

    #[inline]
    fn write_blob_header(
        &mut self,
        size: u64,
        alignment: u64,
        count: Option<u32>,
    ) -> std::io::Result<u64> {
        let base_offset = self.offset()?;

        self.align_blob_offset(alignment);

        let blob_offset = self.blob_offset();
        self.write_u32((blob_offset - base_offset) as u32)?;

        if let Some(count) = count {
            self.write_u32(count)?;
        }

        let offset = self.blob_offset();

        self.add_blob_offset(size);
        self.align_blob_offset(alignment);

        Ok(offset)
    }
}

impl<W: Write + Seek> Deref for Writer<'_, W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl<W: Write + Seek> DerefMut for Writer<'_, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

struct TreePath {
    stack: Vec<String>,
    len: usize,
}

impl TreePath {
    #[inline]
    fn new() -> Self {
        Self {
            stack: vec![String::with_capacity(32); 16],
            len: 0,
        }
    }

    fn push(&mut self, name: &str) {
        if self.len == self.stack.len() {
            self.stack.push(String::with_capacity(32));
        }

        self.stack[self.len].clear();
        self.stack[self.len].push_str(name);
        self.len += 1;
    }

    fn push_index(&mut self, index: usize) {
        use std::fmt::Write;

        if self.len == self.stack.len() {
            self.stack.push(String::with_capacity(32));
        }

        self.stack[self.len].clear();
        write!(self.stack[self.len], "{index}").unwrap();
        self.len += 1;
    }

    fn pop(&mut self) {
        if self.len > 0 {
            self.len -= 1;
        }
    }
}

impl Display for TreePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.len == 0 {
            write!(f, ".")
        } else {
            write!(f, "{}", self.stack[..self.len].join("."))
        }
    }
}

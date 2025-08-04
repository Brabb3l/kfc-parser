use std::{borrow::Borrow, ops::Deref};

use kfc::{guid::BlobGuid, reflection::{EnumFieldMetadata, LookupKey, PrimitiveType, StructFieldMetadata, TypeMetadata, TypeRegistry}};

mod error;
mod util;
mod type_handle;

pub use error::*;
pub use type_handle::*;
use util::*;

#[derive(Debug, Clone)]
pub enum MappedValue<D, T> {
    None,
    Bool(bool),
    UInt8(u8),
    SInt8(i8),
    UInt16(u16),
    SInt16(i16),
    UInt32(u32),
    SInt32(i32),
    UInt64(u64),
    SInt64(i64),
    Float32(f32),
    Float64(f64),
    Enum(MappedEnum<T>),
    Bitmask(MappedBitmask<T>),
    Struct(MappedStruct<D, T>),
    Array(MappedArray<D, T>),
    String(MappedString<D>),
    Optional(MappedOptional<D, T>),
    Variant(MappedVariant<D, T>),
    Reference(MappedReference<T>),
    Guid(BlobGuid),
}

impl<D, T> MappedValue<D, T>
where
    D: Borrow<[u8]> + Clone,
    T: Borrow<TypeRegistry> + Clone,
{
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    #[inline]
    pub fn as_none(&self) -> Option<()> {
        match self {
            Self::None => Some(()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_u8(&self) -> bool {
        matches!(self, Self::UInt8(_))
    }

    #[inline]
    pub fn as_u8(&self) -> Option<u8> {
        if let Self::UInt8(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_i8(&self) -> bool {
        matches!(self, Self::SInt8(_))
    }

    #[inline]
    pub fn as_i8(&self) -> Option<i8> {
        if let Self::SInt8(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_u16(&self) -> bool {
        matches!(self, Self::UInt16(_))
    }

    #[inline]
    pub fn as_u16(&self) -> Option<u16> {
        if let Self::UInt16(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_i16(&self) -> bool {
        matches!(self, Self::SInt16(_))
    }

    #[inline]
    pub fn as_i16(&self) -> Option<i16> {
        if let Self::SInt16(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_u32(&self) -> bool {
        matches!(self, Self::UInt32(_))
    }

    #[inline]
    pub fn as_u32(&self) -> Option<u32> {
        if let Self::UInt32(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_i32(&self) -> bool {
        matches!(self, Self::SInt32(_))
    }

    #[inline]
    pub fn as_i32(&self) -> Option<i32> {
        if let Self::SInt32(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_u64(&self) -> bool {
        matches!(self, Self::UInt64(_))
    }

    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        if let Self::UInt64(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_i64(&self) -> bool {
        matches!(self, Self::SInt64(_))
    }

    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        if let Self::SInt64(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_f32(&self) -> bool {
        matches!(self, Self::Float32(_))
    }

    #[inline]
    pub fn as_f32(&self) -> Option<f32> {
        if let Self::Float32(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_f64(&self) -> bool {
        matches!(self, Self::Float64(_))
    }

    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        if let Self::Float64(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_enum(&self) -> bool {
        matches!(self, Self::Enum(_))
    }

    #[inline]
    pub fn as_enum(&self) -> Option<&MappedEnum<T>> {
        if let Self::Enum(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_bitmask(&self) -> bool {
        matches!(self, Self::Bitmask(_))
    }

    #[inline]
    pub fn as_bitmask(&self) -> Option<&MappedBitmask<T>> {
        if let Self::Bitmask(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Struct(_))
    }

    #[inline]
    pub fn as_struct(&self) -> Option<&MappedStruct<D, T>> {
        if let Self::Struct(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    #[inline]
    pub fn as_array(&self) -> Option<&MappedArray<D, T>> {
        if let Self::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    #[inline]
    pub fn as_string(&self) -> Option<&MappedString<D>> {
        if let Self::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_optional(&self) -> bool {
        matches!(self, Self::Optional(_))
    }

    #[inline]
    pub fn as_optional(&self) -> Option<&MappedOptional<D, T>> {
        if let Self::Optional(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_variant(&self) -> bool {
        matches!(self, Self::Variant(_))
    }

    #[inline]
    pub fn as_variant(&self) -> Option<&MappedVariant<D, T>> {
        if let Self::Variant(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_reference(&self) -> bool {
        matches!(self, Self::Reference(_))
    }

    #[inline]
    pub fn as_reference(&self) -> Option<&MappedReference<T>> {
        if let Self::Reference(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_guid(&self) -> bool {
        matches!(self, Self::Guid(_))
    }

    #[inline]
    pub fn as_guid(&self) -> Option<&BlobGuid> {
        if let Self::Guid(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

impl<D, T> MappedValue<D, T>
where
    D: Borrow<[u8]> + Clone,
    T: Borrow<TypeRegistry> + Clone,
{
    #[inline]
    pub fn from_bytes(
        type_registry: &T,
        r#type: &TypeMetadata,
        data: &D,
    ) -> Result<Self, MappingError> {
        let r#type = TypeHandle::try_new(type_registry.clone(), r#type.index)
            .ok_or(MappingError::InvalidTypeIndex(r#type.index))?;

        Self::from_impl(&r#type, data, 0)
    }

    fn from_impl(
        r#type: &TypeHandle<T>,
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        match r#type.primitive_type {
            PrimitiveType::None => Ok(Self::None),
            PrimitiveType::Bool => Ok(get_bool(data.borrow(), offset)?.into()),
            PrimitiveType::UInt8 => Ok(get_u8(data.borrow(), offset)?.into()),
            PrimitiveType::SInt8 => Ok(get_i8(data.borrow(), offset)?.into()),
            PrimitiveType::UInt16 => Ok(get_u16(data.borrow(), offset)?.into()),
            PrimitiveType::SInt16 => Ok(get_i16(data.borrow(), offset)?.into()),
            PrimitiveType::UInt32 => Ok(get_u32(data.borrow(), offset)?.into()),
            PrimitiveType::SInt32 => Ok(get_i32(data.borrow(), offset)?.into()),
            PrimitiveType::UInt64 => Ok(get_u64(data.borrow(), offset)?.into()),
            PrimitiveType::SInt64 => Ok(get_i64(data.borrow(), offset)?.into()),
            PrimitiveType::Float32 => Ok(get_f32(data.borrow(), offset)?.into()),
            PrimitiveType::Float64 => Ok(get_f64(data.borrow(), offset)?.into()),
            PrimitiveType::Enum => Self::from_enum(r#type, data, offset),
            PrimitiveType::Bitmask8 => Self::from_bitmask8(r#type, data, offset),
            PrimitiveType::Bitmask16 => Self::from_bitmask16(r#type, data, offset),
            PrimitiveType::Bitmask32 => Self::from_bitmask32(r#type, data, offset),
            PrimitiveType::Bitmask64 => Self::from_bitmask64(r#type, data, offset),
            PrimitiveType::Typedef => Self::from_typedef(r#type, data, offset),
            PrimitiveType::Struct => Self::from_struct(r#type, data, offset),
            PrimitiveType::StaticArray => Self::from_static_array(r#type, data, offset),
            PrimitiveType::DsArray => Err(MappingError::UnsupportedOperation("DsArrays are not supported yet")),
            PrimitiveType::DsString => Err(MappingError::UnsupportedOperation("DsStrings are not supported yet")),
            PrimitiveType::DsOptional => Err(MappingError::UnsupportedOperation("DsOptionals are not supported yet")),
            PrimitiveType::DsVariant => Err(MappingError::UnsupportedOperation("DsVariants are not supported yet")),
            PrimitiveType::BlobArray => Self::from_blob_array(r#type, data, offset),
            PrimitiveType::BlobString => Self::from_blob_string(data, offset),
            PrimitiveType::BlobOptional => Self::from_blob_optional(r#type, data, offset),
            PrimitiveType::BlobVariant => Self::from_blob_variant(r#type, data, offset),
            PrimitiveType::ObjectReference => Self::from_object_reference(r#type, data, offset),
            PrimitiveType::Guid => Self::from_guid(data, offset),
        }
    }

    #[inline]
    fn from_enum(
        r#type: &TypeHandle<T>,
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        let enum_value_type = get_inner_type(r#type);
        let enum_value = match enum_value_type.primitive_type {
            PrimitiveType::UInt8 => get_u8(data.borrow(), offset)? as u64,
            PrimitiveType::SInt8 => get_i8(data.borrow(), offset)? as u64,
            PrimitiveType::UInt16 => get_u16(data.borrow(), offset)? as u64,
            PrimitiveType::SInt16 => get_i16(data.borrow(), offset)? as u64,
            PrimitiveType::UInt32 => get_u32(data.borrow(), offset)? as u64,
            PrimitiveType::SInt32 => get_i32(data.borrow(), offset)? as u64,
            PrimitiveType::UInt64 => get_u64(data.borrow(), offset)?,
            PrimitiveType::SInt64 => get_i64(data.borrow(), offset)? as u64,
            _ => panic!("Invalid enum value type: {:?}", enum_value_type.primitive_type),
        };

        Ok(MappedEnum::new(r#type.clone(), enum_value).into())
    }

    #[inline]
    fn from_bitmask8(
        r#type: &TypeHandle<T>,
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        let value = get_u8(data.borrow(), offset)?;
        Ok(MappedBitmask::new(r#type.clone(), value as u64).into())
    }

    #[inline]
    fn from_bitmask16(
        r#type: &TypeHandle<T>,
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        let value = get_u16(data.borrow(), offset)?;
        Ok(MappedBitmask::new(r#type.clone(), value as u64).into())
    }

    #[inline]
    fn from_bitmask32(
        r#type: &TypeHandle<T>,
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        let value = get_u32(data.borrow(), offset)?;
        Ok(MappedBitmask::new(r#type.clone(), value as u64).into())
    }

    #[inline]
    fn from_bitmask64(
        r#type: &TypeHandle<T>,
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        let value = get_u64(data.borrow(), offset)?;
        Ok(MappedBitmask::new(r#type.clone(), value).into())
    }

    #[inline]
    fn from_typedef(
        r#type: &TypeHandle<T>,
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        let inner_type = get_inner_type(r#type);
        Self::from_impl(&inner_type, data, offset)
    }

    #[inline]
    fn from_struct(
        r#type: &TypeHandle<T>,
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        Ok(MappedStruct::new(
            r#type.clone(),
            data.clone(),
            offset
        ).into())
    }

    #[inline]
    fn from_static_array(
        r#type: &TypeHandle<T>,
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        Ok(MappedArray::new(
            r#type.clone(),
            data.clone(),
            offset,
            r#type.field_count as usize
        ).into())
    }

    #[inline]
    fn from_blob_array(
        r#type: &TypeHandle<T>,
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        let blob_offset = get_u32(data.borrow(), offset)? as usize;
        let count = get_u32(data.borrow(), offset + 4)? as usize;

        Ok(MappedArray::new(
            r#type.clone(),
            data.clone(),
            offset + blob_offset,
            count
        ).into())
    }

    #[inline]
    fn from_blob_string(
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        let blob_offset = get_u32(data.borrow(), offset)? as usize;

        if blob_offset == 0 {
            return Ok(MappedString::new(
                data.clone(),
                0,
                0
            )?.into());
        }

        let blob_size = get_u32(data.borrow(), offset + 4)? as usize;

        Ok(MappedString::new(
            data.clone(),
            offset + blob_offset,
            blob_size,
        )?.into())
    }

    #[inline]
    fn from_blob_optional(
        r#type: &TypeHandle<T>,
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        let inner_type = match get_inner_type_opt(r#type) {
            Some(t) => t,
            None => return Ok(MappedOptional::new(
                r#type.clone(),
                None
            ).into()),
        };
        let blob_offset = get_u32(data.borrow(), offset)? as usize;

        if blob_offset == 0 {
            return Ok(MappedOptional::new(
                r#type.clone(),
                None
            ).into());
        }

        Ok(MappedOptional::new(
            r#type.clone(),
            Some(Self::from_impl(
                &inner_type,
                data,
                offset + blob_offset
            )?)
        ).into())
    }

    #[inline]
    fn from_blob_variant(
        r#type: &TypeHandle<T>,
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        let variant_hash = get_u32(data.borrow(), offset)?;
        let blob_offset = get_u32(data.borrow(), offset + 4)? as usize;
        // let size = get_u32(data, 8)? as usize;

        if blob_offset == 0 {
            return Ok(MappedVariant::new(
                r#type.clone(),
                None
            ).into());
        }

        let variant_type = r#type.type_registry().borrow()
            .get_by_hash(LookupKey::Qualified(variant_hash))
            .map(|t| t.index)
            .ok_or(MappingError::InvalidTypeHash(variant_hash))?;
        let variant_type = TypeHandle::new(
            r#type.type_registry().clone(),
            variant_type
        );

        Ok(MappedVariant::new(
            r#type.clone(),
            Some(MappedStruct::new(
                variant_type,
                data.clone(),
                offset + blob_offset + 4
            )),
        ).into())
    }

    #[inline]
    fn from_object_reference(
        r#type: &TypeHandle<T>,
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        let data = get_bytes(data.borrow(), offset, 16)?.try_into().unwrap();

        Ok(MappedReference::new(
            r#type.clone(),
            BlobGuid::new(data),
        ).into())
    }

    #[inline]
    fn from_guid(
        data: &D,
        offset: usize,
    ) -> Result<Self, MappingError> {
        let bytes = get_bytes(data.borrow(), offset, 16)?;
        let guid = BlobGuid::new(bytes.try_into().unwrap());

        Ok(guid.into())
    }

}

#[derive(Debug, Clone)]
pub struct MappedString<D> {
    data: D,
    offset: usize,
    length: usize,
}

impl<D> MappedString<D>
where
    D: Borrow<[u8]> + Clone,
{
    #[inline]
    fn new(data: D, offset: usize, length: usize) -> Result<Self, MappingError> {
        if offset + length > data.borrow().len() {
            return Err(MappingError::UnexpectedEndOfData);
        }

        Ok(Self { data, offset, length })
    }

    #[inline]
    pub fn as_str(&self) -> Result<&str, MappingError> {
        if self.offset == 0 {
            return Ok("");
        }

        let bytes = get_bytes(self.data.borrow(), self.offset, self.length)?;
        std::str::from_utf8(bytes).map_err(MappingError::Utf8)
    }
}

#[derive(Debug, Clone)]
pub struct MappedEnum<T> {
    r#type: TypeHandle<T>,
    value: u64,
}

impl<T> MappedEnum<T>
where
    T: Borrow<TypeRegistry> + Clone,
{
    #[inline]
    fn new(r#type: TypeHandle<T>, value: u64) -> Self {
        Self { r#type, value }
    }

    #[inline]
    pub fn r#type(&self) -> &TypeHandle<T> {
        &self.r#type
    }

    #[inline]
    pub fn value(&self) -> u64 {
        self.value
    }

    #[inline]
    pub fn name(&self) -> Option<&str> {
        self.r#type.enum_fields.values()
            .find(|f| f.value == self.value)
            .map(|f| f.name.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct MappedBitmask<T> {
    r#type: TypeHandle<T>,
    bit_type: TypeHandle<T>,
    value: u64,
}

impl<T> MappedBitmask<T>
where
    T: Borrow<TypeRegistry> + Clone,
{
    #[inline]
    fn new(r#type: TypeHandle<T>, value: u64) -> Self {
        let bit_type = get_inner_type(&r#type);

        Self {
            r#type,
            bit_type,
            value,
        }
    }

    #[inline]
    pub fn r#type(&self) -> &TypeHandle<T> {
        &self.r#type
    }

    #[inline]
    pub fn bit_type(&self) -> &TypeHandle<T> {
        &self.bit_type
    }

    #[inline]
    pub fn value(&self) -> u64 {
        self.value
    }

    #[inline]
    pub fn bit_count(&self) -> u32 {
        match self.r#type.primitive_type {
            PrimitiveType::Bitmask8 => 8,
            PrimitiveType::Bitmask16 => 16,
            PrimitiveType::Bitmask32 => 32,
            PrimitiveType::Bitmask64 => 64,
            _ => panic!("Invalid bitmask type: {:?}", self.bit_type.primitive_type),
        }
    }

    pub fn iter(&self) -> BitIter<'_> {
        BitIter::new(self)
    }

}

pub struct BitIter<'a> {
    value: u64,
    enum_fields: indexmap::map::Values<'a, String, EnumFieldMetadata>,

    bits_left: u32,
    checked_bits: u64,
    next_unnamed: u64,
    check_unnamed: bool,
}

impl<'a> BitIter<'a> {

    pub fn new<T>(
        bitmask: &'a MappedBitmask<T>
    ) -> Self
    where
        T: Borrow<TypeRegistry> + Clone,
    {
        BitIter {
            value: bitmask.value,
            enum_fields: bitmask.bit_type.enum_fields.values(),

            bits_left: bitmask.value.count_ones(),
            checked_bits: 0,
            next_unnamed: 0,
            check_unnamed: false,
        }
    }

}

impl<'a> Iterator for BitIter<'a> {
    type Item = MappedBit<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.check_unnamed {
            for enum_field in self.enum_fields.by_ref() {
                let enum_value = enum_field.value;
                let enum_name = &enum_field.name;

                self.checked_bits |= 1 << enum_value;

                if self.value & (1 << enum_value) != 0 {
                    self.bits_left -= 1;
                    return Some(MappedBit::new(Some(enum_name), enum_value));
                }
            }

            self.check_unnamed = true;
        }

        while self.bits_left > 0 {
            let i = self.next_unnamed;
            self.next_unnamed += 1;

            if self.checked_bits & (1 << i) == 0 && self.value & (1 << i) != 0 {
                self.bits_left -= 1;
                return Some(MappedBit::new(None, i));
            }
        }

        None
    }
}

pub struct MappedBit<'a> {
    name: Option<&'a str>,
    value: u64,
}

impl<'a> MappedBit<'a> {
    #[inline]
    pub fn new(
        name: Option<&'a str>,
        value: u64,
    ) -> Self {
        Self { name, value }
    }

    #[inline]
    pub fn name(&self) -> Option<&str> {
        self.name
    }

    #[inline]
    pub fn value(&self) -> u64 {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct MappedOptional<D, T> {
    r#type: TypeHandle<T>,
    value: Option<Box<MappedValue<D, T>>>,
}

impl<D, T> MappedOptional<D, T>
where
    D: Borrow<[u8]> + Clone,
    T: Borrow<TypeRegistry> + Clone,
{
    #[inline]
    pub fn new(r#type: TypeHandle<T>, value: Option<MappedValue<D, T>>) -> Self {
        Self {
            r#type,
            value: value.map(Box::new)
        }
    }

    #[inline]
    pub fn r#type(&self) -> &TypeHandle<T> {
        &self.r#type
    }

    #[inline]
    pub fn value(&self) -> Option<&MappedValue<D, T>> {
        self.value.as_deref()
    }

    #[inline]
    pub fn into_value(self) -> Option<MappedValue<D, T>> {
        self.value.map(|v| *v)
    }
}

#[derive(Debug, Clone)]
pub struct MappedVariant<D, T> {
    base_type: TypeHandle<T>,
    value: Option<MappedStruct<D, T>>,
}

impl<D, T> MappedVariant<D, T>
where
    D: Borrow<[u8]> + Clone,
    T: Borrow<TypeRegistry> + Clone,
{
    #[inline]
    fn new(base_type: TypeHandle<T>, value: Option<MappedStruct<D, T>>) -> Self {
        Self { base_type, value }
    }

    #[inline]
    pub fn base_type(&self) -> &TypeHandle<T> {
        &self.base_type
    }

    #[inline]
    pub fn variant_type(&self) -> Option<&TypeHandle<T>> {
        self.value.as_ref().map(|v| &v.r#type)
    }

    #[inline]
    pub fn value(&self) -> Option<&MappedStruct<D, T>> {
        self.value.as_ref()
    }

    #[inline]
    pub fn into_value(self) -> Option<MappedStruct<D, T>> {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct MappedStruct<D, T> {
    r#type: TypeHandle<T>,
    data: D,
    offset: usize,
}

impl<D, T> MappedStruct<D, T>
where
    D: Borrow<[u8]> + Clone,
    T: Borrow<TypeRegistry> + Clone,
{
    #[inline]
    fn new(
        r#type: TypeHandle<T>,
        data: D,
        offset: usize,
    ) -> Self {
        Self {
            r#type,
            data,
            offset,
        }
    }

    #[inline]
    pub fn r#type(&self) -> &TypeHandle<T> {
        &self.r#type
    }

    #[inline]
    pub fn data(&self) -> &D {
        &self.data
    }

    pub fn get_field_type(
        &self,
        field_name: &str
    ) -> Result<Option<TypeHandle<T>>, MappingError> {
        let field = match self.get_field_metadata(field_name) {
            Some(value) => value,
            None => return Ok(None),
        };

        let type_registry = self.r#type.type_registry().clone();
        let field_type = match TypeHandle::try_new(type_registry, field.r#type) {
            Some(t) => t,
            None => return Err(MappingError::InvalidTypeIndex(field.r#type)),
        };

        Ok(Some(field_type))
    }

    pub fn get_field_metadata(
        &self,
        field_name: &str
    ) -> Option<&StructFieldMetadata> {
        let type_registry = self.r#type.type_registry().borrow();
        let mut r#type = self.r#type.deref();
        let mut field;

        loop {
            field = r#type.struct_fields.get(field_name);

            if field.is_some() {
                break;
            }

            match type_registry.get_inner_type(r#type) {
                Some(parent_type) => r#type = parent_type,
                None => return None,
            }
        }

        field
    }

    pub fn get(
        &self,
        field_name: &str
    ) -> Result<Option<MappedValue<D, T>>, MappingError> {
        let field = match self.get_field_metadata(field_name) {
            Some(value) => value,
            None => return Ok(None),
        };

        let type_registry = self.r#type.type_registry().clone();
        let field_type = match TypeHandle::try_new(type_registry, field.r#type) {
            Some(t) => t,
            None => return Err(MappingError::InvalidTypeIndex(field.r#type)),
        };

        let field_offset = field.data_offset as usize;
        let field_value = MappedValue::from_impl(
            &field_type,
            &self.data,
            self.offset + field_offset
        )?;

        Ok(Some(field_value))
    }

    pub fn iter(&self) -> impl Iterator<Item = Result<(&str, MappedValue<D, T>), MappingError>> {
        self.r#type.iter_fields()
            .map(|field| {
                let type_registry = self.r#type.type_registry().clone();
                let field_type = TypeHandle::try_new(type_registry, field.r#type)
                    .ok_or(MappingError::InvalidTypeIndex(field.r#type))?;
                let field_offset = field.data_offset as usize;
                let field_value = MappedValue::from_impl(
                    &field_type,
                    &self.data,
                    self.offset + field_offset
                )?;

                Ok((field.name.as_str(), field_value))
            })
    }

    #[inline]
    pub fn iter_keys(&self) -> impl Iterator<Item = &str> {
        self.r#type.iter_fields().map(|field| field.name.as_str())
    }

    #[inline]
    pub fn len(&self) -> Result<usize, MappingError> {
        let mut r#type = self.r#type.clone();
        let mut total = r#type.field_count;

        while let Some(parent_type) = get_inner_type_opt(&r#type) {
            total += parent_type.field_count;
            r#type = parent_type;
        }

        Ok(total as usize)
    }

    #[inline]
    pub fn is_empty(&self) -> Result<bool, MappingError> {
        Ok(self.len()? == 0)
    }

}

#[derive(Debug, Clone)]
pub struct MappedArray<D, T> {
    r#type: TypeHandle<T>,
    element_type: TypeHandle<T>,
    data: D,
    offset: usize,
    length: usize,
}

impl<D, T> MappedArray<D, T>
where
    D: Borrow<[u8]> + Clone,
    T: Borrow<TypeRegistry> + Clone,
{
    #[inline]
    fn new(
        r#type: TypeHandle<T>,
        data: D,
        offset: usize,
        length: usize,
    ) -> Self {
        let element_type = get_inner_type(&r#type);

        Self {
            r#type,
            element_type,
            data,
            offset,
            length,
        }
    }

    #[inline]
    pub fn r#type(&self) -> &TypeHandle<T> {
        &self.r#type
    }

    #[inline]
    pub fn element_type(&self) -> &TypeHandle<T> {
        &self.element_type
    }

    #[inline]
    pub fn data(&self) -> &D {
        &self.data
    }

    #[inline]
    pub fn get(
        &self,
        index: usize
    ) -> Result<Option<MappedValue<D, T>>, MappingError> {
        if index >= self.length {
            return Ok(None);
        }

        let element_size = self.element_type.size as usize;
        let element_offset = index * element_size;

        Ok(Some(MappedValue::from_impl(
            &self.element_type,
            &self.data,
            self.offset + element_offset
        )?))
    }

    #[inline]
    pub fn iter(
        &self
    ) -> impl Iterator<Item = Result<MappedValue<D, T>, MappingError>> + '_ {
        (0..self.length).filter_map(move |index| self.get(index).transpose())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

#[derive(Debug, Clone)]
pub struct MappedReference<T> {
    r#type: TypeHandle<T>,
    guid: BlobGuid,
}

impl<T> MappedReference<T>
where
    T: Borrow<TypeRegistry> + Clone,
{
    #[inline]
    fn new(r#type: TypeHandle<T>, guid: BlobGuid) -> Self {
        Self { r#type, guid }
    }

    #[inline]
    pub fn r#type(&self) -> &TypeHandle<T> {
        &self.r#type
    }

    #[inline]
    pub fn guid(&self) -> &BlobGuid {
        &self.guid
    }

    #[inline]
    pub fn into_guid(self) -> BlobGuid {
        self.guid
    }
}

macro_rules! impl_from_mapped_value {
    ($type:ty, $variant:ident) => {
        impl<D, T> From<$type> for MappedValue<D, T>
        where
            D: Borrow<[u8]> + Clone,
            T: Borrow<TypeRegistry> + Clone,
        {
            #[inline]
            fn from(value: $type) -> Self {
                MappedValue::$variant(value)
            }
        }
    };
}

impl_from_mapped_value!(bool, Bool);
impl_from_mapped_value!(u8, UInt8);
impl_from_mapped_value!(i8, SInt8);
impl_from_mapped_value!(u16, UInt16);
impl_from_mapped_value!(i16, SInt16);
impl_from_mapped_value!(u32, UInt32);
impl_from_mapped_value!(i32, SInt32);
impl_from_mapped_value!(u64, UInt64);
impl_from_mapped_value!(i64, SInt64);
impl_from_mapped_value!(f32, Float32);
impl_from_mapped_value!(f64, Float64);
impl_from_mapped_value!(MappedEnum<T>, Enum);
impl_from_mapped_value!(MappedBitmask<T>, Bitmask);
impl_from_mapped_value!(MappedStruct<D, T>, Struct);
impl_from_mapped_value!(MappedArray<D, T>, Array);
impl_from_mapped_value!(MappedString<D>, String);
impl_from_mapped_value!(MappedOptional<D, T>, Optional);
impl_from_mapped_value!(MappedVariant<D, T>, Variant);
impl_from_mapped_value!(MappedReference<T>, Reference);
impl_from_mapped_value!(BlobGuid, Guid);

use std::borrow::Borrow;

use kfc::reflection::TypeRegistry;

use super::{TypeHandle, MappingError};

#[inline]
pub fn get_inner_type<T>(
    r#type: &TypeHandle<T>
) -> TypeHandle<T>
where
    T: Borrow<TypeRegistry> + Clone,
{
    r#type.inner_type()
        .map(|t| t.unwrap_typedef())
        .expect("invalid type handle: inner type is None")
}

#[inline]
pub fn get_inner_type_opt<T>(
    r#type: &TypeHandle<T>
) -> Option<TypeHandle<T>>
where
    T: Borrow<TypeRegistry> + Clone,
{
    r#type.inner_type()
        .map(|t| t.unwrap_typedef())
}

#[inline]
pub fn get_bytes(buf: &[u8], offset: usize, length: usize) -> Result<&[u8], MappingError> {
    buf.get(offset..offset + length)
        .ok_or(MappingError::UnexpectedEndOfData)
}

#[inline]
pub fn get_bool(buf: &[u8], offset: usize) -> Result<bool, MappingError> {
    buf.get(offset)
        .map(|&b| b != 0)
        .ok_or(MappingError::UnexpectedEndOfData)
}

#[inline]
pub fn get_u8(buf: &[u8], offset: usize) -> Result<u8, MappingError> {
    buf.get(offset)
        .copied()
        .ok_or(MappingError::UnexpectedEndOfData)
}

#[inline]
pub fn get_i8(buf: &[u8], offset: usize) -> Result<i8, MappingError> {
    buf.get(offset)
        .map(|&b| b as i8)
        .ok_or(MappingError::UnexpectedEndOfData)
}

#[inline]
pub fn get_u16(buf: &[u8], offset: usize) -> Result<u16, MappingError> {
    buf.get(offset..offset + 2)
        .map(|slice| u16::from_le_bytes(slice.try_into().unwrap()))
        .ok_or(MappingError::UnexpectedEndOfData)
}

#[inline]
pub fn get_i16(buf: &[u8], offset: usize) -> Result<i16, MappingError> {
    buf.get(offset..offset + 2)
        .map(|slice| i16::from_le_bytes(slice.try_into().unwrap()))
        .ok_or(MappingError::UnexpectedEndOfData)
}

#[inline]
pub fn get_u32(buf: &[u8], offset: usize) -> Result<u32, MappingError> {
    buf.get(offset..offset + 4)
        .map(|slice| u32::from_le_bytes(slice.try_into().unwrap()))
        .ok_or(MappingError::UnexpectedEndOfData)
}

#[inline]
pub fn get_i32(buf: &[u8], offset: usize) -> Result<i32, MappingError> {
    buf.get(offset..offset + 4)
        .map(|slice| i32::from_le_bytes(slice.try_into().unwrap()))
        .ok_or(MappingError::UnexpectedEndOfData)
}

#[inline]
pub fn get_u64(buf: &[u8], offset: usize) -> Result<u64, MappingError> {
    buf.get(offset..offset + 8)
        .map(|slice| u64::from_le_bytes(slice.try_into().unwrap()))
        .ok_or(MappingError::UnexpectedEndOfData)
}

#[inline]
pub fn get_i64(buf: &[u8], offset: usize) -> Result<i64, MappingError> {
    buf.get(offset..offset + 8)
        .map(|slice| i64::from_le_bytes(slice.try_into().unwrap()))
        .ok_or(MappingError::UnexpectedEndOfData)
}

#[inline]
pub fn get_f32(buf: &[u8], offset: usize) -> Result<f32, MappingError> {
    buf.get(offset..offset + 4)
        .map(|slice| f32::from_le_bytes(slice.try_into().unwrap()))
        .ok_or(MappingError::UnexpectedEndOfData)
}

#[inline]
pub fn get_f64(buf: &[u8], offset: usize) -> Result<f64, MappingError> {
    buf.get(offset..offset + 8)
        .map(|slice| f64::from_le_bytes(slice.try_into().unwrap()))
        .ok_or(MappingError::UnexpectedEndOfData)
}

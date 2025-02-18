use std::io::Cursor;
use std::path::Path;

use shared::io::ReadExt;

use super::ReflectionParseError;
use super::pe_file::{PEFile, ReadPEExt};
use super::types::*;

pub fn extract_reflection_data(exe_file: impl AsRef<Path>) -> Result<Vec<TypeInfo>, ReflectionParseError> {
    let pe_file = PEFile::load_from_file(exe_file)?;
    let data_section_offset = pe_file.offset_to_section(".data")
        .ok_or(ReflectionParseError::MissingDataSection)?;
    let rdata_section_offset = pe_file.offset_to_section(".rdata")
        .ok_or(ReflectionParseError::MissingRDataSection)?;

    let offset_to_blob_string_literal = pe_file.find(
        rdata_section_offset - 1,
        [0x00, 0x42, 0x6C, 0x6F, 0x62, 0x53, 0x74, 0x72, 0x69, 0x6E, 0x67, 0x00],
        8
    )
        .and_then(|offset| pe_file.fo_to_va(offset + 1))
        .ok_or(ReflectionParseError::MalformedPattern)?;

    let offset = pe_file.find_pointer_to_0va(rdata_section_offset, offset_to_blob_string_literal)
        .and_then(|offset| pe_file.fo_to_va(offset))
        .ok_or(ReflectionParseError::MalformedPattern)?;

    let offset = pe_file.find_pointer_to_0va(data_section_offset, offset)
        .and_then(|offset| pe_file.fo_to_va(offset))
        .ok_or(ReflectionParseError::MalformedPattern)?;

    let offset = pe_file.find_pointer_to_0va(rdata_section_offset, offset)
        .ok_or(ReflectionParseError::MalformedPattern)?;

    let mut cursor = pe_file.get_cursor_at(offset)?;
    let mut table_cursor = cursor.read_pointee(&pe_file)?;
    let table_count = cursor.read_u64()?;

    let mut table = Vec::with_capacity(table_count as usize);

    for _ in 0..table_count {
        let mut type_cursor = table_cursor.read_pointee(&pe_file)?;
        let ty = read_type(&mut type_cursor, &pe_file, false)?;

        table.push(ty);
    }

    Ok(table)
}

/// # Layout
/// 
/// ```c
/// struct TypeInfo {
///     char* type_name_1_ptr;
///     u64 type_name_1_len;
///     char* type_name_2_ptr;
///     u64 type_name_2_len;
///     char* qualified_name_ptr;
///     u64 qualified_name_len;
///     Namespace* namespace;
///     struct {
///         char* referenced_type_name_ptr;
///         u64 referenced_type_name_len;
///     }* referenced_type_name;
///     u32 class_size;
///     u16 flag_0;
///     u16 flag_1;
///     u32 field_count;
///     u8 primitive_type;
///     u8 flag_2;
///     u8 padding[2];
///     u32 hash_1;
///     u32 hash_2;
///     StructFieldInfo* struct_fields[field_count]; // ptr to array of field_count StructFieldInfos
///     EnumFieldInfo* enum_fields[field_count]; // ptr to array of field_count EnumFieldInfos
/// }
/// ```
fn read_type(
    cursor: &mut Cursor<&[u8]>,
    pe_file: &PEFile,
    skip_fields: bool
) -> std::io::Result<TypeInfo> {
    let name = cursor.read_pointee(pe_file)?
        .read_string(cursor.read_u64()? as usize)?;
    let impact_name = cursor.read_pointee(pe_file)?
        .read_string(cursor.read_u64()? as usize)?;
    let qualified_name = cursor.read_pointee(pe_file)?
        .read_string(cursor.read_u64()? as usize)?;

    let namespace_cursor = cursor.read_pointee(pe_file)?;
    let namespace = read_namespace(namespace_cursor, pe_file)?;

    let inner_type = read_type_ref(cursor, pe_file)?;

    let size = cursor.read_u32()?;
    let alignment = cursor.read_u16()?;
    let element_alignment = cursor.read_u16()?;
    let field_count = cursor.read_u32()?;
    let primitive_type = PrimitiveType::from_u8(cursor.read_u8()?);
    let flags = cursor.read_u8()?;
    cursor.padding(2)?;
    let qualified_hash = cursor.read_u32()?;
    let impact_hash = cursor.read_u32()?;

    let struct_fields = if !skip_fields {
        cursor.read_pointee_opt(pe_file)?
            .map(|mut cursor| {
                read_struct_fields(&mut cursor, field_count as u64, pe_file)
            })
            .transpose()?
            .unwrap_or_else(Vec::new)
    } else {
        Vec::new()
    };

    let enum_fields = if !skip_fields {
        cursor.read_pointee_opt(pe_file)?
            .map(|mut cursor| {
                read_enum_fields(&mut cursor, field_count as u64, pe_file)
            })
            .transpose()?
            .unwrap_or_else(Vec::new)
    } else {
        Vec::new()
    };

    Ok(TypeInfo {
        name,
        impact_name,
        qualified_name,
        namespace,
        inner_type,
        size,
        alignment,
        element_alignment,
        field_count,
        primitive_type,
        flags,
        qualified_hash,
        impact_hash,
        struct_fields,
        enum_fields,
    })
}

fn read_struct_fields(
    cursor: &mut Cursor<&[u8]>,
    count: u64,
    pe_file: &PEFile,
) -> std::io::Result<Vec<StructFieldInfo>> {
    let mut fields = Vec::with_capacity(count as usize);

    for _ in 0..count {
        fields.push(read_struct_field(cursor, pe_file)?);
    }

    Ok(fields)
}

/// # Layout
///
/// ```c
/// struct StructFieldInfo {
///     char* name_ptr;
///     u64 name_len;
///     TypeInfo* type;
///     u64 data_offset;
///     StructFieldAttribute* attributes;
///     u64 attribute_count;
/// }
/// ```
fn read_struct_field(
    cursor: &mut Cursor<&[u8]>,
    pe_file: &PEFile,
) -> std::io::Result<StructFieldInfo> {
    let name = cursor.read_pointee(pe_file)?
        .read_string(cursor.read_u64()? as usize)?;
    let type_info = read_type_ref(cursor, pe_file)?
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "TypeRef is None"))?;
    let data_offset = cursor.read_u64()?;
    let attributes_cursor = cursor.read_pointee_opt(pe_file)?;
    let attribute_count = cursor.read_u64()?;

    let attributes = attributes_cursor.map(|mut cursor| {
        read_attributes(&mut cursor, attribute_count, pe_file)
    })
        .transpose()?
        .unwrap_or_else(Vec::new);

    Ok(StructFieldInfo {
        name,
        r#type: type_info,
        data_offset,
        attributes,
    })
}

fn read_enum_fields(
    cursor: &mut Cursor<&[u8]>,
    count: u64,
    pe_file: &PEFile,
) -> std::io::Result<Vec<EnumFieldInfo>> {
    let mut fields = Vec::with_capacity(count as usize);

    for _ in 0..count {
        fields.push(read_enum_field(cursor, pe_file)?);
    }

    Ok(fields)
}

/// # Layout
///
/// ```c
/// struct EnumFieldInfo {
///     char* name_ptr;
///     u64 name_len;
///     u64 value;
///     u8 padding[16];
/// }
/// ```
fn read_enum_field(
    cursor: &mut Cursor<&[u8]>,
    pe_file: &PEFile,
) -> std::io::Result<EnumFieldInfo> {
    let name = cursor.read_pointee(pe_file)?
        .read_string(cursor.read_u64()? as usize)?;
    let value = cursor.read_u64()?;
    
    cursor.padding(16)?;
    
    Ok(EnumFieldInfo {
        name,
        value,
    })
}

fn read_attributes(
    cursor: &mut Cursor<&[u8]>,
    count: u64,
    pe_file: &PEFile,
) -> std::io::Result<Vec<StructFieldAttribute>> {
    let mut attributes = Vec::with_capacity(count as usize);

    for _ in 0..count {
        attributes.push(read_attribute(cursor, pe_file)?);
    }

    Ok(attributes)
}

/// # Layout
/// 
/// ```c
/// struct StructFieldAttribute {
///     StructFieldAttributeData* data;
///     char* value_ptr;
///     u64 value_len;
/// }
/// 
/// struct StructFieldAttributeData {
///     Namespace* namespace;
///     char* name_ptr;
///     u64 name_len;
///     TypeInfo* type;
/// }
/// ```
fn read_attribute(
    cursor: &mut Cursor<&[u8]>,
    pe_file: &PEFile,
) -> std::io::Result<StructFieldAttribute> {
    let mut data_cursor = cursor.read_pointee(pe_file)?;
    let value = cursor.read_pointee(pe_file)?
        .read_string(cursor.read_u64()? as usize)?;

    // data
    let namespace = read_namespace(data_cursor.read_pointee(pe_file)?, pe_file)?;
    let name = data_cursor.read_pointee(pe_file)?
        .read_string(data_cursor.read_u64()? as usize)?;
    let r#type = read_type_ref(&mut data_cursor, pe_file)?;

    Ok(StructFieldAttribute {
        name,
        namespace,
        r#type,
        value,
    })
}

/// # Layout
/// 
/// ```c
/// struct Namespace {
///     char* name_ptr;
///     u64 name_len;
///     Namespace* parent;
/// };
/// ```
fn read_namespace(
    cursor: Cursor<&[u8]>,
    pe_file: &PEFile,
) -> std::io::Result<Vec<String>> {
    let mut namespaces = Vec::new();
    let mut parent_cursor = Some(cursor);

    while let Some(mut cursor) = parent_cursor {
        let name = cursor.read_pointee(pe_file)?
            .read_string(cursor.read_u64()? as usize)?;

        namespaces.push(name);
        parent_cursor = cursor.read_pointee_opt(pe_file)?;
    }
    
    namespaces.reverse();

    Ok(namespaces)
}

fn read_type_ref(
    cursor: &mut Cursor<&[u8]>,
    pe_file: &PEFile,
) -> std::io::Result<Option<TypeRef>> {
    cursor.read_pointee_opt(pe_file)?
        .map(|mut cursor| {
            let ty = read_type(
                &mut cursor,
                pe_file,
                true
            )?;

            Ok(TypeRef {
                name: ty.qualified_name,
                hash: ty.qualified_hash,
            })
        })
        .transpose()
}
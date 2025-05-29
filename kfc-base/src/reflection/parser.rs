use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

use crate::{hash::fnv, io::ReadExt};

use super::pe_file::{PEFile, ReadPEExt};
use super::types::*;
use super::{ReflectionParseError, TypeCollection};

pub fn extract_reflection_data(
    exe_file: impl AsRef<Path>,
    deserialize_default_values: bool,
) -> Result<Vec<TypeInfo>, ReflectionParseError> {
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
    let start_position = table_cursor.stream_position()?;
    let table_count = cursor.read_u64()?;

    let mut reference_table = HashMap::new();
    let mut reference_table_cursor = table_cursor.clone();

    for _ in 0..table_count {
        let offset = reference_table_cursor.read_u64()?;
        reference_table.insert(offset, reference_table.len());
    }

    let mut table = Vec::with_capacity(table_count as usize);

    for _ in 0..table_count {
        let mut type_cursor = table_cursor.read_pointee(&pe_file)?;
        let ty = read_type(&mut type_cursor, &pe_file, &reference_table, false)?;

        table.push(ty);
    }

    if deserialize_default_values {
        table_cursor.seek(SeekFrom::Start(start_position))?;

        let mut type_collection = TypeCollection::default();
        let mut values = Vec::with_capacity(table_count as usize);

        type_collection.extend(table);

        for _ in 0..table_count {
            let mut type_cursor = table_cursor.read_pointee(&pe_file)?;
            let value = read_default_value(
                &mut type_cursor,
                &pe_file,
                &type_collection,
            )?;

            values.push(value);
        }

        table = type_collection.into_inner().unwrap();

        for (ty, value) in table.iter_mut().zip(values) {
            ty.default_value = value;
        }
    }

    Ok(table)
}

/// # Layout
///
/// ```c
/// struct TypeInfo {
///     char* name_ptr;
///     u64 name_len;
///     char* impact_name_ptr;
///     u64 impact_name_len;
///     char* qualified_name_ptr;
///     u64 qualified_name_len;
///     Namespace* namespace;
///     TypeInfo* inner_type;
///     u32 size;
///     u16 alignment;
///     u16 element_alignment;
///     u32 field_count;
///     u8 primitive_type;
///     TypeFlags flags;
///     u8 padding[2];
///     u32 qualified_hash; // @0x50
///     u32 internal_hash;
///     StructFieldInfo* struct_fields[field_count]; // ptr to array of field_count StructFieldInfos
///     EnumFieldInfo* enum_fields[field_count]; // ptr to array of field_count EnumFieldInfos
///     TypeInfo** variant_type; // contains as inner_type
///     u8* default_value_ptr; // @0x70
///     u64 default_value_len;
///     Attribute* attributes_ptr;
///     u64 attributes_count;
/// }
///
/// enum TypeFlags : u8 {
///     None = 0x00,
///     HasDS = 0x01,
///     HasBlobArray = 0x02,
///     HasBlobString = 0x04,
///     HasBlobOptional = 0x08,
///     HasBlobVariant = 0x10,
///     Unknown_0x20 = 0x20, // shader/gpu related?
///     Unknown_0x40 = 0x40, // shader/gpu related?
///     Unknown_0x80 = 0x80, // shader/gpu related?
/// }
/// ```
fn read_type(
    cursor: &mut Cursor<&[u8]>,
    pe_file: &PEFile,
    reference_table: &HashMap<u64, usize>,
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

    let inner_type = read_type_ref(cursor, reference_table)?;

    let size = cursor.read_u32()?;
    let alignment = cursor.read_u16()?;
    let element_alignment = cursor.read_u16()?;
    let field_count = cursor.read_u32()?;
    let primitive_type = PrimitiveType::from_u8(cursor.read_u8()?);
    let flags = TypeFlags::from_bits_truncate(cursor.read_u8()?);
    cursor.padding(2)?;
    let qualified_hash = cursor.read_u32()?;
    let internal_hash = cursor.read_u32()?;

    let (struct_fields, enum_fields) = if !skip_fields {
        let struct_fields = cursor.read_pointee_opt(pe_file)?
            .map(|mut cursor| {
                read_struct_fields(&mut cursor, field_count as u64, pe_file, reference_table)
            })
            .transpose()?
            .unwrap_or_else(Vec::new);

        let enum_fields = cursor.read_pointee_opt(pe_file)?
            .map(|mut cursor| {
                read_enum_fields(&mut cursor, field_count as u64, pe_file)
            })
            .transpose()?
            .unwrap_or_else(Vec::new);

        (struct_fields, enum_fields)
    } else {
        cursor.seek_relative(16)?;
        (Vec::new(), Vec::new())
    };

    cursor.seek_relative(8)?; // skip variant_type
    cursor.seek_relative(8)?; // skip default_value_ptr
    cursor.seek_relative(8)?; // skip default_value_len

    let attributes_cursor = cursor.read_pointee_opt(pe_file)?;
    let attributes_count = cursor.read_u64()?;
    let attributes = attributes_cursor.map(|mut cursor| {
        read_attributes(&mut cursor, attributes_count, pe_file, reference_table)
    })
        .transpose()?
        .unwrap_or_else(Vec::new);

    Ok(TypeInfo {
        name_hash: fnv(&name),
        impact_hash: fnv(&impact_name),

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
        internal_hash,
        struct_fields,
        enum_fields,
        default_value: None,
        attributes,
    })
}

fn read_default_value(
    cursor: &mut Cursor<&[u8]>,
    pe_file: &PEFile,
    type_collection: &TypeCollection,
) -> std::io::Result<Option<serde_json::Value>> {
    cursor.seek_relative(0x50)?; // skip to qualified_hash
    let qualified_hash = cursor.read_u32()?;

    cursor.seek_relative(0x20 - 0x04)?; // skip to default_value_ptr

    let default_value_cursor = cursor.read_pointee_opt(pe_file)?;
    let default_value_len = cursor.read_u64()?;

    if let Some(mut cursor) = default_value_cursor {
        let type_info = type_collection.get_type_by_qualified_hash(qualified_hash)
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to find type by hash: {}", qualified_hash)
            ))?;

        let mut value = vec![0; default_value_len as usize];
        cursor.read_exact(&mut value)?;

        let value = type_collection.deserialize(type_info, &value)
            .map_err(|e| std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to deserialize default value: {}", e)
            ))?;

        Ok(Some(value))
    } else {
        Ok(None)
    }
}

fn read_struct_fields(
    cursor: &mut Cursor<&[u8]>,
    count: u64,
    pe_file: &PEFile,
    reference_table: &HashMap<u64, usize>,
) -> std::io::Result<Vec<StructFieldInfo>> {
    let mut fields = Vec::with_capacity(count as usize);

    for _ in 0..count {
        fields.push(read_struct_field(cursor, pe_file, reference_table)?);
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
///     Attribute* attributes;
///     u64 attributes_count;
/// }
/// ```
fn read_struct_field(
    cursor: &mut Cursor<&[u8]>,
    pe_file: &PEFile,
    reference_table: &HashMap<u64, usize>,
) -> std::io::Result<StructFieldInfo> {
    let name = cursor.read_pointee(pe_file)?
        .read_string(cursor.read_u64()? as usize)?;
    let type_info = read_type_ref(cursor, reference_table)?
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "TypeRef is None"))?;
    let data_offset = cursor.read_u64()?;
    let attributes_cursor = cursor.read_pointee_opt(pe_file)?;
    let attribute_count = cursor.read_u64()?;

    let attributes = attributes_cursor.map(|mut cursor| {
        read_attributes(&mut cursor, attribute_count, pe_file, reference_table)
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
    reference_table: &HashMap<u64, usize>,
) -> std::io::Result<Vec<Attribute>> {
    let mut attributes = Vec::with_capacity(count as usize);

    for _ in 0..count {
        attributes.push(read_attribute(cursor, pe_file, reference_table)?);
    }

    Ok(attributes)
}

/// # Layout
///
/// ```c
/// struct Attribute {
///     AttributeInfo* info;
///     char* value_ptr;
///     u64 value_len;
/// }
///
/// struct AttributeInfo {
///     Namespace* namespace;
///     char* name_ptr;
///     u64 name_len;
///     TypeInfo* type;
/// }
/// ```
fn read_attribute(
    cursor: &mut Cursor<&[u8]>,
    pe_file: &PEFile,
    reference_table: &HashMap<u64, usize>,
) -> std::io::Result<Attribute> {
    let mut data_cursor = cursor.read_pointee(pe_file)?;
    let value = cursor.read_pointee(pe_file)?
        .read_string(cursor.read_u64()? as usize)?;

    // data
    let namespace = read_namespace(data_cursor.read_pointee(pe_file)?, pe_file)?;
    let name = data_cursor.read_pointee(pe_file)?
        .read_string(data_cursor.read_u64()? as usize)?;
    let r#type = read_type_ref(&mut data_cursor, reference_table)?;

    Ok(Attribute {
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
    reference_table: &HashMap<u64, usize>,
) -> std::io::Result<Option<usize>> {
    let offset = cursor.read_u64()?;

    if offset == 0 {
        return Ok(None);
    }

    Ok(reference_table.get(&offset).copied())
}

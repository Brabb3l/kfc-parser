use std::collections::HashMap;
use std::io::{Cursor, Error, ErrorKind, Read, Result, Seek};
use std::path::Path;

use indexmap::IndexMap;

use crate::reflection::TypeIndex;
use crate::{hash::fnv, io::ReadExt};

use super::pe_file::{PEFile, ReadPEExt};
use super::types::*;
use super::error::ReflectionParseError;

pub fn extract_reflection_data<P: AsRef<Path>>(
    exe_file: P,
) -> std::result::Result<Vec<TypeMetadata>, ReflectionParseError> {
    let pe_file = PEFile::load_from_file(exe_file)?;
    // let data_section_offset = pe_file.offset_to_section(".data")
    //     .ok_or(ReflectionParseError::MissingDataSection)?;
    let rdata_section_offset = pe_file.offset_to_section(".rdata")
        .ok_or(ReflectionParseError::MissingRDataSection)?;

    let offset_to_blob_string_literal = pe_file.find(
        rdata_section_offset - 1,
        [0x00, 0x42, 0x6C, 0x6F, 0x62, 0x53, 0x74, 0x72, 0x69, 0x6E, 0x67, 0x00],
        8
    )
        .and_then(|offset| pe_file.fo_to_va(offset + 1))
        .ok_or(ReflectionParseError::MalformedPattern)?;

    let offset_to_uint32_literal = pe_file.find(
        rdata_section_offset - 1,
        [0x00, 0x75, 0x69, 0x6E, 0x74, 0x33, 0x32, 0x00],
        8
    )
        .and_then(|offset| pe_file.fo_to_va(offset + 1))
        .ok_or(ReflectionParseError::MalformedPattern)?;

    let offset_1 = pe_file.find_pointer_to_0va(rdata_section_offset, offset_to_blob_string_literal)
        .and_then(|offset| pe_file.fo_to_va(offset))
        .ok_or(ReflectionParseError::MalformedPattern)?;

    let offset_2 = pe_file.find_pointer_to_0va(rdata_section_offset, offset_to_uint32_literal)
        .and_then(|offset| pe_file.fo_to_va(offset))
        .ok_or(ReflectionParseError::MalformedPattern)?;

    let offset = pe_file.find_pointer_to_0va2x(
        rdata_section_offset,
        offset_1,
        offset_2
    )
        .and_then(|offset| pe_file.fo_to_va(offset))
        .ok_or(ReflectionParseError::MalformedPattern)?;

    let offset = pe_file.find_pointer_to_0va(rdata_section_offset, offset)
        .ok_or(ReflectionParseError::MalformedPattern)?;

    let mut cursor = pe_file.get_cursor_at(offset)?;
    let mut table_cursor = cursor.read_pointee(&pe_file)?;
    let table_count = cursor.read_u64()?;

    let mut reference_table = HashMap::new();
    let mut reference_table_cursor = table_cursor.clone();

    for _ in 0..table_count {
        let offset = reference_table_cursor.read_u64()?;
        reference_table.insert(offset, reference_table.len());
    }

    let mut table = Vec::with_capacity(table_count as usize);

    for i in 0..table_count {
        let mut type_cursor = table_cursor.read_pointee(&pe_file)?;
        let ty = read_type(
            &mut type_cursor,
            &pe_file,
            &reference_table,
            TypeIndex(i as usize)
        )?;

        table.push(ty);
    }

    Ok(table)
}

/// # Layout
///
/// ```c
/// struct TypeMetadata {
///     char* name_ptr;
///     u64 name_len;
///     char* impact_name_ptr;
///     u64 impact_name_len;
///     char* qualified_name_ptr;
///     u64 qualified_name_len;
///     Namespace* namespace;
///     TypeMetadata* inner_type;
///     u32 size;
///     u16 alignment;
///     u16 element_alignment;
///     u32 field_count;
///     u8 primitive_type;
///     TypeFlags flags;
///     u8 padding[2];
///     u32 qualified_hash; // @0x50
///     u32 internal_hash;
///     StructFieldMetadata* struct_fields[field_count]; // ptr to array of field_count StructFieldMetadatas
///     EnumFieldMetadata* enum_fields[field_count]; // ptr to array of field_count EnumFieldMetadatas
///     TypeMetadata** variant_type; // contains as inner_type
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
///     IsGpuUniform = 0x20,
///     IsGpuStorage = 0x40,
///     IsGpuConstant = 0x80,
/// }
/// ```
fn read_type(
    cursor: &mut Cursor<&[u8]>,
    pe_file: &PEFile,
    reference_table: &HashMap<u64, usize>,
    index: TypeIndex,
) -> Result<TypeMetadata> {
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

    let struct_fields = cursor.read_pointee_opt(pe_file)?
        .map(|mut cursor| {
            read_struct_fields(
                &mut cursor,
                field_count as u64,
                pe_file,
                reference_table
            )
        })
        .transpose()?
        .unwrap_or_else(IndexMap::new);

    let enum_fields = cursor.read_pointee_opt(pe_file)?
        .map(|mut cursor| {
            read_enum_fields(
                &mut cursor,
                field_count as u64,
                pe_file
            )
        })
        .transpose()?
        .unwrap_or_else(IndexMap::new);

    cursor.seek_relative(8)?; // skip variant_type

    let default_value_cursor = cursor.read_pointee_opt(pe_file)?;
    let default_value_len = cursor.read_u64()?;
    let default_value = default_value_cursor.map(|mut cursor| {
        let mut value = vec![0; default_value_len as usize];
        cursor.read_exact(&mut value)?;
        Result::Ok(value)
    }).transpose()?;

    let attributes_cursor = cursor.read_pointee_opt(pe_file)?;
    let attributes_count = cursor.read_u64()?;
    let attributes = attributes_cursor.map(|mut cursor| {
        read_attributes(
            &mut cursor,
            attributes_count,
            pe_file,
            reference_table
        )
    })
        .transpose()?
        .unwrap_or_else(IndexMap::new);

    Ok(TypeMetadata {
        name_hash: fnv(&name),
        impact_hash: fnv(&impact_name),

        index,
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
        default_value,
        attributes,
    })
}

fn read_struct_fields(
    cursor: &mut Cursor<&[u8]>,
    count: u64,
    pe_file: &PEFile,
    reference_table: &HashMap<u64, usize>,
) -> Result<IndexMap<String, StructFieldMetadata>> {
    let mut fields = IndexMap::with_capacity(count as usize);

    for _ in 0..count {
        let field = read_struct_field(
            cursor,
            pe_file,
            reference_table
        )?;

        fields.insert(field.name.clone(), field);
    }

    Ok(fields)
}

/// # Layout
///
/// ```c
/// struct StructFieldMetadata {
///     char* name_ptr;
///     u64 name_len;
///     TypeMetadata* type;
///     u64 data_offset;
///     Attribute* attributes;
///     u64 attributes_count;
/// }
/// ```
fn read_struct_field(
    cursor: &mut Cursor<&[u8]>,
    pe_file: &PEFile,
    reference_table: &HashMap<u64, usize>,
) -> Result<StructFieldMetadata> {
    let name = cursor.read_pointee(pe_file)?
        .read_string(cursor.read_u64()? as usize)?;
    let r#type = read_type_ref(cursor, reference_table)?
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "TypeRef is None"))?;
    let data_offset = cursor.read_u64()?;
    let attributes_cursor = cursor.read_pointee_opt(pe_file)?;
    let attribute_count = cursor.read_u64()?;

    let attributes = attributes_cursor.map(|mut cursor| {
        read_attributes(
            &mut cursor,
            attribute_count,
            pe_file,
            reference_table
        )
    })
        .transpose()?
        .unwrap_or_else(IndexMap::new);

    Ok(StructFieldMetadata {
        name,
        r#type,
        data_offset,
        attributes,
    })
}

fn read_enum_fields(
    cursor: &mut Cursor<&[u8]>,
    count: u64,
    pe_file: &PEFile,
) -> Result<IndexMap<String, EnumFieldMetadata>> {
    let mut fields = IndexMap::with_capacity(count as usize);

    for _ in 0..count {
        let field = read_enum_field(
            cursor,
            pe_file
        )?;

        fields.insert(field.name.clone(), field);
    }

    Ok(fields)
}

/// # Layout
///
/// ```c
/// struct EnumFieldMetadata {
///     char* name_ptr;
///     u64 name_len;
///     u64 value;
///     u8 padding[16];
/// }
/// ```
fn read_enum_field(
    cursor: &mut Cursor<&[u8]>,
    pe_file: &PEFile,
) -> Result<EnumFieldMetadata> {
    let name = cursor.read_pointee(pe_file)?
        .read_string(cursor.read_u64()? as usize)?;
    let value = cursor.read_u64()?;

    cursor.padding(16)?;

    Ok(EnumFieldMetadata {
        name,
        value,
    })
}

fn read_attributes(
    cursor: &mut Cursor<&[u8]>,
    count: u64,
    pe_file: &PEFile,
    reference_table: &HashMap<u64, usize>,
) -> Result<IndexMap<String, Attribute>> {
    let mut attributes = IndexMap::with_capacity(count as usize);

    for _ in 0..count {
        let attribute = read_attribute(
            cursor,
            pe_file,
            reference_table
        )?;

        attributes.insert(attribute.name.clone(), attribute);
    }

    Ok(attributes)
}

/// # Layout
///
/// ```c
/// struct Attribute {
///     AttributeMetadata* metadata;
///     char* value_ptr;
///     u64 value_len;
/// }
///
/// struct AttributeMetadata {
///     Namespace* namespace;
///     char* name_ptr;
///     u64 name_len;
///     TypeMetadata* type;
/// }
/// ```
fn read_attribute(
    cursor: &mut Cursor<&[u8]>,
    pe_file: &PEFile,
    reference_table: &HashMap<u64, usize>,
) -> Result<Attribute> {
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
) -> Result<Vec<String>> {
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
) -> Result<Option<TypeIndex>> {
    let offset = cursor.read_u64()?;

    if offset == 0 {
        return Ok(None);
    }

    Ok(reference_table.get(&offset).copied().map(TypeIndex))
}

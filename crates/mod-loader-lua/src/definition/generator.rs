use kfc::reflection::{PrimitiveType, TypeFlags, TypeMetadata, TypeRegistry};

const PRELUDE: &str = r#"---@meta
-- This file was automatically generated.

"#;

/// Generates a lua_ls definition file from the given type collection.
pub fn generate(
    type_registry: &TypeRegistry,
) -> String {
    let mut output = String::new();

    output.push_str(PRELUDE);

    for r#type in type_registry.iter() {
        if r#type.flags.contains(TypeFlags::HAS_DS) {
            continue;
        }

        match r#type.primitive_type {
            PrimitiveType::Enum => {
                // ---@alias EnumName
                // ---| "EnumValue1"
                // ---| "EnumValue2"

                output.push_str("---@alias ");
                append_qualified_name(&mut output, r#type.qualified_name.as_str());
                output.push('\n');

                for field in r#type.enum_fields.values() {
                    output.push_str("---| \"");
                    output.push_str(field.name.as_str());
                    output.push_str("\"\n");
                }

                output.push('\n');
            }
            PrimitiveType::Struct => {
                // ---@class StructName
                // ---@field field_name FieldType

                output.push_str("---@class ");
                append_qualified_name(&mut output, r#type.qualified_name.as_str());

                if let Some(inner) = r#type.inner_type {
                    let inner_type = type_registry.get(inner)
                        .expect("Struct should have an inner type");

                    output.push_str(" : ");
                    append_type_name(&mut output, inner_type, type_registry);
                }

                output.push('\n');

                for field in r#type.struct_fields.values() {
                    output.push_str("---@field ");
                    output.push_str(field.name.as_str());
                    output.push(' ');
                    append_type_name(&mut output, type_registry.get(field.r#type).unwrap(), type_registry);
                    output.push('\n');
                }

                output.push('\n');
            }
            _ => {}
        }
    }

    output
}

fn append_type_name(
    output: &mut String,
    r#type: &TypeMetadata,
    type_registry: &TypeRegistry,
) {
    match r#type.primitive_type {
        PrimitiveType::None => output.push_str("nil"),
        PrimitiveType::Bool => output.push_str("bool"),
        PrimitiveType::UInt8 => output.push_str("u8"),
        PrimitiveType::SInt8 => output.push_str("i8"),
        PrimitiveType::UInt16 => output.push_str("u16"),
        PrimitiveType::SInt16 => output.push_str("i16"),
        PrimitiveType::UInt32 => output.push_str("u32"),
        PrimitiveType::SInt32 => output.push_str("i32"),
        PrimitiveType::UInt64 => output.push_str("u64"),
        PrimitiveType::SInt64 => output.push_str("i64"),
        PrimitiveType::Float32 => output.push_str("f32"),
        PrimitiveType::Float64 => output.push_str("f64"),
        PrimitiveType::Enum => append_qualified_name(output, r#type.qualified_name.as_str()),
        PrimitiveType::Bitmask8 | PrimitiveType::Bitmask16 | PrimitiveType::Bitmask32 | PrimitiveType::Bitmask64 => {
            output.push_str("Bitmask<");
            let inner_type = type_registry.get_inner_type(r#type)
                .expect("Bitmask type should have an inner type");
            append_type_name(output, inner_type, type_registry);
            output.push('>');
        }
        PrimitiveType::Typedef => {
            let inner_type = type_registry.get_inner_type(r#type)
                .expect("Typedef type should have an inner type");
            append_type_name(output, inner_type, type_registry);
        }
        PrimitiveType::Struct => append_qualified_name(output, r#type.qualified_name.as_str()),
        PrimitiveType::StaticArray => {
            output.push_str("StaticArray<");
            let inner_type = type_registry.get_inner_type(r#type)
                .expect("StaticArray type should have an inner type");
            append_type_name(output, inner_type, type_registry);
            output.push_str(", ");
            output.push_str(&r#type.field_count.to_string());
            output.push('>');
        }
        PrimitiveType::DsArray => unimplemented!(),
        PrimitiveType::DsString => unimplemented!(),
        PrimitiveType::DsOptional => unimplemented!(),
        PrimitiveType::DsVariant => unimplemented!(),
        PrimitiveType::BlobArray => {
            output.push_str("Array<");
            let inner_type = type_registry.get_inner_type(r#type)
                .expect("BlobArray type should have an inner type");
            append_type_name(output, inner_type, type_registry);
            output.push('>');
        }
        PrimitiveType::BlobString => {
            output.push_str("string");
        }
        PrimitiveType::BlobOptional => {
            let inner_type = type_registry.get_inner_type(r#type)
                .expect("BlobOptional type should have an inner type");
            append_type_name(output, inner_type, type_registry);
            output.push('?');
        }
        PrimitiveType::BlobVariant => {
            let inner_type = type_registry.get_inner_type(r#type)
                .expect("BlobVariant type should have an inner type");
            output.push_str("Variant<");
            append_type_name(output, inner_type, type_registry);
            output.push('>');
        }
        PrimitiveType::ObjectReference => {
            let inner_type = type_registry.get_inner_type(r#type)
                .expect("ObjectReference type should have an inner type");

            output.push_str("ObjectReference<");
            append_qualified_name(output, inner_type.qualified_name.as_str());
            output.push('>');
        },
        PrimitiveType::Guid => output.push_str("Guid"),
    }
}

fn append_qualified_name(
    output: &mut String,
    qualified_name: &str,
) {
    let mut chars = qualified_name.chars();

    loop {
        match chars.next() {
            Some(':') => {
                chars.next();
                output.push('.');
            }
            Some(c) => output.push(c),
            None => break,
        }
    }
}

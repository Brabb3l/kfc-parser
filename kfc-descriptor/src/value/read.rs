use std::borrow::Borrow;

use indexmap::IndexMap;
use kfc::{
    guid::BlobGuid,
    reflection::{TypeMetadata, TypeRegistry},
};

use crate::{
    mapped::{
        MappedArray, MappedBit, MappedEnum, MappedStruct, MappedValue, MappedVariant, MappingError,
    },
    value::{Value, Variant},
};

#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// The representation of enum values.
    pub enum_repr: EnumRepr,
    /// The representation of bitmask values.
    pub bitmask_repr: BitmaskRepr,
    /// Options for variants.
    pub variant: VariantOptions,
    /// Whether to convert GUIDs to strings.
    pub guid_as_string: bool,
}

impl ConversionOptions {
    pub const COMPACT: Self = Self {
        enum_repr: EnumRepr::Value,
        bitmask_repr: BitmaskRepr::Value,
        variant: VariantOptions::COMPACT,
        guid_as_string: false,
    };

    pub const HUMAN_READABLE: Self = Self {
        enum_repr: EnumRepr::Name,
        bitmask_repr: BitmaskRepr::ArrayName,
        variant: VariantOptions::HUMAN_READABLE,
        guid_as_string: true,
    };
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self::COMPACT
    }
}

#[derive(Debug, Clone)]
pub struct VariantOptions {
    /// Whether to convert variants to structs with a `$type` and `$value` field.
    pub as_struct: bool,
    /// If `as_struct` and this is `true`, the `$type` field will be the qualified type name,
    /// otherwise it will be the type's index.
    pub qualified_type_name: bool,
}

impl VariantOptions {
    pub const COMPACT: Self = Self {
        as_struct: false,
        qualified_type_name: false,
    };

    pub const HUMAN_READABLE: Self = Self {
        as_struct: true,
        qualified_type_name: true,
    };
}

impl Default for VariantOptions {
    fn default() -> Self {
        Self::COMPACT
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnumRepr {
    /// An integer value representing the enum's value.
    Value,
    /// A string representation of the enum's name.
    Name,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BitmaskRepr {
    /// An array of integer values representing the bit indices that are set in the bitmask.
    ArrayValue,
    /// An array of string values representing the names of the bits.
    /// If a bit does not have a name, it will be represented as an integer value (bit index) instead.
    ArrayName,
    /// An integer value representing the bitmask's value.
    Value,
}

impl Value {
    #[inline]
    pub fn from_bytes<T: AsRef<[u8]>>(
        type_registry: &TypeRegistry,
        r#type: &TypeMetadata,
        value: T,
    ) -> Result<Self, MappingError> {
        Self::from_bytes_with_options(type_registry, r#type, value, ConversionOptions::default())
    }

    #[inline]
    pub fn from_bytes_with_options<T: AsRef<[u8]>>(
        type_registry: &TypeRegistry,
        r#type: &TypeMetadata,
        value: T,
        options: ConversionOptions,
    ) -> Result<Self, MappingError> {
        let value = MappedValue::from_bytes(&type_registry, r#type, &value.as_ref())?;

        Self::from_impl(value, &options)
    }

    #[inline]
    pub fn from<D, T>(value: MappedValue<D, T>) -> Result<Self, MappingError>
    where
        D: Borrow<[u8]> + Clone,
        T: Borrow<TypeRegistry> + Clone,
    {
        Self::from_with_options(value, ConversionOptions::default())
    }

    #[inline]
    pub fn from_with_options<D, T>(
        value: MappedValue<D, T>,
        options: ConversionOptions,
    ) -> Result<Self, MappingError>
    where
        D: Borrow<[u8]> + Clone,
        T: Borrow<TypeRegistry> + Clone,
    {
        Self::from_impl(value, &options)
    }

    fn from_impl<D, T>(
        value: MappedValue<D, T>,
        options: &ConversionOptions,
    ) -> Result<Self, MappingError>
    where
        D: Borrow<[u8]> + Clone,
        T: Borrow<TypeRegistry> + Clone,
    {
        Ok(match value {
            MappedValue::None => Value::None,
            MappedValue::Bool(b) => Value::Bool(b),
            MappedValue::UInt8(v) => Value::UInt(v.into()),
            MappedValue::SInt8(v) => Value::SInt(v.into()),
            MappedValue::UInt16(v) => Value::UInt(v.into()),
            MappedValue::SInt16(v) => Value::SInt(v.into()),
            MappedValue::UInt32(v) => Value::UInt(v.into()),
            MappedValue::SInt32(v) => Value::SInt(v.into()),
            MappedValue::UInt64(v) => Value::UInt(v),
            MappedValue::SInt64(v) => Value::SInt(v),
            MappedValue::Float32(v) => Value::Float(v.into()),
            MappedValue::Float64(v) => Value::Float(v),
            MappedValue::Enum(v) => Self::from_enum(options, v),
            MappedValue::Bitmask8(v) => Self::from_bitmask(v.value().into(), || v.bits(), options),
            MappedValue::Bitmask16(v) => Self::from_bitmask(v.value().into(), || v.bits(), options),
            MappedValue::Bitmask32(v) => Self::from_bitmask(v.value().into(), || v.bits(), options),
            MappedValue::Bitmask64(v) => Self::from_bitmask(v.value(), || v.bits(), options),
            MappedValue::Struct(r#struct) => Self::from_struct(r#struct, options)?,
            MappedValue::Array(array) => Self::from_array(array, options)?,
            MappedValue::String(s) => Value::String(s.as_str()?.to_string()),
            MappedValue::Optional(v) => v
                .map(|v| Self::from_impl(*v, options))
                .transpose()?
                .unwrap_or(Value::None),
            MappedValue::Variant(variant) => Self::from_variant(variant, options)?,
            MappedValue::Guid(guid) => Self::from_guid(guid, options),
            MappedValue::Reference(reference) => Self::from_guid(reference.guid().clone(), options),
        })
    }

    #[inline]
    fn from_enum<T>(options: &ConversionOptions, r#enum: MappedEnum<T>) -> Value
    where
        T: Borrow<TypeRegistry> + Clone,
    {
        match options.enum_repr {
            EnumRepr::Value => Value::UInt(r#enum.value()),
            EnumRepr::Name => r#enum
                .name()
                .map(str::to_string)
                .map(Value::String)
                .unwrap_or_else(|| Value::UInt(r#enum.value())),
        }
    }

    #[inline]
    fn from_bitmask<'a, F>(value: u64, bits: F, options: &ConversionOptions) -> Value
    where
        F: Fn() -> Vec<MappedBit<'a>>,
    {
        match options.bitmask_repr {
            BitmaskRepr::ArrayValue => Value::Array(
                bits()
                    .iter()
                    .map(|v| Value::UInt(v.value())).collect()
            ),
            BitmaskRepr::ArrayName => Value::Array(
                bits()
                    .iter()
                    .map(|v| {
                        v.name()
                            .map(str::to_string)
                            .map(Value::String)
                            .unwrap_or_else(|| Value::UInt(v.value()))
                    })
                    .collect(),
            ),
            BitmaskRepr::Value => Value::UInt(value),
        }
    }

    #[inline]
    fn from_struct<D, T>(
        r#struct: MappedStruct<D, T>,
        options: &ConversionOptions
    ) -> Result<Value, MappingError>
    where
        D: Borrow<[u8]> + Clone,
        T: Borrow<TypeRegistry> + Clone,
    {
        let mut map = IndexMap::with_capacity(r#struct.len()?);

        for field in r#struct.iter() {
            let (name, value) = field?;
            map.insert(name.to_string(), Self::from_impl(value, options)?);
        }

        Ok(Value::Struct(map.into()))
    }

    #[inline]
    fn from_array<D, T>(
        array: MappedArray<D, T>,
        options: &ConversionOptions,
    ) -> Result<Value, MappingError>
    where
        D: Borrow<[u8]> + Clone,
        T: Borrow<TypeRegistry> + Clone,
    {
        let mut values = Vec::with_capacity(array.len());

        for value in array.iter() {
            values.push(Self::from_impl(value?, options)?);
        }

        Ok(Value::Array(values))
    }

    #[inline]
    fn from_variant<D, T>(
        variant: Option<MappedVariant<D, T>>,
        options: &ConversionOptions,
    ) -> Result<Value, MappingError>
    where
        D: Borrow<[u8]> + Clone,
        T: Borrow<TypeRegistry> + Clone,
    {
        Ok(match variant {
            None => Value::None,
            Some(variant) => {
                let value = Self::from_impl(MappedValue::Struct(variant.value().clone()), options)?
                    .into_struct()
                    .expect("Expected variant value to be a struct");

                if !options.variant.as_struct {
                    let type_index = variant.variant_type().index;

                    Value::Variant(Variant { type_index, value }.into())
                } else {
                    let mut map = IndexMap::with_capacity(2);

                    if options.variant.qualified_type_name {
                        let name = variant.variant_type().qualified_name.clone();
                        map.insert("$type".to_string(), Value::String(name));
                    } else {
                        let type_index = variant.variant_type().index.as_usize() as u64;
                        map.insert("$type".to_string(), Value::UInt(type_index));
                    }

                    map.insert("$value".to_string(), Value::Struct(value.into()));

                    Value::Struct(map.into())
                }
            }
        })
    }

    #[inline]
    fn from_guid(guid: BlobGuid, options: &ConversionOptions) -> Value {
        if guid.is_none() {
            Value::None
        } else if options.guid_as_string {
            Value::String(guid.to_string())
        } else {
            Value::Guid(guid)
        }
    }
}

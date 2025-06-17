use indexmap::IndexMap;
use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    Deserialize, Serialize,
};

use crate::value::Value;

impl Serialize for Value {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::None => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::UInt(n) => n.serialize(serializer),
            Value::SInt(n) => n.serialize(serializer),
            Value::Float(f) => f.serialize(serializer),
            Value::String(s) => serializer.serialize_str(s),
            Value::Struct(m) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m.iter() {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Value::Array(v) => v.serialize(serializer),
            Value::Variant(v) => {
                use serde::ser::SerializeStruct;
                let mut s = serializer.serialize_struct("Variant", 2)?;
                s.serialize_field("$type", &v.type_index)?;
                s.serialize_field("$value", &v.value)?;
                s.end()
            }
            Value::Guid(guid) => guid.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("any valid value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                Ok(Value::SInt(value))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
                Ok(Value::UInt(value))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
                Ok(Value::Float(value))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Value, E> {
                Ok(Value::String(value))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::None)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::None)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }

                Ok(Value::Array(vec))
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut values = IndexMap::with_capacity(visitor.size_hint().unwrap_or(0));

                while let Some((key, value)) = visitor.next_entry()? {
                    values.insert(key, value);
                }

                Ok(Value::Struct(values.into()))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

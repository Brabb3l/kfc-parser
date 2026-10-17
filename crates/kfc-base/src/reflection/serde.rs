use serde::{Deserialize, Serialize};

use crate::reflection::{TypeMetadata, TypeRegistry};

#[derive(Deserialize)]
struct TypeRegistrySerdeOwned {
    version: String,
    types: Vec<TypeMetadata>,
}

#[derive(Serialize)]
struct TypeRegistrySerdeRef<'a> {
    version: &'a str,
    types: &'a [TypeMetadata],
}

impl<'de> Deserialize<'de> for TypeRegistry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = TypeRegistrySerdeOwned::deserialize(deserializer)?;
        let mut registry = Self {
            version: data.version,
            ..Default::default()
        };

        registry.extend(data.types);

        Ok(registry)
    }
}

impl serde::Serialize for TypeRegistry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data = TypeRegistrySerdeRef {
            version: &self.version,
            types: &self.types,
        };

        data.serialize(serializer)
    }
}

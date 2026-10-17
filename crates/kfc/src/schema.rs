use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct GameFiles {
    pub executable: PathBuf,
    pub kfc: PathBuf,
}

impl GameFiles {
    pub fn client(game: impl AsRef<Path>) -> Self {
        Self {
            executable: game.as_ref().join("enshrouded.exe"),
            kfc: game.as_ref().join("enshrouded.kfc"),
        }
    }

    pub fn server(game: impl AsRef<Path>) -> Self {
        Self {
            executable: game.as_ref().join("enshrouded_server.exe"),
            kfc: game.as_ref().join("enshrouded_server.kfc"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldDefinition {
    pub name: String,
    pub type_name: String,
    pub data_offset: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnumValueDefinition {
    pub name: String,
    pub value: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypeDefinition {
    pub name: String,
    pub impact_name: String,
    pub qualified_name: String,
    pub namespace: Vec<String>,
    pub inner_type: Option<String>,
    pub primitive: String,
    pub size: u32,
    pub alignment: u16,
    pub element_alignment: u16,
    pub field_count: u32,
    pub fields: BTreeMap<String, FieldDefinition>,
    pub enum_values: BTreeMap<String, EnumValueDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceReference {
    pub guid: String,
    pub part: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSchema {
    pub parser: String,
    pub game_version: String,
    pub types: BTreeMap<String, TypeDefinition>,
    pub resources: BTreeMap<String, Vec<ResourceReference>>,
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("game executable does not exist: {0}")]
    MissingExecutable(PathBuf),
    #[error("game data does not exist: {0}")]
    MissingData(PathBuf),
    #[error("parser backend failed: {0}")]
    Backend(String),
}

pub trait GameParser: Send + Sync {
    fn id(&self) -> &'static str;
    fn parse(&self, files: &GameFiles) -> Result<GameSchema, ParserError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct KfcParser;

impl GameParser for KfcParser {
    fn id(&self) -> &'static str {
        "kfc"
    }

    fn parse(&self, files: &GameFiles) -> Result<GameSchema, ParserError> {
        use crate::{container::KFCFile, reflection::TypeRegistry};

        require_file(&files.executable, true)?;
        require_file(&files.kfc, false)?;
        let registry = TypeRegistry::load_from_executable(&files.executable)
            .map_err(|error| ParserError::Backend(error.to_string()))?;
        let container = KFCFile::from_path(&files.kfc, false)
            .map_err(|error| ParserError::Backend(error.to_string()))?;

        Ok(schema_from_registry(&registry, &container))
    }
}

/// Build the API view from already loaded metadata; never rescan the executable.
pub fn schema_from_registry(
    registry: &crate::reflection::TypeRegistry,
    container: &crate::container::KFCFile,
) -> GameSchema {
    use crate::reflection::LookupKey;

    let mut types = BTreeMap::new();
    for metadata in registry.iter() {
        let fields = metadata
            .struct_fields
            .values()
            .map(|field| {
                let type_name = registry
                    .get(field.r#type)
                    .map(|value| value.qualified_name.clone())
                    .unwrap_or_else(|| "<unknown>".into());
                (
                    field.name.clone(),
                    FieldDefinition {
                        name: field.name.clone(),
                        type_name,
                        data_offset: field.data_offset,
                    },
                )
            })
            .collect();
        let enum_values = metadata
            .enum_fields
            .values()
            .map(|field| {
                (
                    field.name.clone(),
                    EnumValueDefinition {
                        name: field.name.clone(),
                        value: field.value,
                    },
                )
            })
            .collect();
        types.insert(
            metadata.qualified_name.clone(),
            TypeDefinition {
                name: metadata.name.clone(),
                impact_name: metadata.impact_name.clone(),
                qualified_name: metadata.qualified_name.clone(),
                namespace: metadata.namespace.clone(),
                inner_type: metadata
                    .inner_type
                    .and_then(|index| registry.get(index))
                    .map(|value| value.qualified_name.clone()),
                primitive: format!("{:?}", metadata.primitive_type),
                size: metadata.size,
                alignment: metadata.alignment,
                element_alignment: metadata.element_alignment,
                field_count: metadata.field_count,
                fields,
                enum_values,
            },
        );
    }

    let mut resources = BTreeMap::new();
    for type_hash in container.resource_types() {
        let Some(metadata) = registry.get_by_hash(LookupKey::Qualified(type_hash)) else {
            continue;
        };
        let values = container
            .resources_by_type(type_hash)
            .map(|resource| ResourceReference {
                guid: resource.to_string(),
                part: resource.part_index(),
            })
            .collect();
        resources.insert(metadata.qualified_name.clone(), values);
    }

    GameSchema {
        parser: "kfc".into(),
        game_version: container.game_version().into(),
        types,
        resources,
    }
}

fn require_file(path: &Path, executable: bool) -> Result<(), ParserError> {
    if path.is_file() {
        return Ok(());
    }
    if executable {
        Err(ParserError::MissingExecutable(path.into()))
    } else {
        Err(ParserError::MissingData(path.into()))
    }
}

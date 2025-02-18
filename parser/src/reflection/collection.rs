use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::sync::Arc;

use shared::hash::fnv;

use super::types::*;
use super::{parser, ReflectionParseError, TypeParseError};

#[derive(Debug, Default)]
pub struct TypeCollection {
    types: Vec<Arc<TypeInfo>>,
    types_by_qualified_hash: HashMap<u32, Arc<TypeInfo>>,
    types_by_impact_hash: HashMap<u32, Arc<TypeInfo>>,
}

impl TypeCollection {
    
    pub fn get_type_by_qualified_hash(
        &self,
        hash: u32
    ) -> Option<&TypeInfo> {
        self.types_by_qualified_hash.get(&hash)
            .map(|node| node.as_ref())
    }

    pub fn get_type_by_impact_hash(
        &self,
        hash: u32
    ) -> Option<&TypeInfo> {
        self.types_by_impact_hash.get(&hash)
            .map(|node| node.as_ref())
    }

    pub fn get_type_by_qualified_name(
        &self,
        name: &str
    ) -> Option<&TypeInfo> {
        self.get_type_by_qualified_hash(fnv(name.as_bytes()))
    }

    pub fn get_type_by_impact_name(
        &self,
        name: &str
    ) -> Option<&TypeInfo> {
        self.get_type_by_impact_hash(fnv(name.as_bytes()))
    }

    pub fn get_impact_nodes(&self) -> HashMap<u32, &TypeInfo> {
        let mut nodes = HashMap::new();
    
        for node in self.types_by_qualified_hash.values() {
            let inheritance_chain = self.get_inheritance_chain(node);
    
            for child_node in inheritance_chain {
                if child_node.name == "ImpactNode" {
                    nodes.insert(fnv(node.name.as_bytes()), node.as_ref());
                    break;
                }
            }
        }
    
        nodes
    }
    
    fn get_inheritance_chain<'a>(&'a self, node: &'a TypeInfo) -> Vec<&'a TypeInfo> {
        let mut chain = Vec::new();
        let mut current = node;
    
        loop {
            chain.push(current);
    
            if let Some(parent) = &current.inner_type {
                if let Some(parent) = self.get_type_by_qualified_hash(parent.hash) {
                    current = parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    
        chain
    }
    
    pub fn clear(&mut self) {
        self.types_by_qualified_hash.clear();
        self.types_by_impact_hash.clear();
    }
    
    pub fn load_from_path(&mut self, path: impl AsRef<Path>) -> Result<usize, TypeParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let type_info: Vec<TypeInfo> = serde_json::from_reader(reader)?;
        let len = type_info.len();
        
        for entry in type_info {
            let value = Arc::new(entry);
            
            self.types.push(value.clone());
            self.types_by_qualified_hash.insert(value.qualified_hash, value.clone());
            self.types_by_impact_hash.insert(fnv(value.impact_name.as_bytes()), value);
        }

        Ok(len)
    }
    
    pub fn load_from_executable(&mut self, path: impl AsRef<Path>) -> Result<usize, ReflectionParseError> {
        let types = parser::extract_reflection_data(path)?;
        let len = types.len();
        
        for entry in types {
            let value = Arc::new(entry);
            
            self.types.push(value.clone());
            self.types_by_qualified_hash.insert(value.qualified_hash, value.clone());
            self.types_by_impact_hash.insert(fnv(value.impact_name.as_bytes()), value);
        }
        
        Ok(len)
    }
    
    pub fn dump_to_path(&self, path: impl AsRef<Path>) -> Result<(), TypeParseError> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let types = self.types.iter()
            .map(|node| node.as_ref())
            .collect::<Vec<_>>();
        
        serde_json::to_writer(writer, &types)?;
        
        Ok(())
    }
    
}

use shared::hash::{fnv, fnv_const};
use std::collections::HashMap;

use crate::{reflection::{TypeCollection, TypeInfo}, Hash32};

pub const IMPACT_NODE_EXECUTION: &str = "$ImpactNodeExecution";
pub const IMPACT_NODE_EXECUTION_HASH: u32 = fnv_const(IMPACT_NODE_EXECUTION);

pub const IMPACT_NODE_EXECUTION_BRANCH: &str = "keen::impact_nodes::ImpactNodeExecutionBranch";
pub const IMPACT_NODE_EXECUTION_BRANCH_HASH: u32 = fnv_const(IMPACT_NODE_EXECUTION_BRANCH);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactNode<'a> {
    pub name: &'a str,
    pub hash: Hash32,
    pub r#type: ImpactNodeTypeRef<'a>,
    pub super_types: Vec<ImpactNodeTypeRef<'a>>,
    pub inputs: Vec<ImpactNodePin<'a>>,
    pub outputs: Vec<ImpactNodePin<'a>>,
    pub configs: Vec<ImpactNodePin<'a>>,
    pub values: Vec<ImpactNodePin<'a>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactNodeTypeRef<'a> {
    pub name: &'a str,
    pub hash: Hash32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactNodePin<'a> {
    pub name: &'a str,
    pub r#type: ImpactNodeTypeRef<'a>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_mandatory: bool,
}

impl ImpactNodePin<'_> {
    pub fn is_execution(&self) -> bool {
        self.r#type.hash == IMPACT_NODE_EXECUTION_HASH ||
        self.r#type.hash == IMPACT_NODE_EXECUTION_BRANCH_HASH
    }
}

fn is_false(b: impl std::borrow::Borrow<bool>) -> bool {
    !b.borrow()
}

const IMPACT_INPUT: &str = "impact_node_input";
const IMPACT_OUTPUT: &str = "impact_node_output";
const IMPACT_CONFIG: &str = "impact_config";
const IMPACT_VALUE: &str = "impact_node_value";
const IMPACT_MANDATORY: &str = "impact_mandatory_connection";
const IMPACT_NODE_SHUTDOWN: &str = "impact_node_shutdown";

impl TypeCollection {
    pub fn get_impact_nodes(&self) -> HashMap<u32, ImpactNode> {
        let node_types = self.get_impact_node_types();
        let mut nodes = HashMap::with_capacity(node_types.len());

        for node in node_types.values() {
            self.create_node(node, &mut nodes, None);
        }
        
        nodes
    }

    fn create_node<'a: 'b, 'b: 'c, 'c>(
        &'a self,
        node: &'a TypeInfo,
        nodes: &'c mut HashMap<Hash32, ImpactNode<'b>>,
        shutdown_name: Option<&'a str>,
    ) -> &'c ImpactNode<'b> {
        if nodes.contains_key(&node.name_hash) {
            return &nodes[&node.name_hash];
        }

        if shutdown_name.is_none() {
            if let Some(shutdown_name) = node.attributes.iter()
                .find(|attr| attr.name == IMPACT_NODE_SHUTDOWN)
                .map(|attr| attr.value.as_str())
            {
                self.create_node(node, nodes, Some(shutdown_name));
            }
        }

        let mut super_types = Vec::new();
        let mut inner = &node.inner_type;
        while let Some(ty) = inner {
            let info = self.get_type(*ty)
                .expect("invalid inner type");

            super_types.push(ImpactNodeTypeRef {
                name: &info.qualified_name,
                hash: info.qualified_hash,
            });

            inner = if let Some(ty) = self.get_type(*ty) {
                &ty.inner_type
            } else {
                break;
            };
        }

        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut configs = Vec::new();
        let mut values = Vec::new();

        if let Some(super_type) = super_types.first()
            .map(|ty| ty.hash)
            .and_then(|hash| self.get_type_by_qualified_hash(hash))
        {
            let super_node = self.create_node(super_type, nodes, None);

            inputs.extend(super_node.inputs.iter().cloned());
            outputs.extend(super_node.outputs.iter().cloned());
            configs.extend(super_node.configs.iter().cloned());
            values.extend(super_node.values.iter().cloned());
        }

        if node.qualified_hash == 2802340932 {
            inputs.push(ImpactNodePin {
                name: "",
                r#type: ImpactNodeTypeRef {
                    name: IMPACT_NODE_EXECUTION,
                    hash: IMPACT_NODE_EXECUTION_HASH,
                },
                is_mandatory: false,
            });
            outputs.push(ImpactNodePin {
                name: "",
                r#type: ImpactNodeTypeRef {
                    name: IMPACT_NODE_EXECUTION,
                    hash: IMPACT_NODE_EXECUTION_HASH,
                },
                is_mandatory: false,
            });
        }

        inputs.extend(
            node.struct_fields.iter()
                .filter(|field| field.attributes.iter()
                    .any(|attr| attr.name == IMPACT_INPUT))
                .map(|field| {
                    let ty = self.get_type(field.r#type)
                        .expect("invalid type");

                    ImpactNodePin {
                        name: &field.name,
                        r#type: ImpactNodeTypeRef {
                            name: &ty.qualified_name,
                            hash: ty.qualified_hash,
                        },
                        is_mandatory: field.attributes.iter()
                            .any(|attr| attr.name == IMPACT_MANDATORY)
                    }
                })
        );

        outputs.extend(
            node.struct_fields.iter()
                .filter(|field|
                    field.attributes.iter()
                        .any(|attr| attr.name == IMPACT_OUTPUT) || 
                    self.get_type(field.r#type).unwrap().qualified_hash == IMPACT_NODE_EXECUTION_BRANCH_HASH
                )
                .map(|field| {
                    let ty = self.get_type(field.r#type)
                        .expect("invalid type");

                    ImpactNodePin {
                        name: &field.name,
                        r#type: ImpactNodeTypeRef {
                            name: &ty.qualified_name,
                            hash: ty.qualified_hash,
                        },
                        is_mandatory: field.attributes.iter()
                            .any(|attr| attr.name == IMPACT_MANDATORY)
                    }
                })
        );

        configs.extend(
            node.struct_fields.iter()
                .filter(|field| field.attributes.iter()
                    .any(|attr| attr.name == IMPACT_CONFIG))
                .map(|field| {
                    let ty = self.get_type(field.r#type)
                        .expect("invalid type");

                    ImpactNodePin {
                        name: &field.name,
                        r#type: ImpactNodeTypeRef {
                            name: &ty.qualified_name,
                            hash: ty.qualified_hash,
                        },
                        is_mandatory: false
                    }
                })
        );

        values.extend(
            node.struct_fields.iter()
                .filter(|field| field.attributes.iter()
                    .any(|attr| attr.name == IMPACT_VALUE))
                .map(|field| {
                    let ty = self.get_type(field.r#type)
                        .expect("invalid type");

                    ImpactNodePin {
                        name: &field.name,
                        r#type: ImpactNodeTypeRef {
                            name: &ty.qualified_name,
                            hash: ty.qualified_hash,
                        },
                        is_mandatory: false
                    }
                })
        );
        
        let node = ImpactNode {
            name: shutdown_name.unwrap_or(&node.name),
            hash: fnv(shutdown_name.unwrap_or(&node.name)),
            r#type: ImpactNodeTypeRef {
                name: &node.qualified_name,
                hash: node.qualified_hash,
            },
            super_types,
            inputs,
            outputs,
            configs,
            values,
        };
        
        nodes.entry(node.hash)
            .or_insert_with(|| node)
    }

}

use std::collections::HashMap;
use shared::hash::fnv_const;

use crate::reflection::{TypeCollection, TypeFlags, TypeInfo};

mod nodes;
mod descriptor;

pub mod graph;
pub mod bytecode;

pub use nodes::*;
pub use descriptor::*;

const IMPACT_NODE_HASH: u32 = fnv_const("keen::impact_nodes::ImpactNode");

impl TypeCollection {
    pub(super) fn get_impact_node_types(&self) -> HashMap<u32, &TypeInfo> {
        let mut nodes = HashMap::new();

        for node in self.iter() {
            if node.flags.contains(TypeFlags::HAS_DS) {
                continue;
            }

            let inheritance_chain = self.get_inheritance_chain(node);

            for child_node in inheritance_chain {
                if child_node.qualified_hash == IMPACT_NODE_HASH {
                    nodes.insert(node.name_hash, node);
                    break;
                }
            }
        }

        nodes
    }
}

use serde::{Deserialize, Serialize};
use shared::hash::fnv;
use std::collections::HashMap;

use crate::reflection::{TypeCollection, TypeInfo};

use super::bytecode::{ImpactAssembler, ImpactOps};
use super::{ImpactNode, ImpactProgram};

///// WIP /////

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactNodeGraph {
    nodes: Vec<ImpactNodeInfo>,
    edges: Vec<ImpactNodeEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactNodeInfo {
    r#type: String,
    configs: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactNodeEdge {
    from: usize,
    from_pin: usize,
    to: usize,
    to_pin: usize,
}

pub struct ImpactProgramDecompiler<'a, 'b> {
    type_collection: &'a TypeCollection,
    node_types: HashMap<u32, &'a TypeInfo>,
    node_infos: HashMap<u32, ImpactNode<'a>>,
    value_nodes: HashMap<u32, ImpactNode<'a>>,
    program: &'b ImpactProgram,
    instructions: Vec<ImpactOps>,

    branches: HashMap<usize, usize>, // address -> index of node
    branch_sizes: HashMap<usize, usize>, // address -> size of branch
    outputs: HashMap<u32, (usize, usize)>, // data_layout index -> node index, pin index
    input_bindings: Vec<(usize, usize, u32)>, // node index, pin index, data_layout
    nodes: Vec<ImpactNodeInfo>,
    edges: Vec<ImpactNodeEdge>,
}

struct State {
    branches: Vec<usize>,
}

impl<'a, 'b> ImpactProgramDecompiler<'a, 'b> {
    pub fn new(
        type_collection: &'a TypeCollection,
        program: &'b ImpactProgram,
    ) -> Self {
        let node_types = type_collection.get_impact_node_types();
        let node_infos = type_collection.get_impact_nodes();
        let mut instructions = ImpactAssembler::disassemble(&program.code);
        let value_nodes = node_infos.iter()
            .filter(|(_, node)| node.super_types.iter()
                .any(|t| t.hash == 1667341273 /* keen::impact_nodes::ImpactValueNode */))
            .filter(|(_, node)| !node.outputs.is_empty())
            .map(|(_, node)| {
                let output = node.outputs.first().unwrap();
                let ty = type_collection.get_type_by_qualified_hash(output.r#type.hash)
                    .expect("missing type info");
                (fnv(ty.name.as_bytes()), node.clone())
            })
            .collect::<HashMap<_, _>>();

        Self::byte_offset_to_inst_offset(&mut instructions);

        Self {
            node_types,
            node_infos,
            value_nodes,
            type_collection,
            program,
            instructions,

            branches: HashMap::new(),
            branch_sizes: HashMap::new(),
            outputs: HashMap::new(),
            input_bindings: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
    
    pub fn decompile(mut self) -> ImpactNodeGraph {
        // decompile node graph flow
        self.decompile_section(&mut State {
            branches: Vec::new(),
        }, &mut 0);

        // construct edges
        for (input_node_index, input_pin_index, data_layout_index) in self.input_bindings {
            if let Some((output_node_index, output_pin_index)) = self.outputs.get(&data_layout_index) {
                self.edges.push(ImpactNodeEdge {
                    from: *output_node_index,
                    from_pin: *output_pin_index,
                    to: input_node_index,
                    to_pin: input_pin_index,
                });
            } else {
                // TODO: config ids with value 0 are immediate values
                let layout = &self.program.data_layout[data_layout_index as usize & 0xFFFF];
                let id = layout.config_id.value;
                let node = match self.value_nodes.get(&layout.r#type.value) {
                    Some(node) => node,
                    None => panic!("missing value node: {}", layout.r#type.value),
                };
                let node_index = self.nodes.len();

                self.nodes.push(ImpactNodeInfo {
                    r#type: node.r#type.name.into(),
                    configs: vec![serde_json::Value::Number(id.into())],
                });

                self.edges.push(ImpactNodeEdge {
                    from: node_index,
                    from_pin: 0,
                    to: input_node_index,
                    to_pin: input_pin_index,
                });
            }
        }

        ImpactNodeGraph {
            nodes: self.nodes,
            edges: self.edges,
        }
    }

    fn decompile_section(
        &mut self,
        state: &mut State,
        pc: &mut usize,
    ) -> usize {
        if let Some(branch_index) = self.branches.get(pc).copied() {
            if let Some(size) = self.branch_sizes.get(pc).copied() {
                *pc += size;
            } else {
                panic!("missing branch size");
            }

            return branch_index;
        } else {
            self.branches.insert(*pc, self.nodes.len());
        }

        let start_pc = *pc;
        let node_index = self.decompile_node(state, pc);
        let end_pc = *pc;
        let size = end_pc - start_pc;

        self.branches.insert(start_pc, node_index);
        self.branch_sizes.insert(start_pc, size);

        node_index
    }

    fn decompile_node(&mut self, state: &mut State, pc: &mut usize) -> usize {
        // there are a couple of different variants of intrinsic nodes such as:
        // - for loops
        // - conditional branches
        // - wait
        // - single instructions such as rvm, halt, etc.
        // besides those, there are also regular function nodes

        // # Function Nodes
        // currently there are only `ecall` instructions in the bytecode which follow the
        // following pattern:
        // - they start with `len(inputs) + len(outputs) + len(configs)` `iconst` or `iconst0` instructions
        // - then the `ecall` instruction
        // - then the `pop` instruction

        // # Conditional Branches
        // there are two types of conditional branches:
        // - intrinsic branches such as BoolIfBranchNode
        // - function-like branches using the return value of a function call for the condition
        // ## Intrinsic Branches
        // intrinsic branches seem to make use of the `gload` instruction instead of `iconst` for
        // loading the condition value. the condition value is then compared using `brt` and `brf`
        // instructions.
        // ## Pattern
        // - `gload` or `ecall` instruction (with args before) for the condition
        // - `brt` or `brf` instruction for the branch
        // - the else branch
        // - the code after the branch (if {} else {} /* after */)

        // # Wait
        // - `gload` instruction for the time value
        // - `ltime` instruction for the current time
        // - `iadd`
        // - `dup` // > label_0
        // - `ltime`
        // - `ileq`
        // - `brt label_1`
        // - `rvm` // yield
        // - `br label_0`
        // - > label_1

        // # For Loops
        // - `iconst0` // i = 0
        // - `gstore ForEach::index`
        // - `gload ForEach::index` // > label_0
        // - `gload $INPUT_NODE::count`
        // - `ilt`
        // - `brt label_1`
        // - false branch
        // - > label_1
        // - true branch
        // - `gload ForEach::index`
        // - `inc`
        // - `gstore ForEach::index`
        // - `br label_0`

        enum NodeKind {
            Function(u32),
            ConditionalFunction(u32),
            IfBoolBranch,
            ForLoop,
            Wait,
            Rvm,
            DSelf,
            Halt,
            Unknown
        }

        // find out the node kind
        let mut kind = NodeKind::Unknown;
        let start_pc = *pc;

        loop {
            match self.instructions.get(*pc) {
                Some(ImpactOps::IConst0) => {
                    if *pc == start_pc {
                        if let Some(ImpactOps::GStore(_)) = self.instructions.get(*pc + 1) {
                            kind = NodeKind::ForLoop;
                            break;
                        }
                    }
                }
                Some(ImpactOps::GLoad(_)) => {
                    if *pc == start_pc {
                        if let Some(ImpactOps::LTime) = self.instructions.get(*pc + 1) {
                            kind = NodeKind::Wait;
                            break;
                        } else if let Some(ImpactOps::BRT(_)) = self.instructions.get(*pc + 1) {
                            kind = NodeKind::IfBoolBranch;
                            break;
                        } else if let Some(ImpactOps::BRF(_)) = self.instructions.get(*pc + 1) {
                            kind = NodeKind::IfBoolBranch;
                            break;
                        } else if let Some(ImpactOps::Inc) = self.instructions.get(*pc + 1) {
                            if let Some(ImpactOps::GStore(_)) = self.instructions.get(*pc + 2) {
                                // skip end of for loop
                                *pc += 3;
                                continue;
                            }
                        }
                    }
                }
                Some(ImpactOps::ECall(index)) => {
                    // expect pop instruction
                    match self.instructions.get(*pc + 1) {
                        Some(ImpactOps::Pop) => {
                            kind = NodeKind::Function(*index);
                        }
                        Some(ImpactOps::BRT(_) | ImpactOps::BRF(_)) => {
                            kind = NodeKind::ConditionalFunction(*index);
                        }
                        _ => {}
                    }

                    break;
                }
                Some(ImpactOps::BR(address)) => {
                    *pc += 1;
                    return self.decompile_section(state, &mut (*address as usize));
                }
                Some(ImpactOps::Halt) => {
                    kind = NodeKind::Halt;
                    break;
                }
                Some(ImpactOps::DSelf) => {
                    kind = NodeKind::DSelf;
                    break;
                }
                Some(ImpactOps::RVM) => {
                    kind = NodeKind::Rvm;
                    break;
                }
                _ => {}
            }

            *pc += 1;
        }

        *pc = start_pc;

        let node_index = match kind {
            NodeKind::Function(index) => {
                let node_index = self.decompile_function(state, pc, index);
                *pc += 1;
                node_index
            },
            NodeKind::ConditionalFunction(index) => {
                self.decompile_conditional_function(state, pc, index)
            }
            NodeKind::IfBoolBranch => {
                self.decompile_if_bool_branch(state, pc)
            }
            NodeKind::ForLoop => {
                self.decompile_for_loop(state, pc)
            }
            NodeKind::Wait => {
                self.decompile_wait(state, pc)
            }
            NodeKind::Rvm => {
                *pc += 1;

                let node_index = self.nodes.len();

                if let Some(ImpactOps::BR(address)) = self.instructions.get(*pc) {
                    if *address == 0 {
                        *pc += 1;

                        self.nodes.push(ImpactNodeInfo {
                            r#type: "keen::impact_nodes::ResetNode".into(),
                            configs: Vec::new(),
                        });

                        // doesn't have any outputs
                        return node_index;
                    } else {
                        self.nodes.push(ImpactNodeInfo {
                            r#type: "keen::impact_nodes::RVM".into(),
                            configs: Vec::new(),
                        });
                    }
                } else {
                    self.nodes.push(ImpactNodeInfo {
                        r#type: "keen::impact_nodes::RVM".into(),
                        configs: Vec::new(),
                    });
                }

                node_index
            },
            NodeKind::DSelf => {
                *pc += 1;

                let node_index = self.nodes.len();

                self.nodes.push(ImpactNodeInfo {
                    r#type: "keen::impact_nodes::DeleteSelfNode".into(),
                    configs: Vec::new(),
                });

                // doesn't have any outputs
                return node_index;
            },
            NodeKind::Halt => {
                *pc += 1;

                let node_index = self.nodes.len();

                self.nodes.push(ImpactNodeInfo {
                    r#type: "keen::impact_nodes::HaltNode".into(),
                    configs: Vec::new(),
                });

                // doesn't have any outputs
                return node_index;
            },
            NodeKind::Unknown => {
                panic!("unexpected instruction: {:?}", self.instructions.get(*pc));
            }
        };

        let next_node_index = self.decompile_section(state, pc);
        self.create_exec_edge(
            state,
            node_index,
            0,
            next_node_index,
            0,
        );

        node_index
    }

    fn decompile_function(
        &mut self,
        state: &mut State,
        pc: &mut usize,
        index: u32
    ) -> usize {
        let node_index = self.nodes.len();

        let function = &self.get_function_type(index)
            .and_then(|ty| self.node_infos.get(&ty.qualified_hash))
            .expect("missing function node");
        let input_value_count = function.inputs.iter()
            .filter(|pin| pin.r#type.name != "$ImpactNodeExecution")
            .count();
        let input_exec_count = function.inputs.len() - input_value_count;
        let config_count = function.configs.len();
        let output_count = function.outputs.len();
        let mut i = 0;
        let mut configs = Vec::new();

        loop {
            match self.instructions.get(*pc) {
                Some(ImpactOps::IConst(index)) => {
                    if i < input_value_count {
                        let i = input_value_count - i - 1;
                        self.input_bindings.push((node_index, i + input_exec_count, *index));
                    } else if i < input_value_count + config_count {
                        let json = self.read_variable(state, *index);
                        configs.insert(0, json);
                    } else {
                        let size = input_value_count + config_count + output_count;
                        let i = size - i - 1;
                        self.outputs.insert(*index, (node_index, i));
                    }
                },
                Some(ImpactOps::IConst0) => {
                    if i < input_value_count {
                        // ignore
                    } else if i < input_value_count + config_count {
                        configs.insert(0, serde_json::Value::Null);
                    }
                },
                Some(ImpactOps::ECall(_)) => {
                    *pc += 1;
                    self.nodes.push(ImpactNodeInfo {
                        r#type: function.r#type.name.into(),
                        configs,
                    });
                    break;
                },
                _ => panic!("unexpected instruction: {:?} @ {}", self.instructions.get(*pc), *pc),
            }

            i += 1;
            *pc += 1;
        }

        node_index
    }

    fn decompile_conditional_function(
        &mut self,
        state: &mut State,
        pc: &mut usize,
        index: u32
    ) -> usize {
        let node_index = self.decompile_function(state, pc, index);

        match self.instructions.get(*pc) {
            Some(ImpactOps::BRT(address)) => {
                *pc += 1;

                self.nodes.get_mut(node_index)
                    .unwrap()
                    .configs.insert(0, serde_json::Value::Bool(false));

                let if_node_index = self.decompile_section(state, &mut (*address as usize));
                // let function = &self.get_function_type(index)
                //     .and_then(|ty| self.node_infos.get(&ty.qualified_hash))
                //     .expect("missing function node");

                self.create_exec_edge(
                    state,
                    node_index,
                    1,
                    if_node_index,
                    0,
                );
            },
            Some(ImpactOps::BRF(address)) => {
                *pc += 1;

                self.nodes.get_mut(node_index)
                    .unwrap()
                    .configs.insert(0, serde_json::Value::Bool(true));

                let if_node_index = self.decompile_section(state, &mut (*address as usize));

                self.create_exec_edge(
                    state,
                    node_index,
                    1,
                    if_node_index,
                    0,
                );
            },
            _ => panic!("unexpected instruction: {:?}", self.instructions.get(*pc)),
        }

        // else branch
        let else_node_index = self.decompile_section(state, pc);

        self.create_exec_edge(
            state,
            node_index,
            2,
            else_node_index,
            0,
        );

        node_index
    }

    fn decompile_for_loop(
        &mut self,
        state: &mut State,
        pc: &mut usize
    ) -> usize {
        let node_index = self.nodes.len();

        self.nodes.push(ImpactNodeInfo {
            r#type: "keen::impact_nodes::ForEach".into(),
            configs: Vec::new(),
        });

        // iconst0
        // gstore ForEach::index

        let index = match self.instructions.get(*pc + 1) {
            Some(ImpactOps::GStore(index)) => *index,
            _ => panic!("unexpected instruction: {:?}", self.instructions.get(*pc + 1)),
        };

        self.outputs.insert(index, (node_index, 2));

        *pc += 2;

        let loop_start = *pc;
        self.branches.insert(loop_start, node_index);
        self.branch_sizes.insert(loop_start, 0); // TODO: this should not be needed
        state.branches.push(node_index);

        // gload ForEach::index
        // gload $INPUT_NODE::count
        // ilt

        let count = match self.instructions.get(*pc + 1) {
            Some(ImpactOps::GLoad(index)) => *index,
            _ => panic!("unexpected instruction: {:?}", self.instructions.get(*pc + 1)),
        };

        self.input_bindings.push((node_index, 1, count));

        *pc += 3;

        // brt label_1

        let mut address = match self.instructions.get(*pc) {
            Some(ImpactOps::BRT(address)) => *address as usize,
            _ => panic!("unexpected instruction: {:?}", self.instructions.get(*pc)),
        };

        let do_node_index = self.decompile_section(state, &mut address);

        self.create_exec_edge(
            state,
            node_index,
            1,
            do_node_index,
            0,
        );

        state.branches.pop();

        *pc += 1;

        node_index
    }

    fn decompile_wait(
        &mut self,
        _state: &mut State,
        pc: &mut usize
    ) -> usize {
        let node_index = self.nodes.len();

        self.nodes.push(ImpactNodeInfo {
            r#type: "keen::impact_nodes::Wait".into(),
            configs: Vec::new(),
        });

        // gload WaitDuration::value

        let value = match self.instructions.get(*pc) {
            Some(ImpactOps::GLoad(index)) => *index,
            _ => panic!("unexpected instruction: {:?}", self.instructions.get(*pc)),
        };

        *pc += 1;

        // ltime
        // iadd
        // dup
        // ltime
        // ileq
        // brt ...
        // rvm
        // br ...
        // pop

        *pc += 9;

        self.input_bindings.push((node_index, 1, value));

        node_index
    }

    fn decompile_if_bool_branch(
        &mut self,
        state: &mut State,
        pc: &mut usize
    ) -> usize {
        let node_index = self.nodes.len();

        self.nodes.push(ImpactNodeInfo {
            r#type: "keen::impact_nodes::BoolIfBranchNode".into(),
            configs: Vec::new(),
        });

        // gload

        let value = match self.instructions.get(*pc) {
            Some(ImpactOps::GLoad(index)) => *index,
            _ => panic!("unexpected instruction: {:?}", self.instructions.get(*pc)),
        };

        self.input_bindings.push((node_index, 1, value));

        *pc += 1;

        // brt or brf

        let mut address = match self.instructions.get(*pc) {
            Some(ImpactOps::BRT(address)) => *address as usize,
            Some(ImpactOps::BRF(address)) => *address as usize,
            _ => panic!("unexpected instruction: {:?}", self.instructions.get(*pc)),
        };

        let if_node_index = self.decompile_section(state, &mut address);

        self.create_exec_edge(
            state,
            node_index,
            1,
            if_node_index,
            0,
        );

        *pc += 1;

        let else_node_index = self.decompile_section(state, pc);

        self.create_exec_edge(
            state,
            node_index,
            2,
            else_node_index,
            0,
        );

        node_index
    }

    fn create_exec_edge(
        &mut self,
        state: &State,
        from: usize,
        from_pin: usize,
        to: usize,
        to_pin: usize
    ) {
        if !state.branches.contains(&to) {
            self.edges.push(ImpactNodeEdge {
                from,
                from_pin,
                to,
                to_pin,
            });
        }
    }

    fn read_variable(
        &self,
        _state: &State,
        index: u32
    ) -> serde_json::Value {
        let layout = &self.program.data_layout[index as usize & 0xFFFF];
        let type_info = self.type_collection.get_type_by_impact_hash(layout.r#type.value)
            .expect("missing type info");

        let start = layout.offset_in_bytes as usize;
        let end = start + layout.size as usize;

        self.type_collection.deserialize(
            type_info,
            &self.program.data[start..end]
        ).expect("failed to deserialize data")
    }

    fn get_function_type(&self, index: u32) -> Option<&TypeInfo> {
        self.node_types.get(&index).copied()
    }
    
    fn byte_offset_to_inst_offset(instructions: &mut [ImpactOps]) {
        let mut mappings = HashMap::new();
        let mut byte_offset = 0;

        for (inst_offset, instruction) in instructions.iter().enumerate() {
            mappings.insert(byte_offset, inst_offset);
            byte_offset += instruction.size() as u32;
        }

        for instruction in instructions.iter_mut() {
            match instruction {
                ImpactOps::BR(address) => {
                    *address = *mappings.get(address).unwrap() as u32;
                },
                ImpactOps::BRT(address) => {
                    *address = *mappings.get(address).unwrap() as u32;
                },
                ImpactOps::BRF(address) => {
                    *address = *mappings.get(address).unwrap() as u32;
                },
                _ => {}
            }
        }
    }

}

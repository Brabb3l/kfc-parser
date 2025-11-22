use std::collections::{HashMap, HashSet};
use std::io::Write;

use kfc::Hash32;

use crate::impact::TypeRegistryImpactExt;

use super::parser::Parser;
use super::{ImpactNode, ImpactProgram, ImpactAssembler, ImpactOps, ImpactProgramData, ParseError};

impl ImpactAssembler<'_> {
    pub fn parse_text(&self, data: &ImpactProgramData, code: &str) -> Result<Vec<ImpactOps>, ParseError> {
        Parser::new(&self.type_collection.get_impact_nodes(), data, code).parse()
    }

    // TODO: Replace unwraps with proper error handling
    pub fn write_text<W: Write>(
        &self,
        out: &mut W,
        program: &ImpactProgram,
        code: &[ImpactOps]
    ) -> std::io::Result<()> {
        let mut labels = code.iter()
            .filter_map(|instruction| {
                match instruction {
                    ImpactOps::BR(address) => Some(*address),
                    ImpactOps::BRT(address) => Some(*address),
                    ImpactOps::BRF(address) => Some(*address),
                    _ => None,
                }
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        labels.sort();

        let labels = labels.into_iter()
            .enumerate()
            .map(|(i, address)| (address, i))
            .collect::<HashMap<_, _>>();

        let mut pc = 0;

        for instruction in code {
            if let Some(label) = labels.get(&pc) {
                writeln!(out, "label_{}:", label)?;
            }

            pc += instruction.size() as u32;

            write!(out, "{}", instruction.name())?;

            match instruction {
                ImpactOps::BR(address) => {
                    let label = labels.get(address).unwrap();
                    write!(out, " label_{}", label)?;
                },
                ImpactOps::BRT(address) => {
                    let label = labels.get(address).unwrap();
                    write!(out, " label_{}", label)?;
                },
                ImpactOps::BRF(address) => {
                    let label = labels.get(address).unwrap();
                    write!(out, " label_{}", label)?;
                },
                ImpactOps::IConst(index) |
                ImpactOps::Load(index) |
                ImpactOps::GLoad(index) |
                ImpactOps::Store(index) |
                ImpactOps::GStore(index) => {
                    let index = *index as usize & 0xFFFF;
                    let layout = &program.data_layout[index];

                    write!(out, " {}", layout.dbg_name)?;
                }
                ImpactOps::Call(unk, hash) |
                ImpactOps::ECall(unk, hash) => {
                    if let Some(node) = self.get_call_type(*hash) {
                        write!(out, " {} {} # (", unk, node.name)?;

                        let mut count = 0;

                        for input in &node.inputs {
                            if input.is_execution() { continue; }
                            if count > 0 { write!(out, ", ")? }
                            write!(out, "in {}: {}", input.name, input.r#type.name)?;
                            count += 1;
                        }

                        for config in &node.configs {
                            if count > 0 { write!(out, ", ")? }
                            write!(out, "cfg {}: {}", config.name, config.r#type.name)?;
                            count += 1;
                        }

                        for output in &node.outputs {
                            if output.is_execution() { continue; }
                            if count > 0 { write!(out, ", ")? }
                            write!(out, "out {}: {}", output.name, output.r#type.name)?;
                            count += 1;
                        }

                        write!(out, ")")?;
                    } else {
                        println!("Missing type info for hash: {:08X}", hash);
                        write!(out, " {} {} # missing type info", unk, hash)?;
                    }
                },
                ImpactOps::Unknown(value) => write!(out, "{:08X}", value)?,
                _ => {},
            }

            writeln!(out)?;
        }

        Ok(())
    }

    fn get_call_type(&self, index: Hash32) -> Option<&ImpactNode> {
        self.nodes.get(&index)
    }

}

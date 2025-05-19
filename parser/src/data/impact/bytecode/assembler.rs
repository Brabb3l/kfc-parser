use std::collections::HashMap;

use crate::{reflection::TypeCollection, Hash32};

use super::{ImpactNode, ImpactCommand};

#[derive(Debug)]
pub struct ImpactAssembler<'a> {
    pub(super) type_collection: &'a TypeCollection,
    pub(super) nodes: HashMap<Hash32, ImpactNode<'a>>,
}

impl<'a> ImpactAssembler<'a> {
    pub fn new(type_collection: &'a TypeCollection) -> Self {
        let nodes = type_collection.get_impact_nodes();

        Self {
            type_collection,
            nodes,
        }
    }

    pub fn disassemble(code: &[ImpactCommand]) -> Vec<ImpactOps> {
        let mut instructions = Vec::new();
        let mut ptr = 0;

        while ptr < code.len() {
            let value = code[ptr];
            ptr += 1;

            let value = match value {
                0x00 => ImpactOps::Invalid,
                0x01 => ImpactOps::IAdd,
                0x02 => ImpactOps::ISub,
                0x03 => ImpactOps::IMul,
                0x04 => ImpactOps::IDiv,
                0x05 => ImpactOps::ILT,
                0x06 => ImpactOps::IEQ,
                0x07 => ImpactOps::ILEQ,
                0x08 => ImpactOps::BR({
                    let address = code[ptr];
                    ptr += 1;
                    address
                }),
                0x09 => ImpactOps::BRT({
                    let address = code[ptr];
                    ptr += 1;
                    address
                }),
                0x0A => ImpactOps::BRF({
                    let address = code[ptr];
                    ptr += 1;
                    address
                }),
                0x0B => ImpactOps::IConst({
                    let value = code[ptr];
                    assert_eq!(value & 0xFFFF_0000, 0xFFFF_0000);
                    ptr += 1;
                    value
                }),
                0x0C => ImpactOps::IConst0,
                0x0D => ImpactOps::IConst1,
                0x0E => ImpactOps::Inc,
                0x0F => ImpactOps::Dec,
                0x10 => ImpactOps::Copy,
                0x11 => ImpactOps::Dup,
                0x12 => ImpactOps::Call({
                    let index = code[ptr];
                    ptr += 1;
                    index
                }),
                0x13 => ImpactOps::ECall({
                    let index = code[ptr];
                    ptr += 1;
                    index
                }),
                0x14 => ImpactOps::Ret,
                0x15 => ImpactOps::Load({
                    let index = code[ptr];
                    ptr += 1;
                    index
                }),
                0x16 => ImpactOps::GLoad({
                    let index = code[ptr];
                    ptr += 1;
                    index
                }),
                0x17 => ImpactOps::Store({
                    let index = code[ptr];
                    ptr += 1;
                    index
                }),
                0x18 => ImpactOps::GStore({
                    let index = code[ptr];
                    ptr += 1;
                    index
                }),
                0x19 => ImpactOps::LTime,
                0x1A => ImpactOps::TimeFF,
                0x1B => ImpactOps::Pop,
                0x1C => ImpactOps::RVM,
                0x1D => ImpactOps::DSelf,
                0x1E => ImpactOps::Halt,
                _ => ImpactOps::Unknown(value),
            };

            instructions.push(value);
        }

        instructions
    }

    pub fn assemble(code: &[ImpactOps]) -> Vec<ImpactCommand> {
        let mut instructions = Vec::new();

        for instruction in code {
            match instruction {
                ImpactOps::Invalid => instructions.push(0x00),
                ImpactOps::IAdd => instructions.push(0x01),
                ImpactOps::ISub => instructions.push(0x02),
                ImpactOps::IMul => instructions.push(0x03),
                ImpactOps::IDiv => instructions.push(0x04),
                ImpactOps::ILT => instructions.push(0x05),
                ImpactOps::IEQ => instructions.push(0x06),
                ImpactOps::ILEQ => instructions.push(0x07),
                ImpactOps::BR(address) => {
                    instructions.push(0x08);
                    instructions.push(*address);
                },
                ImpactOps::BRT(address) => {
                    instructions.push(0x09);
                    instructions.push(*address);
                },
                ImpactOps::BRF(address) => {
                    instructions.push(0x0A);
                    instructions.push(*address);
                },
                ImpactOps::IConst(value) => {
                    instructions.push(0x0B);
                    instructions.push(*value);
                },
                ImpactOps::IConst0 => instructions.push(0x0C),
                ImpactOps::IConst1 => instructions.push(0x0D),
                ImpactOps::Inc => instructions.push(0x0E),
                ImpactOps::Dec => instructions.push(0x0F),
                ImpactOps::Copy => instructions.push(0x10),
                ImpactOps::Dup => instructions.push(0x11),
                ImpactOps::Call(index) => {
                    instructions.push(0x12);
                    instructions.push(*index);
                },
                ImpactOps::ECall(index) => {
                    instructions.push(0x13);
                    instructions.push(*index);
                },
                ImpactOps::Ret => instructions.push(0x14),
                ImpactOps::Load(index) => {
                    instructions.push(0x15);
                    instructions.push(*index);
                },
                ImpactOps::GLoad(index) => {
                    instructions.push(0x16);
                    instructions.push(*index);
                },
                ImpactOps::Store(index) => {
                    instructions.push(0x17);
                    instructions.push(*index);
                },
                ImpactOps::GStore(index) => {
                    instructions.push(0x18);
                    instructions.push(*index);
                },
                ImpactOps::LTime => instructions.push(0x19),
                ImpactOps::TimeFF => instructions.push(0x1A),
                ImpactOps::Pop => instructions.push(0x1B),
                ImpactOps::RVM => instructions.push(0x1C),
                ImpactOps::DSelf => instructions.push(0x1D),
                ImpactOps::Halt => instructions.push(0x1E),
                ImpactOps::Unknown(value) => instructions.push(*value),
            }
        }

        instructions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ImpactOps {
    Invalid = 0x00, // unused
    IAdd = 0x01,
    ISub = 0x02, // unused
    IMul = 0x03, // unused
    IDiv = 0x04, // unused
    ILT = 0x05,
    IEQ = 0x06, // unused
    ILEQ = 0x07,
    BR(u32) = 0x08,
    BRT(u32) = 0x09,
    BRF(u32) = 0x0A,
    IConst(u32) = 0x0B, // load data from data_layout table
    IConst0 = 0x0C,
    IConst1 = 0x0D, // unused
    Inc = 0x0E,
    Dec = 0x0F, // unused
    Copy = 0x10, // unused
    Dup = 0x11,
    Call(u32) = 0x12, // unused
    // Call pops each type of argument from the top of the stack until all arguments are consumed.
    // First comes the input arguments, then config arguments and finally the output arguments.
    // f.e.:
    // MyFunction(
    //     cfg flag: bool,
    //     cfg value: float,
    //     in input1: float,
    //     in input2: float,
    //     out output1: float
    //     out output2: float
    // )
    // requires the stack to be in the following order:
    // - input2
    // - input1
    // - value
    // - flag
    // - output2
    // - output1
    ECall(u32) = 0x13,
    Ret = 0x14, // unused
    Load(u32) = 0x15, // unused
    GLoad(u32) = 0x16,
    Store(u32) = 0x17, // unused
    GStore(u32) = 0x18,
    LTime = 0x19, // pushes some time on the stack
    TimeFF = 0x1A, // time from float
    Pop = 0x1B,
    RVM = 0x1C, // yield
    DSelf = 0x1D, // destroy self
    Halt = 0x1E,

    Unknown(u32) = 0xFF,
}

impl ImpactOps {
    pub fn name(&self) -> &'static str {
        match self {
            ImpactOps::Invalid => "invalid",
            ImpactOps::IAdd => "iadd",
            ImpactOps::ISub => "isub",
            ImpactOps::IMul => "imul",
            ImpactOps::IDiv => "idiv",
            ImpactOps::ILT => "ilt",
            ImpactOps::IEQ => "ieq",
            ImpactOps::ILEQ => "ileq",
            ImpactOps::BR(_) => "br",
            ImpactOps::BRT(_) => "brt",
            ImpactOps::BRF(_) => "brf",
            ImpactOps::IConst(_) => "iconst",
            ImpactOps::IConst0 => "iconst0",
            ImpactOps::IConst1 => "iconst1",
            ImpactOps::Inc => "inc",
            ImpactOps::Dec => "dec",
            ImpactOps::Copy => "copy",
            ImpactOps::Dup => "dup",
            ImpactOps::Call(_) => "call",
            ImpactOps::ECall(_) => "ecall",
            ImpactOps::Ret => "ret",
            ImpactOps::Load(_) => "load",
            ImpactOps::GLoad(_) => "gload",
            ImpactOps::Store(_) => "store",
            ImpactOps::GStore(_) => "gstore",
            ImpactOps::LTime => "ltime",
            ImpactOps::TimeFF => "timeff",
            ImpactOps::Pop => "pop",
            ImpactOps::RVM => "rvm",
            ImpactOps::DSelf => "dself",
            ImpactOps::Halt => "halt",
            ImpactOps::Unknown(_) => "unknown",
        }
    }

    pub fn size(&self) -> usize {
        match self {
            ImpactOps::Invalid => 1,
            ImpactOps::IAdd => 1,
            ImpactOps::ISub => 1,
            ImpactOps::IMul => 1,
            ImpactOps::IDiv => 1,
            ImpactOps::ILT => 1,
            ImpactOps::IEQ => 1,
            ImpactOps::ILEQ => 1,
            ImpactOps::BR(_) => 2,
            ImpactOps::BRT(_) => 2,
            ImpactOps::BRF(_) => 2,
            ImpactOps::IConst(_) => 2,
            ImpactOps::IConst0 => 1,
            ImpactOps::IConst1 => 1,
            ImpactOps::Inc => 1,
            ImpactOps::Dec => 1,
            ImpactOps::Copy => 1,
            ImpactOps::Dup => 1,
            ImpactOps::Call(_) => 2,
            ImpactOps::ECall(_) => 2,
            ImpactOps::Ret => 1,
            ImpactOps::Load(_) => 2,
            ImpactOps::GLoad(_) => 2,
            ImpactOps::Store(_) => 2,
            ImpactOps::GStore(_) => 2,
            ImpactOps::LTime => 1,
            ImpactOps::TimeFF => 1,
            ImpactOps::Pop => 1,
            ImpactOps::RVM => 1,
            ImpactOps::DSelf => 1,
            ImpactOps::Halt => 1,
            ImpactOps::Unknown(_) => 1,
        }
    }
}

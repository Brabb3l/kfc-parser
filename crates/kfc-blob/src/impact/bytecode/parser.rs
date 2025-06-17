use std::cell::{Ref, RefCell};
use std::collections::HashMap;

use kfc::hash::fnv;

use super::token::*;
use super::tokenizer::Tokenizer;
use super::{ImpactNode, ImpactOps, ImpactProgramData, ParseError, ParseErrorKind};

pub struct Parser<'a, 'b, 'c> {
    nodes: &'b HashMap<u32, ImpactNode<'b>>,
    impact_data: &'c ImpactProgramData,
    tokenizer: RefCell<Tokenizer<'a>>,
    next: RefCell<Token<'a>>,
}

impl<'a, 'b, 'c> Parser<'a, 'b, 'c> {
    pub fn new(
        nodes: &'b HashMap<u32, ImpactNode>,
        impact_data: &'c ImpactProgramData,
        content: &'a str
    ) -> Self {
        let mut tokenizer = Tokenizer::new(content);
        let next = tokenizer.advance();

        Self {
            nodes,
            impact_data,
            tokenizer: tokenizer.into(),
            next: next.into(),
        }
    }

    pub fn parse(&'a self) -> Result<Vec<ImpactOps>, ParseError> {
        let mut labels = HashMap::new();
        let mut label_mappings = HashMap::new();
        let mut instructions = Vec::new();
        let mut pc = 0;

        while self.peek().kind != TokenKind::Eof {
            let token = self.expect(TokenKind::Identifier)?;
            let identifier = token.content;

            if let Some(stripped) = identifier.strip_suffix(':') {
                if !(labels.contains_key(stripped)) {
                    labels.insert(stripped, pc);
                } else {
                    return Err(ParseError {
                        span: token.span,
                        kind: ParseErrorKind::DuplicateLabel {
                            label: stripped.to_string(),
                        },
                    });
                }
            } else if let Some(keyword) = Self::match_keyword(identifier) {
                let instruction = self.parse_instruction(keyword, &mut label_mappings)?;
                pc += instruction.size();
                instructions.push(instruction);
            } else {
                return Err(ParseError {
                    span: token.span,
                    kind: ParseErrorKind::Expected {
                        expected: "instruction or label".to_string(),
                        found: identifier.to_string(),
                    },
                });
            }
        }

        for instruction in instructions.iter_mut() {
            match instruction {
                ImpactOps::BR(id) |
                ImpactOps::BRT(id) |
                ImpactOps::BRF(id) => {
                    let name = label_mappings.get(&(*id as usize)).expect("unreachable");

                    if let Some(address) = labels.get(name.content) {
                        *id = *address as u32;
                    } else {
                        return Err(ParseError {
                            span: name.span.clone(),
                            kind: ParseErrorKind::UnknownLabel {
                                label: name.content.to_string(),
                            },
                        });
                    }
                },
                _ => {},
            }
        }

        Ok(instructions)
    }

    fn parse_instruction(
        &'a self,
        keyword: KeywordKind,
        label_mappings: &mut HashMap<usize, Token<'a>>,
    ) -> Result<ImpactOps, ParseError> {
        let op = match keyword {
            KeywordKind::Invalid => ImpactOps::Invalid,
            KeywordKind::IAdd => ImpactOps::IAdd,
            KeywordKind::ISub => ImpactOps::ISub,
            KeywordKind::IMul => ImpactOps::IMul,
            KeywordKind::IDiv => ImpactOps::IDiv,
            KeywordKind::Ilt => ImpactOps::ILT,
            KeywordKind::Ieq => ImpactOps::IEQ,
            KeywordKind::Ileq => ImpactOps::ILEQ,
            KeywordKind::IConst0 => ImpactOps::IConst0,
            KeywordKind::IConst1 => ImpactOps::IConst1,
            KeywordKind::Inc => ImpactOps::Inc,
            KeywordKind::Dec => ImpactOps::Dec,
            KeywordKind::Copy => ImpactOps::Copy,
            KeywordKind::Dup => ImpactOps::Dup,
            KeywordKind::Ret => ImpactOps::Ret,
            KeywordKind::LTime => ImpactOps::LTime,
            KeywordKind::TimeFF => ImpactOps::TimeFF,
            KeywordKind::Pop => ImpactOps::Pop,
            KeywordKind::Rvm => ImpactOps::RVM,
            KeywordKind::DSelf => ImpactOps::DSelf,
            KeywordKind::Halt => ImpactOps::Halt,

            KeywordKind::Br => {
                let label = self.expect(TokenKind::Identifier)?;
                let id = label_mappings.len();
                label_mappings.insert(id, label);
                ImpactOps::BR(id as u32)
            },
            KeywordKind::Brt => {
                let label = self.expect(TokenKind::Identifier)?;
                let id = label_mappings.len();
                label_mappings.insert(id, label);
                ImpactOps::BRT(id as u32)
            },
            KeywordKind::Brf => {
                let label = self.expect(TokenKind::Identifier)?;
                let id = label_mappings.len();
                label_mappings.insert(id, label);
                ImpactOps::BRF(id as u32)
            },

            KeywordKind::IConst => ImpactOps::IConst(self.parse_data()?),
            KeywordKind::Load => ImpactOps::Load(self.parse_data()?),
            KeywordKind::GLoad => ImpactOps::GLoad(self.parse_data()?),
            KeywordKind::Store => ImpactOps::Store(self.parse_data()?),
            KeywordKind::GStore => ImpactOps::GStore(self.parse_data()?),

            KeywordKind::Call => ImpactOps::Call(self.parse_call_type()?),
            KeywordKind::ECall => ImpactOps::ECall(self.parse_call_type()?),

            KeywordKind::Unknown => ImpactOps::Unknown(self.parse_number()?),
        };

        Ok(op)
    }

    fn parse_call_type(&self) -> Result<u32, ParseError> {
        let kind = { self.peek().kind };

        match kind {
            TokenKind::Number => self.parse_number(),
            TokenKind::Identifier => {
                let name = self.expect(TokenKind::Identifier)?;
                let hash = fnv(name.content.as_bytes());

                if !self.nodes.contains_key(&hash) {
                    return Err(ParseError {
                        span: name.span,
                        kind: ParseErrorKind::UnknownType {
                            type_name: name.content.to_string(),
                        },
                    });
                }

                Ok(hash)
            }
            _ => Err(ParseError {
                span: self.peek().span.clone(),
                kind: ParseErrorKind::Expected {
                    expected: "function hash or function name".to_string(),
                    found: self.peek().kind.to_string(),
                },
            }),
        }
    }

    fn parse_number(&self) -> Result<u32, ParseError> {
        let token = self.expect(TokenKind::Number)?;

        token.content.parse().map_err(|error| ParseError {
            span: token.span,
            kind: ParseErrorKind::NumberParseError {
                content: token.content.to_string(),
                error,
            },
        })
    }

    fn parse_data(&self) -> Result<u32, ParseError> {
        let token = self.expect(TokenKind::Identifier)?;
        let data = self.impact_data.data.iter()
            .enumerate()
            .find(|(_, data)| data.name == token.content)
            .map(|(index, _)| index as u32 | 0xFFFF_0000)
            .ok_or_else(|| ParseError {
                span: token.span,
                kind: ParseErrorKind::UnknownData {
                    name: token.content.to_string(),
                },
            })?;

        Ok(data)
    }

    fn match_keyword(identifier: &str) -> Option<KeywordKind> {
        let kind = match identifier {
            "invalid" => KeywordKind::Invalid,
            "iadd" => KeywordKind::IAdd,
            "isub" => KeywordKind::ISub,
            "imul" => KeywordKind::IMul,
            "idiv" => KeywordKind::IDiv,
            "ilt" => KeywordKind::Ilt,
            "ieq" => KeywordKind::Ieq,
            "ileq" => KeywordKind::Ileq,
            "br" => KeywordKind::Br,
            "brt" => KeywordKind::Brt,
            "brf" => KeywordKind::Brf,
            "iconst" => KeywordKind::IConst,
            "iconst0" => KeywordKind::IConst0,
            "iconst1" => KeywordKind::IConst1,
            "inc" => KeywordKind::Inc,
            "dec" => KeywordKind::Dec,
            "copy" => KeywordKind::Copy,
            "dup" => KeywordKind::Dup,
            "call" => KeywordKind::Call,
            "ecall" => KeywordKind::ECall,
            "ret" => KeywordKind::Ret,
            "load" => KeywordKind::Load,
            "gload" => KeywordKind::GLoad,
            "store" => KeywordKind::Store,
            "gstore" => KeywordKind::GStore,
            "ltime" => KeywordKind::LTime,
            "timeff" => KeywordKind::TimeFF,
            "pop" => KeywordKind::Pop,
            "rvm" => KeywordKind::Rvm,
            "dself" => KeywordKind::DSelf,
            "halt" => KeywordKind::Halt,
            "unknown" => KeywordKind::Unknown,
            _ => return None,
        };

        Some(kind)
    }

    fn peek(&self) -> Ref<'_, Token<'a>> {
        self.skip_whitespace();
        self.peek0()
    }

    fn next(&self) -> Token {
        self.skip_whitespace();
        self.next0()
    }

    fn expect(&self, kind: TokenKind) -> Result<Token, ParseError> {
        let token = self.next();

        if token.kind != kind {
            return Err(ParseError {
                span: token.span,
                kind: ParseErrorKind::Expected {
                    expected: kind.to_string(),
                    found: token.kind.to_string(),
                },
            });
        }

        Ok(token)
    }

    fn is_whitespace(&self, token: &Token) -> bool {
        matches!(
            token.kind,
            TokenKind::Comment |
            TokenKind::Whitespace |
            TokenKind::Newline
        )
    }

    fn skip_whitespace(&self) {
        while self.is_whitespace(&self.peek0()) {
            self.next0();
        }
    }

    fn peek0(&self) -> Ref<'_, Token<'a>> {
        self.next.borrow()
    }

    fn next0(&self) -> Token {
        let token = self.tokenizer.borrow_mut().advance();
        self.next.replace(token)
    }

}

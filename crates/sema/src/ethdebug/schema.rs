//! ETHDebug JSON schema definitions.
//!
//! This module defines the data structures for the ETHDebug format,
//! based on the [ETHDebug specification](https://github.com/ethdebug/format).

use serde::{Serialize, Serializer};

/// Data types used in ETHDebug format.
pub mod data {
    use super::*;

    /// A hex-encoded value with "0x" prefix.
    #[derive(Clone, Debug)]
    pub struct HexValue(pub Vec<u8>);

    impl Serialize for HexValue {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&alloy_primitives::hex::encode_prefixed(&self.0))
        }
    }

    /// An unsigned integer value that can be either a number or a hex string.
    #[derive(Clone, Debug)]
    pub enum Unsigned {
        /// A numeric value.
        Numeric(u64),
        /// A hex-encoded value for numbers larger than u64.
        Hex(HexValue),
    }

    impl From<u64> for Unsigned {
        fn from(value: u64) -> Self {
            Self::Numeric(value)
        }
    }

    impl From<usize> for Unsigned {
        fn from(value: usize) -> Self {
            Self::Numeric(value as u64)
        }
    }

    impl Serialize for Unsigned {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Self::Numeric(n) => serializer.serialize_u64(*n),
                Self::Hex(h) => h.serialize(serializer),
            }
        }
    }
}

/// Materials (sources and references) used in ETHDebug format.
pub mod materials {
    use super::*;

    /// A material identifier.
    #[derive(Clone, Debug)]
    pub enum Id {
        /// A numeric identifier.
        Numeric(u64),
        /// A string identifier.
        String(String),
    }

    impl Serialize for Id {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Self::Numeric(n) => serializer.serialize_u64(*n),
                Self::String(s) => serializer.serialize_str(s),
            }
        }
    }

    /// The type of reference (compilation or source).
    #[derive(Clone, Copy, Debug, Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum ReferenceType {
        /// Reference to compilation.
        Compilation,
        /// Reference to source.
        Source,
    }

    /// A reference to a material (source or compilation).
    #[derive(Clone, Debug, Serialize)]
    pub struct Reference {
        /// The material identifier.
        pub id: Id,
        /// The type of reference.
        #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
        pub type_: Option<ReferenceType>,
    }

    /// A byte range within a source.
    #[derive(Clone, Debug, Serialize)]
    pub struct Range {
        /// The byte offset from the start of the source.
        pub offset: u64,
        /// The length in bytes.
        pub length: u64,
    }

    /// A source range (reference to a source with optional byte range).
    #[derive(Clone, Debug, Serialize)]
    pub struct SourceRange {
        /// Reference to the source.
        pub source: Reference,
        /// Optional byte range within the source.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub range: Option<Range>,
    }
}

/// Program-related structures for ETHDebug format.
pub mod program {
    use super::{data, materials, *};

    /// Context information for an instruction or program.
    #[derive(Clone, Debug, Serialize)]
    pub struct Context {
        /// The source code this context refers to.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub code: Option<materials::SourceRange>,
        /// Variables in scope.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub variables: Option<Vec<Variable>>,
        /// A remark about this context.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub remark: Option<String>,
    }

    /// A variable in the debug context.
    #[derive(Clone, Debug, Serialize)]
    pub struct Variable {
        /// The variable identifier/name.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub identifier: Option<String>,
        /// The source location of the variable declaration.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub declaration: Option<materials::SourceRange>,
    }

    /// An EVM instruction with debug information.
    #[derive(Clone, Debug, Serialize)]
    pub struct Instruction {
        /// The bytecode offset.
        pub offset: data::Unsigned,
        /// The instruction operation.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub operation: Option<Operation>,
        /// The context for this instruction.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub context: Option<Context>,
    }

    /// An EVM instruction operation.
    #[derive(Clone, Debug, Serialize)]
    pub struct Operation {
        /// The instruction mnemonic (e.g., "PUSH1", "MSTORE").
        pub mnemonic: String,
        /// The instruction arguments.
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub arguments: Vec<data::Unsigned>,
    }

    /// The execution environment.
    #[derive(Clone, Copy, Debug, Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Environment {
        /// Contract creation (constructor) environment.
        Create,
        /// Contract call (runtime) environment.
        Call,
    }

    /// Contract information in a program.
    #[derive(Clone, Debug, Serialize)]
    pub struct Contract {
        /// The contract name.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        /// The source range of the contract definition.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub definition: Option<materials::SourceRange>,
    }

    /// The kind of item in the contract.
    #[derive(Clone, Copy, Debug, Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum ItemKind {
        /// A function.
        Function,
        /// A state variable.
        Variable,
        /// A constructor.
        Constructor,
        /// A fallback function.
        Fallback,
        /// A receive function.
        Receive,
        /// A modifier.
        Modifier,
        /// An event.
        Event,
        /// An error.
        Error,
        /// A struct.
        Struct,
        /// An enum.
        Enum,
    }

    /// A contract item (function, variable, etc.) with source location.
    #[derive(Clone, Debug, Serialize)]
    pub struct Item {
        /// The kind of item.
        pub kind: ItemKind,
        /// The item name.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        /// The source range of the item.
        #[serde(rename = "sourceRange")]
        pub source_range: materials::SourceRange,
    }
}

/// Resources information (compilation metadata).
pub mod resources {
    use super::*;

    /// Compiler information.
    #[derive(Clone, Debug, Serialize)]
    pub struct Compiler {
        /// The compiler name.
        pub name: String,
        /// The compiler version.
        pub version: String,
    }

    /// Source file information.
    #[derive(Clone, Debug, Serialize)]
    pub struct Source {
        /// The source ID.
        pub id: u64,
        /// The source file path.
        pub path: String,
    }

    /// Compilation information.
    #[derive(Clone, Debug, Serialize)]
    pub struct Compilation {
        /// The compiler information.
        pub compiler: Compiler,
        /// The source files.
        pub sources: Vec<Source>,
    }
}

/// ETHDebug resources document (ethdebug/format/info/resources).
#[derive(Clone, Debug, Serialize)]
pub struct Resources {
    /// Compilation information.
    pub compilation: resources::Compilation,
}

/// ETHDebug program document (ethdebug/format/program).
#[derive(Clone, Debug, Serialize)]
pub struct Program {
    /// Contract information.
    pub contract: program::Contract,
    /// The execution environment.
    pub environment: program::Environment,
    /// Program-level context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<program::Context>,
    /// Contract items with source locations (Solar extension).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<program::Item>,
    /// Bytecode instructions with debug information.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub instructions: Vec<program::Instruction>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_unsigned_serialize() {
        let num = data::Unsigned::Numeric(42);
        assert_eq!(serde_json::to_string(&num).unwrap(), "42");

        let hex = data::Unsigned::Hex(data::HexValue(vec![0x12, 0x34]));
        assert_eq!(serde_json::to_string(&hex).unwrap(), "\"0x1234\"");
    }

    #[test]
    fn test_materials_id_serialize() {
        let num = materials::Id::Numeric(0);
        assert_eq!(serde_json::to_string(&num).unwrap(), "0");

        let str = materials::Id::String("test.sol".to_string());
        assert_eq!(serde_json::to_string(&str).unwrap(), "\"test.sol\"");
    }

    #[test]
    fn test_source_range_serialize() {
        let range = materials::SourceRange {
            source: materials::Reference { id: materials::Id::Numeric(0), type_: None },
            range: Some(materials::Range { offset: 10, length: 20 }),
        };
        let json = serde_json::to_string(&range).unwrap();
        assert!(json.contains("\"offset\":10"));
        assert!(json.contains("\"length\":20"));
    }

    #[test]
    fn test_program_serialize() {
        let program = Program {
            contract: program::Contract {
                name: Some("Test".to_string()),
                definition: Some(materials::SourceRange {
                    source: materials::Reference { id: materials::Id::Numeric(0), type_: None },
                    range: Some(materials::Range { offset: 0, length: 100 }),
                }),
            },
            environment: program::Environment::Create,
            context: None,
            items: vec![],
            instructions: vec![],
        };
        let json = serde_json::to_string_pretty(&program).unwrap();
        assert!(json.contains("\"name\": \"Test\""));
        assert!(json.contains("\"environment\": \"create\""));
    }

    #[test]
    fn test_resources_serialize() {
        let resources = Resources {
            compilation: resources::Compilation {
                compiler: resources::Compiler {
                    name: "solar".to_string(),
                    version: "0.1.0".to_string(),
                },
                sources: vec![resources::Source { id: 0, path: "test.sol".to_string() }],
            },
        };
        let json = serde_json::to_string_pretty(&resources).unwrap();
        assert!(json.contains("\"name\": \"solar\""));
        assert!(json.contains("\"path\": \"test.sol\""));
    }
}

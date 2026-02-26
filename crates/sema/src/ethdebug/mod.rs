//! ETHDebug format support.
//!
//! This module provides initial support for the [ETHDebug format](https://github.com/ethdebug/format),
//! which enables source-level debugging of Solidity smart contracts.
//!
//! Currently, this implementation provides source-level debug information without bytecode
//! instruction mappings, as Solar does not yet have bytecode generation.

pub mod schema;

use crate::{hir::ContractId, ty::Gcx};
use schema::{Program, Resources, materials, program};
use solar_interface::{Span, source_map::FileName};

/// Generates ETHDebug resources (compilation metadata).
pub fn resources(gcx: Gcx<'_>) -> Resources {
    let sources: Vec<_> = gcx
        .hir
        .source_ids()
        .map(|id| {
            let source = gcx.hir.source(id);
            let path = match &source.file.name {
                FileName::Real(path) => path.display().to_string(),
                FileName::Stdin => "<stdin>".to_string(),
                FileName::Custom(name) => name.clone(),
            };
            schema::resources::Source { id: id.index() as u64, path }
        })
        .collect();

    Resources {
        compilation: schema::resources::Compilation {
            compiler: schema::resources::Compiler {
                name: "solar".to_string(),
                version: solar_interface::config::version::SEMVER_VERSION.to_string(),
            },
            sources,
        },
    }
}

/// Generates ETHDebug program information for a contract.
///
/// Since Solar does not yet have bytecode generation, this provides source-level
/// debug information without bytecode instruction mappings.
pub fn contract_program(gcx: Gcx<'_>, contract_id: ContractId) -> Program {
    use crate::hir::FunctionKind;

    let contract = gcx.hir.contract(contract_id);
    let source_id = contract.source.index() as u64;

    // Get the contract definition range
    let definition = span_to_source_range(gcx, contract.span, source_id);

    // Collect source ranges for all items in the contract
    let mut items = Vec::new();

    // Process contract items
    for &item_id in contract.items {
        if let Some(func_id) = item_id.as_function() {
            let func = gcx.hir.function(func_id);
            // Skip auto-generated getter functions
            if func.is_getter() {
                continue;
            }
            if let Some(range) = span_to_source_range(gcx, func.span, source_id) {
                let kind = match func.kind {
                    FunctionKind::Constructor => program::ItemKind::Constructor,
                    FunctionKind::Fallback => program::ItemKind::Fallback,
                    FunctionKind::Receive => program::ItemKind::Receive,
                    FunctionKind::Modifier => program::ItemKind::Modifier,
                    FunctionKind::Function => program::ItemKind::Function,
                };
                items.push(program::Item {
                    kind,
                    name: func.name.map(|n| n.to_string()),
                    source_range: range,
                });
            }
        } else if let Some(var_id) = item_id.as_variable() {
            let var = gcx.hir.variable(var_id);
            if let Some(range) = span_to_source_range(gcx, var.span, source_id) {
                items.push(program::Item {
                    kind: program::ItemKind::Variable,
                    name: var.name.map(|n| n.to_string()),
                    source_range: range,
                });
            }
        }
    }

    // Add constructor, fallback, receive if they weren't already added via items
    // These are stored separately in the contract struct
    if let Some(ctor_id) = contract.ctor {
        let ctor = gcx.hir.function(ctor_id);
        // Check if not already in items (some contracts store it both places)
        let ctor_range = span_to_source_range(gcx, ctor.span, source_id);
        let already_exists = items.iter().any(|i| {
            matches!(i.kind, program::ItemKind::Constructor)
                && i.source_range.range.as_ref().map(|r| r.offset)
                    == ctor_range.as_ref().and_then(|sr| sr.range.as_ref().map(|r| r.offset))
        });
        if !already_exists && let Some(range) = ctor_range {
            items.push(program::Item {
                kind: program::ItemKind::Constructor,
                name: None,
                source_range: range,
            });
        }
    }

    if let Some(fallback_id) = contract.fallback {
        let fallback = gcx.hir.function(fallback_id);
        if !items.iter().any(|i| matches!(i.kind, program::ItemKind::Fallback))
            && let Some(range) = span_to_source_range(gcx, fallback.span, source_id)
        {
            items.push(program::Item {
                kind: program::ItemKind::Fallback,
                name: None,
                source_range: range,
            });
        }
    }

    if let Some(receive_id) = contract.receive {
        let receive = gcx.hir.function(receive_id);
        if !items.iter().any(|i| matches!(i.kind, program::ItemKind::Receive))
            && let Some(range) = span_to_source_range(gcx, receive.span, source_id)
        {
            items.push(program::Item {
                kind: program::ItemKind::Receive,
                name: None,
                source_range: range,
            });
        }
    }

    Program {
        contract: program::Contract { name: Some(contract.name.to_string()), definition },
        // Since we don't have bytecode yet, we default to CREATE environment
        environment: program::Environment::Create,
        context: None,
        items,
        // Empty instructions since we don't have bytecode yet
        instructions: vec![],
    }
}

/// Converts a span to an ETHDebug source range.
fn span_to_source_range(
    gcx: Gcx<'_>,
    span: Span,
    source_id: u64,
) -> Option<materials::SourceRange> {
    if span.is_dummy() {
        return None;
    }

    let source_map = gcx.sess.source_map();
    let range = source_map.span_to_range(span).ok()?;

    Some(materials::SourceRange {
        source: materials::Reference { id: materials::Id::Numeric(source_id), type_: None },
        range: Some(materials::Range {
            offset: range.start as u64,
            length: (range.end - range.start) as u64,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resources_serialization() {
        let resources = Resources {
            compilation: schema::resources::Compilation {
                compiler: schema::resources::Compiler {
                    name: "solar".to_string(),
                    version: "0.1.0".to_string(),
                },
                sources: vec![schema::resources::Source { id: 0, path: "test.sol".to_string() }],
            },
        };

        let json = serde_json::to_string_pretty(&resources).unwrap();
        assert!(json.contains("\"name\": \"solar\""));
        assert!(json.contains("\"path\": \"test.sol\""));
    }
}

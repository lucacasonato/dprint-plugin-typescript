mod module_specifiers;
use module_specifiers::*;

use swc_common::{Span, Spanned};
use std::cmp::Ordering;

use super::*;
use crate::configuration::*;

// a little rough, but good enough for our purposes

pub fn get_node_sorter_from_order<'a>(order: SortOrder) -> Option<Box<dyn Fn((usize, Option<&Node<'a>>), (usize, Option<&Node<'a>>), &Context<'a>) -> Ordering>> {
    // todo: how to reduce code duplication here?
    match order {
        SortOrder::Maintain => None,
        SortOrder::CaseInsensitive => Some(Box::new(|(a_index, a), (b_index, b), context| {
            let result = if is_import_or_export_declaration(&a) {
                cmp_optional_nodes(a, b, context, |a, b, context| cmp_module_specifiers(a.text(context), b.text(context), cmp_text_case_insensitive))
            } else {
                cmp_optional_nodes(a, b, context, |a, b, context| cmp_text_case_insensitive(a.text(context), b.text(context)))
            };
            if result == Ordering::Equal {
                a_index.cmp(&b_index)
            } else {
                result
            }
        })),
        SortOrder::CaseSensitive => Some(Box::new(|(a_index, a), (b_index, b), context| {
            let result = if is_import_or_export_declaration(&a) {
                cmp_optional_nodes(a, b, context, |a, b, context| cmp_module_specifiers(a.text(context), b.text(context), cmp_text_case_sensitive))
            } else {
                cmp_optional_nodes(a, b, context, |a, b, context| cmp_text_case_sensitive(a.text(context), b.text(context)))
            };
            if result == Ordering::Equal {
                a_index.cmp(&b_index)
            } else {
                result
            }
        })),
    }
}

fn cmp_optional_nodes<'a>(
    a: Option<&Node<'a>>,
    b: Option<&Node<'a>>,
    context: &Context<'a>,
    cmp_func: impl Fn(&dyn Ranged, &dyn Ranged, &Context<'a>) -> Ordering,
) -> Ordering {
    if let Some(a) = a {
        if let Some(b) = b {
            cmp_nodes(&a, &b, context, cmp_func)
        } else {
            Ordering::Greater
        }
    } else {
        if b.is_none() {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }
}

fn cmp_nodes<'a>(
    a: &Node<'a>,
    b: &Node<'a>,
    context: &Context<'a>,
    cmp_func: impl Fn(&dyn Ranged, &dyn Ranged, &Context<'a>) -> Ordering,
) -> Ordering {
    let a_nodes = get_comparison_nodes(a);
    let b_nodes = get_comparison_nodes(b);

    for (i, a) in a_nodes.iter().enumerate() {
        if let Some(b) = b_nodes.get(i) {
            let cmp_result = cmp_func(a, b, context);
            if cmp_result != Ordering::Equal {
                return cmp_result;
            }
        } else {
            return Ordering::Greater;
        }
    }

    if a_nodes.len() < b_nodes.len() {
        Ordering::Less
    } else {
        Ordering::Equal
    }
}

fn get_comparison_nodes<'a>(node: &Node<'a>) -> Vec<Span> {
    match node {
        Node::ImportNamedSpecifier(node) => {
            if let Some(imported) = &node.imported {
                vec![imported.span(), node.local.span()]
            } else {
                vec![node.local.span()]
            }
        },
        Node::ExportNamedSpecifier(node) => {
            if let Some(exported) = &node.exported {
                vec![node.orig.span(), exported.span()]
            } else {
                vec![node.orig.span()]
            }
        },
        Node::ImportDecl(node) => {
            vec![node.src.span()]
        },
        Node::NamedExport(node) => {
            vec![node.src.span()]
        },
        Node::ExportAll(node) => {
            vec![node.src.span()]
        },
        _ => {
            #[cfg(debug_assertions)]
            unimplemented!("Not implemented sort node.");
            #[cfg(not(debug_assertions))]
            vec![node.span_data()]
        }
    }
}

fn cmp_text_case_sensitive(a: &str, b: &str) -> Ordering {
    a.cmp(b)
}

fn cmp_text_case_insensitive(a: &str, b: &str) -> Ordering {
    let case_insensitive_result = a.to_lowercase().cmp(&b.to_lowercase());
    if case_insensitive_result == Ordering::Equal {
        cmp_text_case_sensitive(a, b)
    } else {
        case_insensitive_result
    }
}

fn is_import_or_export_declaration<'a>(node: &Option<&Node<'a>>) -> bool {
    match node {
        Some(Node::ImportDecl(_)) | Some(Node::NamedExport(_)) | Some(Node::ExportAll(_)) => {
            true
        },
        _ => false
    }
}

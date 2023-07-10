use itertools::Itertools;
use log::debug;
use typst::syntax::ast;
use typst::syntax::Span;
use typst::syntax::SyntaxKind::*;
use typst::syntax::{parse, LinkedNode};
// use Option::None;

mod config;
use config::Config;

mod writer;

use writer::Writer;

pub fn format(s: &str, _c: &Config) -> String {
    let init = parse(s);
    let root = LinkedNode::new(&init);
    let mut parents: Vec<LinkedNode> = vec![root];
    let mut writer = Writer::new(Config::default(), s.len());

    while !parents.is_empty() {
        let node = parents.pop().unwrap();
        let mut children = node.children().collect_vec();

        children.reverse();
        parents.append(&mut children);
        let oth = node.clone();
        writer.nodes.push(oth);

        writer.write_node(node);
    }
    writer.get_result()
}

#[cfg(test)]
mod tests;

// Heading => todo!(),
// Equation => {
// todo!("starts and ends with space or no space")
// }

// Math => todo!(),

// Destructuring => todo!(),
// DestructAssignment => todo!(),

// WhileLoop => todo!(),
// ForLoop => todo!(),

// Parenthesized => todo!(),

// Binary => todo!(),
// FieldAccess => todo!(),

// Array => todo!(),
// Dict => todo!(),
// Named => todo!(),
// Keyed => todo!(),

// Closure => todo!(),
// Params => todo!(),

// LetBinding => todo!(),
// SetRule => todo!(),
// ShowRule => todo!(),
// Conditional => todo!(),
// ModuleImport => todo!(),
// ImportItems => todo!(),

// ModuleInclude => todo!(),

// LeftBrace => todo!(),
// RightBrace => todo!(),
// LeftBracket => todo!(),
// RightBracket => todo!(),
// LeftParen => todo!(),

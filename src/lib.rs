use itertools::Itertools;
use log::debug;
use typst::syntax::SyntaxKind;
// use log::debug;
use typst::syntax::ast;
// use typst::syntax::Span;
use typst::syntax::SyntaxKind::*;
use typst::syntax::{parse, LinkedNode};
use Option::None;

mod config;
use config::Config;

#[derive(Default)]
struct Ctx {
    config: Config,
    just_spaced: bool,
    consec_new_line: i32,
}

impl Ctx {
    fn from_config(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// avoids:
    /// - putting two consecutive spaces.
    /// - putting more than two consecutive newlines.
    fn process<'a>(&mut self, s: &'a str) -> &'a str {
        match s {
            " " => {
                if self.just_spaced {
                    debug!("IGNORED space");
                    ""
                } else {
                    self.just_spaced = true;
                    s
                }
            }
            "\n" => {
                if self.consec_new_line <= 1 {
                    self.consec_new_line += 1;
                    s
                } else {
                    debug!("IGNORED newline");
                    ""
                }
            }
            _ => {
                debug!("PUSHED {s}");
                self.pushed_raw();
                s
            }
        }
    }
    /// makes the context aware it missed info,
    /// should be called when pushing directly in result.
    fn pushed_raw(&mut self) {
        self.just_spaced = false;
        self.consec_new_line = 0;
    }

    fn indent(&self) -> String {
        " ".repeat(self.config.ident_space)
    }
}

pub fn format(s: &str, config: Config) -> String {
    let init = parse(s);
    let mut context = Ctx::from_config(config);
    let root = LinkedNode::new(&init);
    visit(&root, &mut context)
}

fn visit(node: &LinkedNode, ctx: &mut Ctx) -> String {
    let mut res: Vec<String> = vec![];
    for child in node.children() {
        let child_fmt = visit(&child, ctx);
        res.push(child_fmt);
        ctx.pushed_raw()
    }
    match node.kind() {
        Args => format_args(node, &res, ctx),
        Space => String::from(" "),
        _ => format_default(node, &res, ctx),
    }
}

fn format_default(node: &LinkedNode, children: &Vec<String>, ctx: &mut Ctx) -> String {
    debug!("format_default");
    let mut res = String::new();

    match node.kind() {
        // Space => {
        //     for c in node.text().chars() {
        //         match c {
        //             ' '  => res.push_str(config.process(" ")),
        //             '\n' => res
        //         }
        //     }
        // },
        Parbreak => {
            debug!("format_default::ParBreak");
            for _ in 0..node.text().lines().count() {
                debug!("---try push newline");
                res.push_str(ctx.process("\n"))
            }
        }
        _ => {
            res.push_str(node.text());
            for k in children {
                res.push_str(&k);
                ctx.pushed_raw()
            }
        }
    }
    res
}

fn format_args(parent: &LinkedNode, children: &Vec<String>, ctx: &mut Ctx) -> String {
    let res = format_args_one_line(&children, parent, ctx);

    if max_line_length(&res) >= ctx.config.max_line_length {
        debug!("format_args::breaking");
        ctx.pushed_raw();
        return format_args_breaking(&children, parent, ctx);
    }
    debug!("format_args::one_line");
    res
}

fn format_args_one_line(children: &Vec<String>, parent: &LinkedNode<'_>, ctx: &mut Ctx) -> String {
    let mut res = String::new();
    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            Space => {}
            Comma => {
                if is_trailing_comma(&node) {
                    // don't print
                } else {
                    res.push_str(node.text());
                    res.push(' ');
                    ctx.pushed_raw()
                }
            }
            _ => {
                res.push_str(s);
                ctx.pushed_raw()
            }
        }
    }
    res
}

fn format_args_breaking(children: &Vec<String>, parent: &LinkedNode<'_>, ctx: &mut Ctx) -> String {
    let mut res = String::new();
    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            LeftParen => {
                res.push_str(node.text());
                res.push('\n');
                res.push_str(&ctx.indent());
            }
            Space => {}
            Comma => {
                // print the last comma but don't indent

                if is_last_comma(&node) && is_trailing_comma(&node) {
                    res.push_str(node.text());
                    res.push('\n');
                    ctx.pushed_raw()
                } else {
                    res.push_str(node.text());
                    res.push('\n');
                    res.push_str(&ctx.indent());
                    ctx.pushed_raw();
                }
            }
            _ => {
                // also cannot be a comma
                // so last and no trailing comma
                if next_is_ignoring(&node, RightParen, &[Space]) {
                    res.push_str(node.text());
                    res.push(',');
                    res.push('\n');
                    ctx.pushed_raw()
                } else {
                    res.push_str(s);
                    ctx.pushed_raw()
                }
            }
        }
    }
    res
}

fn next_is_ignoring(node: &LinkedNode, is: SyntaxKind, ignoring: &[SyntaxKind]) -> bool {
    let mut next = node.next_sibling();
    while let Some(next_inner) = &next {
        let kind = next_inner.kind(); 
        if ignoring.contains(&kind) {
            next = next_inner.next_sibling();
            continue
        }
        if kind == is {
            return true;
        } else {
            return false
        }
    }
    false
}

fn is_trailing_comma(node: &LinkedNode<'_>) -> bool {
    assert!(node.kind() == Comma);
    let next = node.next_sibling();
    let next_skipping_space = match &next {
        Some(x) if x.kind() == Space => next.unwrap().next_sibling(),
        _ => next,
    };
    next_skipping_space.is_some_and(|n| n.kind().is_terminator())
}

fn is_last_comma(node: &LinkedNode) -> bool {
    assert!(node.kind() == Comma);
    let mut next = node.next_sibling().unwrap();
    loop {
        if next.kind() == Comma {
            return false;
        }
        if next.kind().is_terminator() {
            return true;
        }
        next = next.next_sibling().unwrap();
    }
}

fn max_line_length(s: &str) -> usize {
    fn len_no_space(s: &str) -> usize {
        s.len() - s.chars().filter(|x| x == &' ').count()
    }
    let Some(last_line) = s.lines().last() else {
        if let Some(app) = s.lines().last() {
            println!("no last line");
           return len_no_space(app);
    } else {
            println!("no last line and no app lines");
            return 0;
        }
    };
    if !s.contains('\n') {
        len_no_space(last_line) + len_no_space(s)
    } else {
        len_no_space(s.split('\n').last().unwrap())
    }
}

#[cfg(test)]
mod tests;

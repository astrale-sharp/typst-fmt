use typst::syntax::SyntaxNode;
use super::*;


trait StrategyType : std::ops::FnOnce(&Writer, &[LinkedNode], &str) -> Amend {}
impl<T: > StrategyType for T where T : std::ops::FnOnce(&Writer, &[LinkedNode], &str) -> Amend{}


// type StrategyType = dyn std::ops::FnOnce(&Writer, &[LinkedNode], &str) -> Amend;
type BoxedStrategy = Box<dyn StrategyType>;
enum Amend {
    Do(String),
    Dont,
}

enum Condition {
    ParentIsNot(Span),
    EndFile
}


struct Mark {
    spans_idx: usize,
    result_idx: usize,
    strategy: BoxedStrategy,
    end_condition: Condition,
}

#[derive(Debug, Default)]
struct JustAdded {
    space: bool,
    newline: bool,
    ident: bool,
}

pub(crate) struct Writer<'a> {
    // if is some, simulation started at index
    // and is still going
    result: String,
    pub nodes: Vec<LinkedNode<'a>>,
    marks: Vec<Mark>,

    config: Config,
    just_added: JustAdded,
}

impl<'a> Writer<'a> {
    pub fn new(config: Config, len: usize) -> Self {
        Self {
            result: String::with_capacity(len),
            marks: vec![],
            nodes: vec![],
            config,
            just_added: JustAdded::default(),
        }
    }

    pub fn make_iter(init: &'a SyntaxNode) -> Vec<LinkedNode<'a>> {
        let root = LinkedNode::new(&init);
        let mut parents: Vec<LinkedNode> = vec![root];
        let mut linear = vec![];
        while !parents.is_empty() {
            let node = parents.pop().unwrap();
            let mut children = node.children().collect_vec();
            children.reverse();
            parents.append(&mut children);
            linear.push(node)
        }
        linear
    }

    pub fn write_node(&mut self, node: LinkedNode<'a>) {
        debug!("node kind: {:?}", node.kind());
        self.nodes.push(node.clone());

        self.check_marks();

        match node.kind() {
            Space => self.space(false),
            Linebreak | Eof | Parbreak => self.newline(false),

            Args => self.mark_here(
                node.next_sibling().map(|s| s.span()),
                Box::new(|_, _, _| Amend::Dont),
            ),

            Comma => self.write_comma(&node),
            // Colon => todo!(),
            _ => self.push(node.text()),
        }
    }

    fn mark_here(&mut self, end_condition: Condition, strategy: impl StrategyType) {
        self.marks.push(Mark {
            spans_idx: self.nodes.len(),
            result_idx: self.result.len(),
            strategy : Box::new(strategy),
            end_condition,
        })
    }

    fn mark(&mut self, mark: Mark) {
        self.marks.push(mark)
    }

    fn check_marks(&mut self) {
        let span = self.nodes.last().unwrap().span();
        let will_unwind = self.marks.last().map(|m| m.end_condition).flatten() == Some(span);
        if will_unwind {
            self.unwind_mark()
        }
    }

    fn unwind_mark(&mut self) {
        let mark = self.marks.pop().unwrap();
        let nodes = &self.nodes[mark.spans_idx..];
        let additions = &self.result[mark.result_idx..];
        let amend = (mark.strategy)(self, nodes, additions);
        if let Amend::Do(rep) = amend {
            debug!("AMENDING with {rep}");
            self.result = self.result[..mark.result_idx].to_string();
            self.result.push_str(&rep);
        }
        self.check_marks()
    }

    pub fn write_comma(&mut self, node: &LinkedNode) {
        self.push(node.text());
        if node.parent().unwrap().cast::<ast::Args>().is_some()
            && node.next_sibling().is_some_and(|x| !x.kind().is_grouping())
        {
            log::debug!("Args::Comma");
            self.space(false);
        }
    }

    pub fn get_result(mut self) -> String {
        // trigger condition
        while !&self.marks.is_empty() {
            debug!("UNWINDING");
            self.unwind_mark();
        }
        self.result
    }

    pub fn current_line_length(&self) -> usize {
        self.result
            .chars()
            .rev()
            .take_while(|x| x != &'\n')
            .collect_vec()
            .len()
    }

    pub fn space(&mut self, strong: bool) {
        debug!("PUSHED space");
        if !self.just_added.space || strong {
            self.result.push(' ');

            self.just_added = JustAdded::default();
            self.just_added.space = true
        }
    }

    pub fn ident(&mut self, strong: bool) {
        if !self.just_added.ident || strong {
            self.result
                .push_str(&" ".repeat(self.config.ident_space as _));

            self.just_added = JustAdded::default();
            self.just_added.ident = true;
        }
    }

    pub fn newline(&mut self, strong: bool) {
        if !self.just_added.newline || strong {
            debug!("PUSHED newline");
            self.result.push('\n');
            self.just_added.newline = true
        }
    }

    pub fn push(&mut self, s: &str) {
        debug!("PUSHED: {s:?}");
        self.result.push_str(s);
    }
}

#[cfg(test)]
mod tests {
    use typst::syntax::{parse_code, SyntaxNode};

    use super::*;

    #[test]
    fn test_unwind_replaces() {
        let nodes = (0..5)
            .map(|i| SyntaxNode::error("", i.to_string()))
            .collect_vec();
        let nodes = nodes.iter().map(LinkedNode::new).collect_vec();
        let mut writer = Writer {
            result: String::from("Hello, world!"),
            nodes,
            marks: vec![Mark {
                spans_idx: 3,
                result_idx: "Hello,".len(),
                end_condition: Some(Span::detached()),
                strategy: Box::new(move |writer, nodes, addition| {
                    assert_eq!(addition, " world!");
                    // assert!(c.len() == 2);
                    Amend::Do(String::from(" john!"))
                }),
            }],
            config: Config::default(),
            just_added: JustAdded::default(),
        };
        writer.unwind_mark();
        assert_eq!(writer.result, String::from("Hello, john!"))
    }

    #[test]
    fn test_unwind_args() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .unwrap();

        let n = parse_code("func(1,2,3)");
        let nodes = Writer::make_iter(&n);
        let mut w = Writer::new(Config::default(), 0);
        w.nodes = nodes.clone();
        let callback_was_called = std::rc::Rc::new(std::cell::RefCell::new(false));

        for lkn in &nodes {
            let oth_ref = callback_was_called.clone();
            match lkn.kind() {
                Args => w.mark_here(
                    Option::None,
                    Box::new(move |_, _, _| {
                        *oth_ref.borrow_mut() = true;
                        Amend::Dont
                    }),
                ),
                _ => w.push(lkn.text()),
            }
        }

        assert!(!&w.marks.is_empty());
        let _ = w.get_result();
        assert!(*callback_was_called.borrow());
    }

    #[test]
    fn visualize_make_iter() {
        let n = parse_code("func(1,2,3)");
        let v = Writer::make_iter(&n);
        let v = v.iter().map(|x| (x.kind(), x.text()));
        for (kind, text) in v {
            println!("{kind:?} -- {text:?}")
        }
    }
}

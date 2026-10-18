use super::rule::{Rule, RuleCond};
use super::{Predicate, Truth};
use crate::rules::RuleContext;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Skip<H> {
    FalseConjunct(H),
    TrueDisjunct(H),
    Terminal(usize),
}

pub(in crate::rules) trait NodeObserver {
    const TOTAL: bool;
    type Handle: Copy;

    fn enter(
        &mut self,
        node: Option<RuleCond>,
        parent: Option<Self::Handle>,
        skip: Option<Skip<Self::Handle>>,
    ) -> Self::Handle;

    fn finish(&mut self, handle: Self::Handle, truth: Truth);
}

pub(crate) struct NoTrace;

impl NodeObserver for NoTrace {
    const TOTAL: bool = false;
    type Handle = ();

    #[inline]
    fn enter(&mut self, _: Option<RuleCond>, _: Option<()>, _: Option<Skip<()>>) {}

    #[inline]
    fn finish(&mut self, (): (), _: Truth) {}
}

#[inline]
pub(in crate::rules) fn evaluate<O: NodeObserver>(
    condition: RuleCond,
    context: &RuleContext<'_>,
    observer: &mut O,
    parent: Option<O::Handle>,
    skip: Option<Skip<O::Handle>>,
) -> (Truth, O::Handle) {
    let handle = observer.enter(Some(condition), parent, skip);
    let truth = match condition {
        RuleCond::Leaf(leaf) => leaf.truth(context),
        RuleCond::All(children) => group(
            children,
            context,
            observer,
            (handle, skip),
            (Truth::True, Truth::False),
            Truth::and,
            Skip::FalseConjunct,
        ),
        RuleCond::Any(children) => group(
            children,
            context,
            observer,
            (handle, skip),
            (Truth::False, Truth::True),
            Truth::or,
            Skip::TrueDisjunct,
        ),
        RuleCond::Not(child) => !evaluate(*child, context, observer, Some(handle), skip).0,
        RuleCond::Named(named) => {
            evaluate(named.condition, context, observer, Some(handle), skip).0
        }
    };
    observer.finish(handle, truth);
    (truth, handle)
}

#[inline]
fn group<O: NodeObserver>(
    children: &[RuleCond],
    context: &RuleContext<'_>,
    observer: &mut O,
    (handle, skip): (O::Handle, Option<Skip<O::Handle>>),
    (identity, absorbing): (Truth, Truth),
    combine: fn(Truth, Truth) -> Truth,
    after: fn(O::Handle) -> Skip<O::Handle>,
) -> Truth {
    let mut truth = identity;
    let mut decided = None;
    for &child in children {
        let answer = if let (false, RuleCond::Leaf(leaf)) = (O::TOTAL, child) {
            leaf.truth(context)
        } else {
            let (answer, child_handle) = evaluate(
                child,
                context,
                observer,
                Some(handle),
                skip.or(decided.map(after)),
            );
            if decided.is_none() && answer == absorbing {
                decided = Some(child_handle);
            }
            answer
        };
        if answer == absorbing && !O::TOTAL {
            return answer;
        }
        truth = combine(truth, answer);
    }
    truth
}

#[inline]
pub(in crate::rules) fn evaluate_rule<O: NodeObserver>(
    rule: &Rule,
    context: &RuleContext<'_>,
    observer: &mut O,
    terminal: Option<usize>,
) -> (Truth, O::Handle) {
    let skip = terminal.map(Skip::Terminal);
    let root = observer.enter(None, None, skip);
    let truth = if O::TOTAL {
        let (audience, audience_handle) = evaluate(
            rule.audience.condition(),
            context,
            observer,
            Some(root),
            skip,
        );
        let skip =
            skip.or((audience == Truth::False).then_some(Skip::FalseConjunct(audience_handle)));
        audience.and(evaluate(rule.when, context, observer, Some(root), skip).0)
    } else {
        let audience = match rule.audience.condition() {
            RuleCond::All([]) => Truth::True,
            RuleCond::Leaf(leaf) => leaf.truth(context),
            RuleCond::Not(RuleCond::Leaf(leaf)) => !leaf.truth(context),
            condition @ (RuleCond::All(_)
            | RuleCond::Any(_)
            | RuleCond::Not(_)
            | RuleCond::Named(_)) => evaluate(condition, context, observer, Some(root), skip).0,
        };
        if audience == Truth::False {
            audience
        } else {
            audience.and(evaluate(rule.when, context, observer, Some(root), skip).0)
        }
    };
    observer.finish(root, truth);
    (truth, root)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Rule,
    Leaf(Predicate),
    All,
    Any,
    Not,
    Named(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Witness {
    pub node: usize,
    pub truth: Truth,
    pub negated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeTrace {
    pub node: usize,
    pub parent: Option<usize>,
    pub kind: NodeKind,
    pub truth: Truth,
    pub skip: Option<Skip<usize>>,
    pub witnesses: Vec<Witness>,
    pub default_witnesses: Vec<Witness>,
}

#[derive(Default)]
pub(in crate::rules) struct NodeTracer {
    pub(in crate::rules) nodes: Vec<NodeTrace>,
}

impl NodeObserver for NodeTracer {
    const TOTAL: bool = true;
    type Handle = usize;

    fn enter(
        &mut self,
        node: Option<RuleCond>,
        parent: Option<usize>,
        skip: Option<Skip<usize>>,
    ) -> usize {
        let kind = match node {
            None => NodeKind::Rule,
            Some(RuleCond::Leaf(leaf)) => NodeKind::Leaf(leaf),
            Some(RuleCond::All(_)) => NodeKind::All,
            Some(RuleCond::Any(_)) => NodeKind::Any,
            Some(RuleCond::Not(_)) => NodeKind::Not,
            Some(RuleCond::Named(named)) => NodeKind::Named(named.name),
        };
        let node = self.nodes.len();
        self.nodes.push(NodeTrace {
            node,
            parent,
            kind,
            truth: Truth::False,
            skip,
            witnesses: Vec::new(),
            default_witnesses: Vec::new(),
        });
        node
    }

    #[expect(
        clippy::indexing_slicing,
        reason = "`enter` returned `handle` as the index of the node it pushed"
    )]
    fn finish(&mut self, handle: usize, truth: Truth) {
        let kind = self.nodes[handle].kind;
        let children: Vec<(usize, Truth)> = self
            .nodes
            .iter()
            .filter(|child| child.parent == Some(handle))
            .map(|child| (child.node, child.truth))
            .collect();
        let negated = kind == NodeKind::Not;
        let conjunction = matches!(kind, NodeKind::All | NodeKind::Rule);
        let decides = |child: Truth| match (kind, truth) {
            (NodeKind::Leaf(_), _) => false,
            (NodeKind::Not | NodeKind::Named(_), _) => true,
            (_, Truth::Unknown { .. }) => matches!(child, Truth::Unknown { .. }),
            (_, Truth::False) if conjunction => child == Truth::False,
            (_, Truth::True) if !conjunction => child == Truth::True,
            (_, Truth::True | Truth::False) => true,
        };
        let witnesses = children
            .iter()
            .filter(|&&(_, child)| decides(child))
            .map(|&(node, child)| Witness {
                node,
                truth: child,
                negated,
            })
            .collect();
        let default_witnesses = match truth {
            Truth::Unknown { default, .. } => children
                .iter()
                .filter(|&&(_, child)| match kind {
                    NodeKind::Leaf(_) => false,
                    NodeKind::Not | NodeKind::Named(_) => true,
                    NodeKind::All | NodeKind::Rule | NodeKind::Any => {
                        child.resolves_true() == default
                    }
                })
                .map(|&(node, child)| Witness {
                    node,
                    truth: if child.resolves_true() {
                        Truth::True
                    } else {
                        Truth::False
                    },
                    negated,
                })
                .collect(),
            Truth::True | Truth::False => Vec::new(),
        };
        let node = &mut self.nodes[handle];
        node.truth = truth;
        node.witnesses = witnesses;
        node.default_witnesses = default_witnesses;
    }
}

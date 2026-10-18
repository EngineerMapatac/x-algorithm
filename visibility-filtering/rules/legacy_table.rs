use crate::rules::rule_spec::features::{PostFactsLeaf, RequestLeaf};
use crate::rules::rule_spec::rule::{Rule, RuleCond};
use crate::rules::rule_spec::Predicate;
use std::fmt::Write;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Unrepresentable {
    pub(super) rule: &'static str,
    pub(super) part: String,
}

pub(super) fn legacy_table(rules: &[(&'static str, Rule)]) -> Result<String, Unrepresentable> {
    let mut table = String::new();
    let mut severity = None;
    for &(name, ref rule) in rules {
        let unrepresentable = |part: RuleCond| Unrepresentable {
            rule: name,
            part: part.spelled(),
        };
        let action = rule.action.spec();
        if severity != Some(action.severity()) {
            severity = Some(action.severity());
            match action.kind_and_slot() {
                (kind, "withholding") => writeln!(table, "## {kind}, withholding"),
                (kind, slot) => writeln!(table, "## {kind}, {slot} slot"),
            }
            .unwrap();
        }
        let mut when = Vec::new();
        conjuncts(rule.when, &mut when).map_err(unrepresentable)?;
        let when = if when.is_empty() {
            "always".to_owned()
        } else {
            when.join(" & ")
        };
        writeln!(
            table,
            "{name} | {:?} | {when} | {}",
            rule.audience,
            action.spelled()
        )
        .unwrap();
    }
    Ok(table)
}

fn conjuncts(condition: RuleCond, conjuncts: &mut Vec<String>) -> Result<(), RuleCond> {
    let leaves = |branches: &[RuleCond]| -> Option<Vec<Predicate>> {
        branches
            .iter()
            .map(|branch| match *branch {
                RuleCond::Leaf(leaf) => Some(leaf),
                RuleCond::All(_) | RuleCond::Any(_) | RuleCond::Not(_) | RuleCond::Named(_) => None,
            })
            .collect()
    };
    match condition {
        RuleCond::Leaf(leaf) => conjuncts.push(leaf.spelled()),
        RuleCond::Not(&RuleCond::Leaf(leaf)) => conjuncts.push(negated(leaf)),
        RuleCond::All(children) => {
            for &child in children {
                self::conjuncts(child, conjuncts)?;
            }
        }
        RuleCond::Named(named) => match named.condition {
            RuleCond::All(_) => self::conjuncts(named.condition, conjuncts)?,
            RuleCond::Any(branches) => {
                let leaves = leaves(branches).ok_or(condition)?;
                let leaves: Vec<String> = leaves.into_iter().map(Predicate::spelled).collect();
                conjuncts.push(format!("{}[{}]", named.name, leaves.join(" | ")));
            }
            RuleCond::Not(&RuleCond::Any(branches)) => {
                conjuncts.extend(leaves(branches).ok_or(condition)?.into_iter().map(negated));
            }
            RuleCond::Leaf(_) | RuleCond::Not(_) | RuleCond::Named(_) => return Err(condition),
        },
        RuleCond::Any(_) | RuleCond::Not(_) => return Err(condition),
    }
    Ok(())
}

fn negated(leaf: Predicate) -> String {
    if let Predicate::PostFacts(PostFactsLeaf::TweetCreatedAfter(unix_ms)) = leaf {
        format!("tweet_created_before({unix_ms})")
    } else if leaf == Predicate::Request(RequestLeaf::ClientHasModernBlur) {
        "is_client_deprecated".to_owned()
    } else {
        format!("not({})", leaf.spelled())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::rule_spec::leaves::*;
    use crate::rules::rule_spec::rule::Audience::{Everyone, NonAuthors};
    use crate::rules::rule_spec::rule::{all, any, Audience, NamedCond};
    use crate::rules::rule_spec::{drop_post, DropAs, RuleId};

    #[test]
    fn the_legacy_table_rejects_a_rule_it_cannot_print() {
        static NESTED: NamedCond = NamedCond {
            name: "nested",
            condition: any![
                ViewerMutesAuthor,
                all![ViewerFollowsAuthor, ViewerBlocksAuthor]
            ],
        };
        let printed = |audience: Audience, when: RuleCond| {
            let rule = Rule {
                id: RuleId::FilterAll,
                audience,
                when,
                action: drop_post(DropAs::UnspecifiedReason),
            };
            legacy_table(&[("rule", rule)]).map_err(|error| error.part)
        };
        let nested_any = const { RuleCond::Not(&any![ViewerMutesAuthor, ViewerBlocksAuthor]) };
        for (audience, when, part) in [
            (
                Everyone,
                nested_any,
                "not(any[viewer_mutes_author | viewer_blocks_author])",
            ),
            (
                NonAuthors,
                const { all![ViewerFollowsAuthor, RuleCond::Named(&NESTED)] },
                "nested=any[viewer_mutes_author | all[viewer_follows_author & viewer_blocks_author]]",
            ),
        ] {
            assert_eq!(printed(audience, when), Err(part.to_owned()));
        }
        assert_eq!(
            printed(NonAuthors, const { all![ViewerFollowsAuthor] }),
            Ok("## drop, withholding\nrule | NonAuthors | viewer_follows_author | Legacy(UnspecifiedReason)\n".to_owned())
        );
    }
}

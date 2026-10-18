use crate::rules::rule_spec::rule::{NamedCond, Rule, RuleCond};
use crate::rules::rule_spec::rule_name;
use crate::rules::SafetyLevel;
use std::cmp::Reverse;
use std::fmt::Write;
use strum::VariantArray;

fn spelled(condition: RuleCond) -> String {
    condition.spelled_with(|named| named.name.to_owned())
}

pub(super) fn catalog() -> String {
    let mut rules: Vec<(&'static Rule, Vec<&'static str>)> = Vec::new();
    for &level in SafetyLevel::VARIANTS {
        for &rule in level.spec().rules {
            match rules.iter_mut().find(|(other, _)| *other == rule) {
                Some((_, levels)) => levels.push(level.into()),
                None => rules.push((rule, vec![level.into()])),
            }
        }
    }
    rules.sort_by_key(|(rule, _)| Reverse(rule.action.spec().severity()));
    let mut named: Vec<&'static NamedCond> = Vec::new();
    for (rule, _) in &rules {
        rule.when.named(&mut named);
    }

    let mut doc = String::from(
        "# Policy catalog\n\n\
         Generated from the rules in `src/rules/` by `scripts/bless-policy`, which\n\
         `rules::catalog::tests::the_policy_catalog_is_printed_from_the_rules` checks. Edit the\n\
         rules, not this file.\n\n\
         ## Precedence\n\n\
         A level folds its rules in this order of action and keeps its listed order within one\n\
         action. The first withholding decides; the slots fill as `GLOSSARY.md` (Slot) says.\n\n\
         | Action | Slot |\n|---|---|\n",
    );
    let mut kinds: Vec<(&str, &str)> = rules
        .iter()
        .map(|(rule, _)| rule.action.spec().kind_and_slot())
        .collect();
    kinds.dedup();
    for (kind, slot) in kinds {
        writeln!(doc, "| {kind} | {slot} |").unwrap();
    }
    doc.push_str("\n## Named conditions\n\n");
    for definition in &named {
        let uses = rules
            .iter()
            .filter(|(rule, _)| {
                let mut read = Vec::new();
                rule.when.named(&mut read);
                read.contains(definition)
            })
            .count();
        writeln!(
            doc,
            "- `{}` = `{}`. Uses: {uses}.",
            definition.name,
            spelled(definition.condition)
        )
        .unwrap();
    }
    let mut band = None;
    for (rule, levels) in &rules {
        let (kind, _) = rule.action.spec().kind_and_slot();
        if band != Some(kind) {
            band = Some(kind);
            writeln!(doc, "\n## {kind} rules\n").unwrap();
        }
        writeln!(
            doc,
            "- `{}`: audience `{:?}`, when `{}`, `{}`. Levels: {}.",
            rule_name(rule.id, &rule.action),
            rule.audience,
            spelled(rule.when),
            rule.action.spec().spelled(),
            levels.join(", ")
        )
        .unwrap();
    }
    doc
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::{env, fs};

    #[test]
    fn the_policy_catalog_is_printed_from_the_rules() {
        let catalog = catalog();
        if catalog != include_str!("../../docs/policy-catalog.md") {
            if let Ok(dir) = env::var("TEST_UNDECLARED_OUTPUTS_DIR") {
                fs::write(Path::new(&dir).join("policy-catalog.md"), &catalog).unwrap();
            }
            panic!("docs/policy-catalog.md differs from the rules: run scripts/bless-policy");
        }
    }
}

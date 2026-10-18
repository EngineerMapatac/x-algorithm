use std::process::ExitCode;

use xai_abuse_enforcement_service::rules::CompiledRules;

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: validate-rules-yaml <path-to-rules.yaml>");
        return ExitCode::FAILURE;
    };
    let src = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    match CompiledRules::from_yaml(&src) {
        Ok(compiled) => {
            let ids: Vec<&str> = compiled.rule_ids().collect();
            println!(
                "OK: {path} — for_entity={:?}, {} rule(s): {}",
                compiled.entity_type(),
                ids.len(),
                ids.join(", "),
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("INVALID: {path}: {e}");
            ExitCode::FAILURE
        }
    }
}

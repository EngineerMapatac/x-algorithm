// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn find_protoc() -> Option<PathBuf> {
    fn usable(protoc: &Path) -> bool {
        let out = match Command::new(protoc).arg("--version").output() {
            Ok(out) if out.status.success() => out,
            _ => return false,
        };
        let ver = String::from_utf8_lossy(&out.stdout);
        let mut nums = ver
            .split_whitespace()
            .last()
            .unwrap_or("")
            .split('.')
            .filter_map(|p| p.parse::<u32>().ok());
        match (nums.next(), nums.next()) {
            (Some(major), minor) => major > 3 || (major == 3 && minor.unwrap_or(0) >= 15),
            _ => false,
        }
    }

    if let Ok(protoc_env) = env::var("PROTOC") {
        let protoc = PathBuf::from(&protoc_env);
        if protoc.exists() {
            return Some(protoc);
        }
    }

    let mut dir = env::current_dir().ok()?;
    let mut dir_rel = PathBuf::new();
    loop {
        let protoc = dir_rel.join("bin/protoc");
        if protoc.exists() {
            if usable(&protoc) {
                return Some(protoc);
            }
            break;
        }
        if !dir.pop() {
            break;
        }
        dir_rel.push("..");
    }

    let path_protoc = Path::new("protoc");
    if usable(path_protoc) {
        return Some(path_protoc.to_path_buf());
    }
    None
}

fn compile(
    protos: &[&str],
    descriptor: &str,
    default_stubs: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let mut config = prost_build::Config::new();
    config.enable_type_names();
    if let Some(protoc) = find_protoc() {
        config.protoc_executable(protoc);
    }
    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join(descriptor))
        .generate_default_stubs(default_stubs)
        .bytes(".")
        .compile_with_config(config, protos, &["proto/"])?;
    Ok(())
}

fn main() {
    compile(
        &[
            "proto/recsys.proto",
            "proto/recsys_features.proto",
            "proto/served_feed.proto",
        ],
        "recsys_descriptor.bin",
        false,
    )
    .unwrap();
}

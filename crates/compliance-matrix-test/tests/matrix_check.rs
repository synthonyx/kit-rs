//! Verifies `docs/compliance.md` stays in sync with the on-disk
//! `compliance_*.rs` / `proptest_*.rs` test files across the workspace.
//!
//! Fails `cargo test` with a structured diff if either:
//! - a path listed in the matrix does not exist on disk, or
//! - a `crates/*/tests/compliance_*.rs` or `crates/*/tests/proptest_*.rs` on
//!   disk is not referenced in the matrix.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = <root>/crates/compliance-matrix-test
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn matrix_in_sync_with_disk() {
    let root = workspace_root();
    let matrix_path = root.join("docs/compliance.md");
    let matrix = fs::read_to_string(&matrix_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", matrix_path.display()));

    let re =
        regex::Regex::new(r"crates/[a-z][a-z0-9_-]*/tests/(compliance|proptest)_[a-z0-9_]+\.rs")
            .unwrap();

    let referenced: BTreeSet<String> = re
        .find_iter(&matrix)
        .map(|m| m.as_str().to_string())
        .collect();

    let mut on_disk: BTreeSet<String> = BTreeSet::new();
    for entry in fs::read_dir(root.join("crates")).expect("read crates/") {
        let crate_dir = entry.expect("crate entry").path();
        let tests_dir = crate_dir.join("tests");
        if !tests_dir.is_dir() {
            continue;
        }
        for f in fs::read_dir(&tests_dir).expect("read tests/") {
            let path = f.expect("tests entry").path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let is_target = (name.starts_with("compliance_") || name.starts_with("proptest_"))
                && name.ends_with(".rs");
            if !is_target {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .expect("under root")
                .to_string_lossy()
                .replace('\\', "/");
            on_disk.insert(rel);
        }
    }

    let missing_on_disk: Vec<_> = referenced.difference(&on_disk).cloned().collect();
    let orphan_in_matrix: Vec<_> = on_disk.difference(&referenced).cloned().collect();

    if !missing_on_disk.is_empty() || !orphan_in_matrix.is_empty() {
        let mut msg = String::from("docs/compliance.md is out of sync with test files on disk.\n");
        if !missing_on_disk.is_empty() {
            msg.push_str("\n  Referenced in matrix but missing on disk:\n");
            for p in &missing_on_disk {
                msg.push_str(&format!("    - {p}\n"));
            }
        }
        if !orphan_in_matrix.is_empty() {
            msg.push_str("\n  Present on disk but not referenced in matrix:\n");
            for p in &orphan_in_matrix {
                msg.push_str(&format!("    - {p}\n"));
            }
        }
        panic!("{}", msg);
    }
}

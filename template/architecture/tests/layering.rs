//! Architecture ("ArchUnit-style") tests enforcing the layering table in `AGENTS.md`.
//!
//! In Rust, a crate cannot reference a symbol from another crate unless it's listed as a
//! direct dependency in its `Cargo.toml` — so unlike Java/ArchUnit (which inspects
//! compiled bytecode for illegal imports), checking the *declared* dependency graph via
//! `cargo metadata` is already equivalent to checking actual usage. If one of these
//! tests fails, either the change violates the architecture, or the architecture
//! genuinely needs to change — in which case update both the code and `AGENTS.md`'s
//! layering table together.

use cargo_metadata::{MetadataCommand, Package, PackageId};
use std::collections::HashMap;

/// Maps a workspace crate to its architectural role by the name of the directory its
/// `Cargo.toml` lives in — stable across `crate_prefix` renames, unlike the crate name.
fn role_of(pkg: &Package) -> Option<&'static str> {
    let dir = pkg.manifest_path.parent()?.file_name()?;
    Some(match dir {
        "domain" => "domain",
        "infrastructure" => "infrastructure",
        "services" => "services",
        "rest_api" => "rest_api",
        "migration" => "migration",
        "launcher" => "launcher",
        _ => return None,
    })
}

/// Dependency identities (local roles or external crate names) a given role must never
/// depend on, directly or transitively through another local crate.
fn forbidden_for(role: &str) -> &'static [&'static str] {
    match role {
        "domain" => &[
            "infrastructure",
            "services",
            "rest_api",
            "launcher",
            "migration",
            "stano-axum",
            "stano-starter-rest",
            "stano-launcher",
            "stano-seaorm",
        ],
        "infrastructure" => &[
            "services",
            "rest_api",
            "launcher",
            "stano-axum",
            "stano-starter-rest",
            "stano-launcher",
        ],
        "services" => &[
            "infrastructure",
            "rest_api",
            "launcher",
            "stano-axum",
            "stano-starter-rest",
            "stano-launcher",
        ],
        "rest_api" => &["launcher", "stano-launcher"],
        "migration" => &["domain", "infrastructure", "services", "rest_api", "launcher"],
        "launcher" => &[],
        _ => &[],
    }
}

fn identity<'a>(
    id: &PackageId,
    role_by_id: &HashMap<PackageId, &'static str>,
    packages: &'a [Package],
) -> &'a str {
    if let Some(role) = role_by_id.get(id) {
        return role;
    }
    packages
        .iter()
        .find(|p| &p.id == id)
        .map(|p| p.name.as_str())
        .unwrap_or("<unknown>")
}

#[test]
fn layering_rules_are_not_violated() {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("`cargo metadata` succeeds");

    let role_by_id: HashMap<PackageId, &'static str> = metadata
        .workspace_members
        .iter()
        .filter_map(|id| {
            let pkg = metadata.packages.iter().find(|p| &p.id == id)?;
            role_of(pkg).map(|role| (id.clone(), role))
        })
        .collect();

    let resolve = metadata
        .resolve
        .as_ref()
        .expect("resolve graph is present (run via `cargo test`, not `cargo metadata --no-deps`)");

    let mut violations = Vec::new();
    for node in &resolve.nodes {
        let Some(&from_role) = role_by_id.get(&node.id) else {
            continue;
        };
        let forbidden = forbidden_for(from_role);
        for dep in &node.deps {
            let dep_identity = identity(&dep.pkg, &role_by_id, &metadata.packages);
            if forbidden.contains(&dep_identity) {
                violations.push(format!("{from_role} must not depend on {dep_identity}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Architecture layering violations found:\n{}",
        violations.join("\n")
    );
}

#[test]
fn launcher_wires_every_layer() {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("`cargo metadata` succeeds");

    let role_by_id: HashMap<PackageId, &'static str> = metadata
        .workspace_members
        .iter()
        .filter_map(|id| {
            let pkg = metadata.packages.iter().find(|p| &p.id == id)?;
            role_of(pkg).map(|role| (id.clone(), role))
        })
        .collect();

    let resolve = metadata.resolve.as_ref().expect("resolve graph is present");

    let launcher_id = role_by_id
        .iter()
        .find(|(_, role)| **role == "launcher")
        .map(|(id, _)| id.clone())
        .expect("a launcher crate exists");

    let launcher_node = resolve
        .nodes
        .iter()
        .find(|n| n.id == launcher_id)
        .expect("launcher has a resolve node");

    let wired_roles: Vec<&'static str> = launcher_node
        .deps
        .iter()
        .filter_map(|dep| role_by_id.get(&dep.pkg).copied())
        .collect();

    for expected in ["domain", "infrastructure", "services", "rest_api"] {
        assert!(
            wired_roles.contains(&expected),
            "launcher must directly depend on the {expected} crate — found: {wired_roles:?}"
        );
    }
}

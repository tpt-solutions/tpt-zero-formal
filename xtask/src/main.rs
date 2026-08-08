//! Developer task runner for the `tpt-zero-formal` workspace.
//!
//! Run with `cargo xtask <subcommand>` (or `cargo run -p xtask -- <subcommand>`).
//!
//! Subcommands:
//! - `check-readmes`      — every crate has a README.md containing a ``` rust block.
//! - `check-consistency`  — published crates carry docs.rs metadata + rust-version.
//! - `check-nostd`        — `cargo build --workspace --no-default-features` succeeds.
//! - `publish-order`      — topological publish order (respecting `publish = false`).
//! - `gen-graph`          — emit a mermaid dependency graph.
//! - `gen-type-level`     — regenerate `out-zero-type-level/src/generated.rs`.
//! - `new-crate <name>`   — scaffold a new crate from `template/`.
//! - `publish`            — publish every published crate in topological order.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("help");
    let code = match cmd {
        "check-readmes" => check_readmes(),
        "check-consistency" => check_consistency(),
        "check-nostd" => check_nostd(),
        "publish-order" => publish_order(),
        "gen-graph" => gen_graph(),
        "gen-type-level" => gen_type_level(),
        "publish" => publish(),
        "new-crate" => {
            let name = args.get(1).expect("new-crate needs a crate name");
            new_crate(name)
        }
        "help" => print_help(),
        _ => print_help(),
    };
    std::process::exit(code);
}

fn print_help() -> i32 {
    println!("subcommands: check-readmes, check-consistency, check-nostd, publish-order, gen-graph, gen-type-level, publish, new-crate <name>");
    0
}

fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("crates")
}

/// Returns (crate dir, parsed Cargo.toml text) for every member crate.
fn all_crates() -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    let dir = crates_dir();
    for entry in fs::read_dir(&dir).expect("crates dir") {
        let path = entry.unwrap().path();
        if !path.is_dir() {
            continue;
        }
        let manifest = path.join("Cargo.toml");
        if !manifest.exists() {
            continue;
        }
        let text = fs::read_to_string(&manifest).unwrap();
        out.push((path, text));
    }
    out
}

fn is_published(text: &str) -> bool {
    !text.contains("publish = false")
}

fn check_readmes() -> i32 {
    let mut bad = 0;
    for (path, _) in all_crates() {
        let readme = path.join("README.md");
        if !readme.exists() {
            println!("MISSING README: {}", path.file_name().unwrap().to_string_lossy());
            bad += 1;
            continue;
        }
        let body = fs::read_to_string(&readme).unwrap();
        if !body.contains("```rust") {
            println!(
                "NO rust example in README: {}",
                path.file_name().unwrap().to_string_lossy()
            );
            bad += 1;
        }
    }
    if bad == 0 {
        println!("check-readmes: OK");
        0
    } else {
        println!("check-readmes: {bad} problem(s)");
        1
    }
}

fn check_consistency() -> i32 {
    let mut bad = 0;
    for (path, text) in all_crates() {
        if !is_published(&text) {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        for key in [
            "rust-version",
            "[package.metadata.docs.rs]",
            "all-features = true",
        ] {
            if !text.contains(key) {
                println!("MISSING `{key}` in {name}");
                bad += 1;
            }
        }
    }
    if bad == 0 {
        println!("check-consistency: OK");
        0
    } else {
        println!("check-consistency: {bad} problem(s)");
        1
    }
}

fn check_nostd() -> i32 {
    let status = Command::new("cargo")
        .args(["build", "--workspace", "--no-default-features"])
        .status()
        .expect("cargo");
    if status.success() {
        println!("check-nostd: OK");
        0
    } else {
        println!("check-nostd: FAILED");
        1
    }
}

/// Build a dependency graph from path dependencies within the workspace and
/// emit a topological order (published crates only for publish ordering).
fn dependency_graph() -> (Vec<String>, std::collections::HashMap<String, Vec<String>>) {
    let mut nodes = Vec::new();
    let mut edges: std::collections::HashMap<String, Vec<String>> = Default::default();
    for (path, text) in all_crates() {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        nodes.push(name.clone());
        let mut deps = Vec::new();
        let mut in_deps = false;
        for line in text.lines() {
            let t = line.trim_start();
            if t.starts_with('[') {
                in_deps = t.starts_with("[dependencies") || t.starts_with("[dev-dependencies");
                continue;
            }
            if in_deps && t.contains("path =") {
                // path = "../crates/foo" or "../foo"
                if let Some(start) = t.find("crates/") {
                    let rest = &t[start + 7..];
                    let dep = rest.split(['"', '\'']).next().unwrap_or("").to_string();
                    if !dep.is_empty() {
                        deps.push(dep);
                    }
                }
            }
        }
        edges.insert(name, deps);
    }
    (nodes, edges)
}

fn publish_order() -> i32 {
    let (nodes, edges) = dependency_graph();
    let published: Vec<String> = nodes
        .iter()
        .filter(|n| is_published(&read_crate(n)))
        .cloned()
        .collect();
    match topo_sort(&published, &edges) {
        Some(order) => {
            for n in order {
                println!("{n}");
            }
            0
        }
        None => {
            println!("publish-order: cycle detected");
            1
        }
    }
}

fn read_crate(name: &str) -> String {
    fs::read_to_string(crates_dir().join(name).join("Cargo.toml")).unwrap_or_default()
}

fn topo_sort(
    nodes: &[String],
    edges: &std::collections::HashMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    let mut visited: std::collections::HashSet<String> = Default::default();
    let mut temp: std::collections::HashSet<String> = Default::default();
    let mut result = Vec::new();
    fn visit(
        n: &str,
        edges: &std::collections::HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        temp: &mut std::collections::HashSet<String>,
        result: &mut Vec<String>,
    ) -> bool {
        if visited.contains(n) {
            return true;
        }
        if temp.contains(n) {
            return false;
        }
        temp.insert(n.to_string());
        if let Some(deps) = edges.get(n) {
            for d in deps {
                if !visit(d, edges, visited, temp, result) {
                    return false;
                }
            }
        }
        temp.remove(n);
        visited.insert(n.to_string());
        result.push(n.to_string());
        true
    }
    for n in nodes {
        if !visit(n, edges, &mut visited, &mut temp, &mut result) {
            return None;
        }
    }
    Some(result)
}

fn gen_graph() -> i32 {
    let (nodes, edges) = dependency_graph();
    println!("```mermaid");
    println!("graph TD");
    for n in &nodes {
        if let Some(deps) = edges.get(n) {
            for d in deps {
                println!("  {d} --> {n}");
            }
        }
    }
    println!("```");
    0
}

fn gen_type_level() -> i32 {
    let dir = crates_dir().join("out-zero-type-level").join("src");
    let out = dir.join("generated.rs");
    let max: usize = 31;
    let mut s = String::new();
    s.push_str("//! Auto-generated by a generator; do not edit by hand.\n");
    s.push_str("//! Provides type-level operation impls for U<N> with N in 0..=31.\n");
    s.push_str("#![allow(missing_docs)]\n\n");
    s.push_str("use super::{Add, Sub, Mul, Min, Max, AssertLe, U};\n\n");
    for a in 0..=max {
        for b in 0..=max {
            s.push_str(&format!(
                "impl Add<U<{b}>> for U<{a}> {{ type Output = U<{}>; const VALUE: usize = {}; }}\n",
                a + b,
                a + b
            ));
            if a >= b {
                s.push_str(&format!(
                    "impl Sub<U<{b}>> for U<{a}> {{ type Output = U<{}>; const VALUE: usize = {}; }}\n",
                    a - b,
                    a - b
                ));
            }
            s.push_str(&format!(
                "impl Mul<U<{b}>> for U<{a}> {{ type Output = U<{}>; const VALUE: usize = {}; }}\n",
                a * b,
                a * b
            ));
            let mn = a.min(b);
            let mx = a.max(b);
            s.push_str(&format!(
                "impl Min<U<{b}>> for U<{a}> {{ type Output = U<{mn}>; const VALUE: usize = {mn}; }}\n"
            ));
            s.push_str(&format!(
                "impl Max<U<{b}>> for U<{a}> {{ type Output = U<{mx}>; const VALUE: usize = {mx}; }}\n"
            ));
            if a <= b {
                s.push_str(&format!("impl AssertLe<{b}> for U<{a}> {{}}\n"));
            }
        }
    }
    fs::write(&out, s).unwrap();
    println!("gen-type-level: wrote {}", out.display());
    0
}

fn publish() -> i32 {
    let (nodes, edges) = dependency_graph();
    let ordered = match topo_sort(&nodes, &edges) {
        Some(o) => o,
        None => {
            println!("publish: cycle detected");
            return 1;
        }
    };
    let mut failures = 0;
    for name in ordered {
        let text = read_crate(&name);
        if !is_published(&text) {
            println!("publish: skip {name} (publish = false)");
            continue;
        }
        println!("publish: cargo publish -p {name}");
        let status = Command::new("cargo")
            .args(["publish", "-p", &name])
            .status()
            .expect("cargo");
        if !status.success() {
            println!("publish: FAILED for {name}");
            failures += 1;
        }
    }
    if failures == 0 {
        println!("publish: all crates published");
        0
    } else {
        1
    }
}

fn new_crate(name: &str) -> i32 {
    let lib_name = name.replace('-', "_");
    let template = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("template");
    let dest = crates_dir().join(name);
    if dest.exists() {
        println!("new-crate: {name} already exists");
        return 1;
    }
    copy_dir(&template, &dest);
    // Substitute placeholders.
    let subs = [
        ("{{crate_name}}", name),
        ("{{lib_name}}", &lib_name),
        (
            "{{description}}",
            "Zero-dependency, no_std building block for the tpt-zero-formal ecosystem.",
        ),
    ];
    rewrite_placeholders(&dest, &subs);
    println!("new-crate: created {name} (lib name {lib_name})");
    0
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let p = entry.unwrap().path();
        let name = p.file_name().unwrap();
        let target = to.join(name);
        if p.is_dir() {
            copy_dir(&p, &target);
        } else {
            fs::copy(&p, &target).unwrap();
        }
    }
}

fn rewrite_placeholders(dir: &Path, subs: &[(&str, &str)]) {
    for entry in fs::read_dir(dir).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            rewrite_placeholders(&p, subs);
        } else {
            let text = fs::read_to_string(&p).unwrap_or_default();
            if subs.iter().any(|(k, _)| text.contains(k)) {
                let mut out = text;
                for (k, v) in subs {
                    out = out.replace(k, v);
                }
                fs::write(&p, out).unwrap();
            }
        }
    }
}

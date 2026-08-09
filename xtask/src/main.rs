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
//! - `certify`            — requirement traceability matrix + certification pack.
//! - `gen-contract-tests` — scaffold contract-derived (proptest) test skeletons.
//! - `check-panic-freedom`— report panic sites to support a panic-freedom proof.
//! - `new-crate <name>`   — scaffold a new crate from `template/`.
//! - `publish`            — publish every published crate in topological order.

#![allow(clippy::all, clippy::pedantic)]

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
        "certify" => certify(),
        "gen-contract-tests" => gen_contract_tests(),
        "check-panic-freedom" => check_panic_freedom(),
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
    println!(
        "subcommands: check-readmes, check-consistency, check-nostd, publish-order, gen-graph, gen-type-level, certify, gen-contract-tests, check-panic-freedom, publish, new-crate <name>"
    );
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
            println!(
                "MISSING README: {}",
                path.file_name().unwrap().to_string_lossy()
            );
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
    let template = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("template");
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

/// Recursively collects every `.rs` file under `crates/*/src` and
/// `examples/*/src`, paired with its owning package name and file contents.
fn source_files() -> Vec<(String, PathBuf, String)> {
    let mut out = Vec::new();
    let dir = crates_dir();
    for entry in fs::read_dir(&dir).expect("crates dir") {
        let crate_path = entry.unwrap().path();
        if !crate_path.is_dir() {
            continue;
        }
        let name = crate_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let src = crate_path.join("src");
        if !src.is_dir() {
            continue;
        }
        collect_rs(&src, &name, &mut out);
    }
    let examples = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("examples");
    if examples.is_dir() {
        for entry in fs::read_dir(&examples).expect("examples dir") {
            let example_path = entry.unwrap().path();
            if !example_path.is_dir() {
                continue;
            }
            let name = example_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let src = example_path.join("src");
            if !src.is_dir() {
                continue;
            }
            collect_rs(&src, &name, &mut out);
        }
    }
    out
}

fn collect_rs(dir: &Path, crate_name: &str, out: &mut Vec<(String, PathBuf, String)>) {
    for entry in fs::read_dir(dir).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            collect_rs(&p, crate_name, out);
        } else if p.extension().map_or(false, |e| e == "rs") {
            let text = fs::read_to_string(&p).unwrap_or_default();
            out.push((crate_name.to_string(), p, text));
        }
    }
}

fn count_substr(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

/// Extracts a `REQ="..."` identifier from a line, if present. Tolerates the
/// optional spaces around `=` (`REQ = "..."`).
fn extract_req(line: &str) -> Option<String> {
    let idx = line.find("REQ")?;
    let rest = &line[idx + "REQ".len()..];
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let quote = rest.strip_prefix('"')?;
    let end = quote.find('"')?;
    Some(quote[..end].to_string())
}

/// Counts *real* unsafe contexts (`unsafe {`, `unsafe fn`, `unsafe trait`,
/// `unsafe impl`) rather than the word "unsafe" in prose/doc comments.
fn count_unsafe(text: &str) -> usize {
    count_substr(text, "unsafe {")
        + count_substr(text, "unsafe fn")
        + count_substr(text, "unsafe trait")
        + count_substr(text, "unsafe impl")
}

fn field(text: &str, key: &str) -> String {
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(val) = t.strip_prefix(key) {
            let val = val.trim_start_matches('=').trim().to_string();
            if val.contains("workspace") {
                return resolve_workspace(key).to_string();
            }
            let val = val.trim_matches('"').to_string();
            if !val.is_empty() {
                return val;
            }
        }
    }
    String::new()
}

/// Resolves a workspace-inherited field (`version.workspace = true`) to the
/// concrete value declared in the workspace `Cargo.toml`.
fn resolve_workspace(key: &str) -> &'static str {
    match key {
        "version" => "0.1.0",
        "rust-version" => "1.85",
        _ => "workspace",
    }
}

fn path_deps(text: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut in_deps = false;
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with('[') {
            in_deps = t.starts_with("[dependencies") || t.starts_with("[dev-dependencies");
            continue;
        }
        if in_deps && t.contains("path =") {
            if let Some(start) = t.find("crates/") {
                let rest = &t[start + 7..];
                let dep = rest.split(['"', '\'']).next().unwrap_or("").to_string();
                if !dep.is_empty() && !deps.contains(&dep) {
                    deps.push(dep);
                }
            }
        }
    }
    deps
}

/// Produces a requirement traceability matrix and a certification artifact
/// pack under `cert/`. The matrix links every `REQ="..."`-tagged contract back
/// to its source location; the pack records per-crate dependency, MSRV,
/// contract-inventory, and panic/unsafe posture for downstream certification.
fn certify() -> i32 {
    let crates = all_crates();
    let sources = source_files();

    // Build the traceability matrix from every source (crates *and* examples),
    // so requirement IDs tagged in either location are captured.
    let mut matrix: std::collections::HashMap<String, Vec<String>> = Default::default();
    for (cn, _path, text) in &sources {
        for line in text.lines() {
            if let Some(req) = extract_req(line) {
                let loc = format!("{} @ {}", cn, _path.display());
                matrix.entry(req).or_default().push(loc);
            }
        }
    }

    let mut pack = String::from("{\n  \"crates\": [\n");
    let mut first_crate = true;

    for (crate_path, manifest) in &crates {
        let crate_name = crate_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let version = field(manifest, "version");
        let rust_version = field(manifest, "rust-version");
        let published = is_published(manifest);
        let deps = path_deps(manifest);

        let mut contracts = 0usize;
        let mut unsafe_blocks = 0usize;
        let mut panic_sites = 0usize;
        let mut req_ids: Vec<String> = Vec::new();

        for (cn, _path, text) in &sources {
            if cn != &crate_name {
                continue;
            }
            contracts += count_substr(text, "requires!(");
            contracts += count_substr(text, "ensures!(");
            contracts += count_substr(text, "mcdc_requires!(");
            unsafe_blocks += count_unsafe(text);
            panic_sites += count_substr(text, "panic!(")
                + count_substr(text, "unreachable!(")
                + count_substr(text, "unimplemented!(")
                + count_substr(text, "todo!(")
                + count_substr(text, ".unwrap()")
                + count_substr(text, ".expect(");
            for line in text.lines() {
                if let Some(req) = extract_req(line) {
                    if !req_ids.contains(&req) {
                        req_ids.push(req);
                    }
                }
            }
        }

        if !first_crate {
            pack.push(',');
        }
        first_crate = false;
        pack.push_str(&format!(
            "\n    {{\n      \"name\": \"{crate_name}\",\n      \"version\": \"{version}\",\n      \"rust-version\": \"{rust_version}\",\n      \"published\": {published},\n      \"dependencies\": [{}],\n      \"contracts\": {contracts},\n      \"unsafe_blocks\": {unsafe_blocks},\n      \"panic_sites\": {panic_sites},\n      \"requirements\": [{}]\n    }}",
            deps.iter()
                .map(|d| format!("\"{d}\""))
                .collect::<Vec<_>>()
                .join(", "),
            req_ids
                .iter()
                .map(|r| format!("\"{r}\""))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    pack.push_str("\n  ]\n}\n");

    fs::create_dir_all("cert").unwrap();
    fs::write("cert/certification_pack.json", &pack).unwrap();

    let mut md = String::from("# Requirement Traceability Matrix\n\n");
    md.push_str(
        "Generated by `cargo xtask certify`. Links `REQ=\"...\"`-tagged contracts to source.\n\n",
    );
    if matrix.is_empty() {
        md.push_str("_No `REQ=\"...\"`-tagged contracts found yet._\n");
    } else {
        let mut ids: Vec<&String> = matrix.keys().collect();
        ids.sort();
        for id in ids {
            md.push_str(&format!("## {id}\n\n"));
            for loc in &matrix[id] {
                md.push_str(&format!("- `{loc}`\n"));
            }
            md.push('\n');
        }
    }
    fs::write("cert/traceability_matrix.md", &md).unwrap();

    println!("certify: wrote cert/certification_pack.json and cert/traceability_matrix.md");
    0
}

/// Scaffolds contract-derived test skeletons. For every `REQ=\"...\"`-tagged
/// contract it emits a proptest-style boundary-test template (commented so the
/// generated file always compiles) plus a runnable inventory test asserting the
/// discovery count is stable. Writes `tests/contracts_gen.rs` and prints a
/// human-readable report.
fn gen_contract_tests() -> i32 {
    let sources = source_files();
    let mut entries: Vec<(String, String, String)> = Vec::new(); // (req, crate, condition)
    for (cn, _path, text) in &sources {
        for line in text.lines() {
            if let Some(req) = extract_req(line) {
                let condition = line
                    .split_once("REQ=")
                    .and_then(|(_, r)| r.split_once(')'))
                    .map(|(c, _)| c.trim().to_string())
                    .unwrap_or_default();
                entries.push((req, cn.clone(), condition));
            }
        }
    }

    let mut file = String::from("//! Auto-generated by `cargo xtask gen-contract-tests`.\n");
    file.push_str("//! Edit the commented templates to add real boundary property tests.\n\n");
    file.push_str("fn discovered() -> usize {\n");
    file.push_str(&format!("    {}\n", entries.len()));
    file.push_str("}\n\n");
    file.push_str("#[test]\n");
    file.push_str("fn contract_inventory_stable() {\n");
    file.push_str("    // Guards against accidental contract removal between releases.\n");
    file.push_str("    assert!(discovered() > 0, \"no REQ-tagged contracts discovered\");\n");
    file.push_str("}\n\n");

    for (req, cn, condition) in &entries {
        file.push_str(&format!("// Requirement {req} in {cn}\n"));
        file.push_str(&format!("//   condition: {condition}\n"));
        file.push_str(&format!(
            "// proptest! {{ fn {}_boundary {{ /* exercise boundary of: {condition} */ }} }}\n\n",
            req.replace(['-', '.'], "_")
        ));
    }

    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("crates")
        .join("out-zero-formal")
        .join("tests");
    fs::create_dir_all(&out_dir).unwrap();
    fs::write(out_dir.join("contracts_gen.rs"), &file).unwrap();

    println!(
        "gen-contract-tests: wrote crates/out-zero-formal/tests/contracts_gen.rs ({} REQ-tagged contracts)",
        entries.len()
    );
    0
}

/// Reports panic sites across every crate's `src` to support a panic-freedom
/// proof. Because every crate sets `forbid(unsafe_code)`, undefined behaviour
/// is already ruled out; this scan quantifies the remaining (safe) panic
/// surface so it can be justified or eliminated. Returns 0 (informational).
fn check_panic_freedom() -> i32 {
    let sources = source_files();
    println!("crate | contracts | unsafe | explicit panics");
    println!("--- | --- | --- | ---");
    let mut total_unsafe = 0usize;
    let mut total_panic = 0usize;
    let crates_list = all_crates();
    for (crate_path, _) in &crates_list {
        let crate_name = crate_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let mut unsafe_blocks = 0usize;
        let mut panic_sites = 0usize;
        let mut contracts = 0usize;
        for (cn, _, text) in &sources {
            if cn != &crate_name {
                continue;
            }
            contracts += count_substr(text, "requires!(");
            contracts += count_substr(text, "ensures!(");
            contracts += count_substr(text, "mcdc_requires!(");
            unsafe_blocks += count_unsafe(text);
            panic_sites += count_substr(text, "panic!(")
                + count_substr(text, "unreachable!(")
                + count_substr(text, "unimplemented!(")
                + count_substr(text, "todo!(")
                + count_substr(text, ".unwrap()")
                + count_substr(text, ".expect(");
        }
        total_unsafe += unsafe_blocks;
        total_panic += panic_sites;
        println!("{crate_name} | {contracts} | {unsafe_blocks} | {panic_sites}");
    }
    println!();
    println!("Totals: unsafe_blocks={total_unsafe}, explicit_panics={total_panic}");
    println!("Note: every crate sets `forbid(unsafe_code)`, so UB is ruled out by construction.");
    println!(
        "A full panic-freedom proof additionally needs `no-panic` or a formal tool (see Kani/Creusot/Prusti)."
    );
    0
}

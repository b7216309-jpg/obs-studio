use obs_c_oracle as _;

use std::collections::BTreeSet;
use std::fs;

/// Names of fns following a `#[unsafe(no_mangle)]` / `#[no_mangle]` attribute.
fn no_mangle_fns(src: &str) -> Vec<String> {
    let lines: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t != "#[unsafe(no_mangle)]" && t != "#[no_mangle]" {
            continue;
        }
        for next in &lines[i + 1..] {
            if let Some(pos) = next.find("fn ") {
                let name: String = next[pos + 3..]
                    .trim_start()
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                out.push(name);
                break;
            }
        }
    }
    out
}

/// Non-wildcard names in the `local:` section of `rust-exports.map`: shims
/// for C functions without `EXPORT`, which stay internal to libobs.
fn hidden_shims(map: &str) -> BTreeSet<String> {
    let local = map.split("local:").nth(1).expect("local: section");
    local
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.contains('*') && !s.contains('}'))
        .filter(|s| s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
        .filter(|s| !s.starts_with("_R") && *s != "rust_eh_personality")
        .map(String::from)
        .collect()
}

fn read(rel: &str) -> String {
    let path = format!("{}/../../{rel}", env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

/// Every `#[no_mangle]` shim is either exported (`rust-exports.txt`) or
/// explicitly hidden (`rust-exports.map`, and `rust-unexports-macos.txt` with
/// Mach-O's leading underscore), and never both.
#[test]
fn rust_exports_list_matches_no_mangle_shims() {
    let listed: BTreeSet<String> = read("libobs/cmake/rust-exports.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect();
    let hidden = hidden_shims(&read("libobs/cmake/rust-exports.map"));
    let macos: BTreeSet<String> = read("libobs/cmake/rust-unexports-macos.txt")
        .lines()
        .map(str::trim)
        .filter_map(|l| l.strip_prefix('_'))
        .map(String::from)
        .collect();

    let mut found = BTreeSet::new();
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/ffi");
    for entry in fs::read_dir(dir).expect("read src/ffi") {
        let path = entry.expect("dir entry").path();
        if path.extension().is_some_and(|e| e == "rs") {
            let src = fs::read_to_string(&path).expect("read ffi source");
            found.extend(no_mangle_fns(&src));
        }
    }

    let accounted: BTreeSet<String> = listed.union(&hidden).cloned().collect();
    let missing_from_lists: Vec<_> = found.difference(&accounted).collect();
    let missing_from_src: Vec<_> = accounted.difference(&found).collect();
    let both: Vec<_> = listed.intersection(&hidden).collect();
    let missing_on_macos: Vec<_> = hidden.difference(&macos).collect();
    assert!(
        missing_from_lists.is_empty()
            && missing_from_src.is_empty()
            && both.is_empty()
            && missing_on_macos.is_empty(),
        "shims in neither rust-exports.txt nor rust-exports.map: {missing_from_lists:?};          listed but no #[no_mangle] shim in src/ffi: {missing_from_src:?};          both exported and hidden: {both:?};          hidden in rust-exports.map but not rust-unexports-macos.txt: {missing_on_macos:?}"
    );
}

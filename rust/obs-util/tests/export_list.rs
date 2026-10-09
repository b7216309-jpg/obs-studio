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

#[test]
fn rust_exports_list_matches_no_mangle_shims() {
    let list = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../libobs/cmake/rust-exports.txt"
    ))
    .expect("read rust-exports.txt");
    let listed: BTreeSet<String> = list
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect();

    let mut found = BTreeSet::new();
    let mut visited = Vec::new();
    for krate in fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/..")).expect("read rust/") {
        let ffi_dir = krate.expect("dir entry").path().join("src/ffi");
        if !ffi_dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&ffi_dir).expect("read src/ffi") {
            let path = entry.expect("dir entry").path();
            if path.extension().is_some_and(|e| e == "rs") {
                let src = fs::read_to_string(&path).expect("read ffi source");
                found.extend(no_mangle_fns(&src));
            }
        }
        visited.push(ffi_dir);
    }
    for krate in ["obs-util", "obs-graphics"] {
        assert!(
            visited.iter().any(|d| d.ends_with(format!("{krate}/src/ffi"))),
            "{krate}/src/ffi was not scanned: {visited:?}"
        );
    }

    let missing_from_list: Vec<_> = found.difference(&listed).collect();
    let missing_from_src: Vec<_> = listed.difference(&found).collect();
    assert!(
        missing_from_list.is_empty() && missing_from_src.is_empty(),
        "shims missing from rust-exports.txt: {missing_from_list:?}; \
         listed but no #[no_mangle] shim in rust/*/src/ffi:{missing_from_src:?}"
    );
}

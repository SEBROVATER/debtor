use std::fs;
use std::path::{Path, PathBuf};

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).expect("read_dir");
    for entry in entries {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

#[test]
fn templates_use_only_htmx_script() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let templates_dir = root.join("src").join("web").join("templates");
    let mut files = Vec::new();
    collect_files(&templates_dir, &mut files);

    let mut layout_scripts = 0usize;

    for path in files {
        let content = fs::read_to_string(&path).expect("read template");
        let lower = content.to_lowercase();
        if lower.contains("<script") {
            if path.ends_with("layout.html") {
                layout_scripts += lower.matches("<script").count();
                assert!(
                    lower.contains("htmx.org"),
                    "layout should only include htmx script"
                );
            } else {
                panic!("unexpected script tag in template: {}", path.display());
            }
        }
    }

    assert_eq!(
        layout_scripts, 1,
        "layout should include a single script tag"
    );
}

#[test]
fn static_assets_do_not_include_custom_js() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let static_dir = root.join("static");
    if !static_dir.exists() {
        return;
    }

    let mut files = Vec::new();
    collect_files(&static_dir, &mut files);

    let js_files: Vec<_> = files
        .into_iter()
        .filter(|path| path.extension().map(|ext| ext == "js").unwrap_or(false))
        .collect();

    assert!(js_files.is_empty(), "unexpected js assets: {js_files:?}");
}

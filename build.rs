use std::path::Path;
use cc::Build;

fn main() {
    let src_dir = Path::new("external/tree-sitter-fortran/src");
    let files = ["parser.c", "scanner.c"].map(|x| src_dir.join(x));
    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display())
    }
    Build::new()
        .include(src_dir)
        .files(files)
        .compile("tree-sitter-fortran");
}

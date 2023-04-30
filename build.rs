// build.rs

use std::env;
use std::path::Path;
use built;

fn main() {
        let mut opts = built::Options::default();
        opts.set_dependencies(true);
        opts.set_git(true);
        let src = env::var("CARGO_MANIFEST_DIR").unwrap();
        let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
        println!("Path: {:?}", dst);
        built::write_built_file_with_opts(&opts, src.as_ref(), &dst)
            .expect("Failed to acquire build-time information");
}
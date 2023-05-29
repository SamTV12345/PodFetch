// build.rs

use std::env;
use std::path::Path;
use std::process::Command;
use built;
use built::Options;

fn main() {
        #[cfg(feature = "sqlite")]
        println!("cargo:rustc-cfg=sqlite");
        #[cfg(feature = "mysql")]
        println!("cargo:rustc-cfg=mysql");
        #[cfg(feature = "postgresql")]
        println!("cargo:rustc-cfg=postgresql");
        let mut opts = Options::default();
        opts.set_dependencies(true);
        let maybe_vaultwarden_version =
            env::var("VW_VERSION").or_else(|_| env::var("BWRS_VERSION")).or_else(|_| version_from_git_info());

        #[cfg(feature = "postgresql")]
        version_from_git_info().expect("Error retrieving git information");

        create_git_sqlite(opts);
        if let Ok(version) = maybe_vaultwarden_version {
                println!("cargo:rustc-env=VW_VERSION={version}");
                println!("cargo:rustc-env=CARGO_PKG_VERSION={version}");
                println!("cargo:rustc-env=GIT_EXACT_TAG={version}");
        }
        else{
                println!("cargo:rustc-env=VW_VERSION=unknown");
                println!("cargo:rustc-env=CARGO_PKG_VERSION=unknown");
                println!("cargo:rustc-env=GIT_EXACT_TAG=unknown");
                println!("cargo:rustc-env=GIT_LAST_TAG=unknown");
                println!("cargo:rustc-env=GIT_BRANCH=unknown");
                println!("cargo:rustc-env=GIT_REV=unknown");
        }
}


fn create_git_sqlite(mut opts: Options) {
        let src = env::var("CARGO_MANIFEST_DIR").unwrap();
        let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
        println!("Path: {:?}", dst);
        built::write_built_file_with_opts(&opts, src.as_ref(), &dst)
            .expect("Failed to acquire build-time information");
}

fn version_from_git_info() -> Result<String, std::io::Error> {
        // The exact tag for the current commit, can be empty when
        // the current commit doesn't have an associated tag
        let exact_tag = run(&["git", "describe", "--abbrev=0", "--tags", "--exact-match"]).ok();
        if let Some(ref exact) = exact_tag {
                println!("cargo:rustc-env=GIT_EXACT_TAG={exact}");
        }

        // The last available tag, equal to exact_tag when
        // the current commit is tagged
        let last_tag = run(&["git", "describe", "--abbrev=0", "--tags"])?;
        println!("cargo:rustc-env=GIT_LAST_TAG={last_tag}");

        // The current branch name
        let branch = run(&["git", "rev-parse", "--abbrev-ref", "HEAD"])?;
        println!("cargo:rustc-env=GIT_BRANCH={branch}");

        // The current git commit hash
        let rev = run(&["git", "rev-parse", "HEAD"])?;
        let rev_short = rev.get(..8).unwrap_or_default();
        println!("cargo:rustc-env=GIT_REV={rev_short}");

        // Combined version
        if let Some(exact) = exact_tag {
                Ok(exact)
        } else if &branch != "main" && &branch != "master" {
                Ok(format!("{last_tag}-{rev_short} ({branch})"))
        } else {
                Ok(format!("{last_tag}-{rev_short}"))
        }
}

fn run(args: &[&str]) -> Result<String, std::io::Error> {
        let out = Command::new(args[0]).args(&args[1..]).output()?;
        if !out.status.success() {
                use std::io::{Error, ErrorKind};
                return Err(Error::new(ErrorKind::Other, "Command not successful"));
        }
        Ok(String::from_utf8(out.stdout).unwrap().trim().to_string())
}
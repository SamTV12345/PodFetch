// build.rs

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let result_of_git = version_from_git_info();
    match result_of_git {
        Ok(version) => {
            println!("cargo:rustc-env=VW_VERSION={version}");
        }
        Err(_) => {
            // Git info not available — VW_VERSION falls back to the Cargo.toml version.
            // Individual GIT_* env vars are already set to "unknown" by version_from_git_info.
            // CARGO_PKG_VERSION is always set by cargo itself; never overwrite it.
            println!("cargo:rustc-env=VW_VERSION=unknown");
        }
    }

    create_git_sqlite();
}

fn create_git_sqlite() {
    let src = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
    println!("Path: {dst:?}");
    let path = Path::new(&src);
    built::write_built_file_with_opts(Option::from(path), &dst)
        .expect("Failed to acquire build-time information");
}

fn version_from_git_info() -> Result<String, std::io::Error> {
    // The exact tag for the current commit, can be empty when
    // the current commit doesn't have an associated tag
    let exact_tag = run(&["git", "describe", "--abbrev=0", "--tags", "--exact-match"]).ok();
    if let Some(ref exact) = exact_tag {
        println!("cargo:rustc-env=GIT_EXACT_TAG={exact}");
    } else {
        println!("cargo:rustc-env=GIT_EXACT_TAG=unknown");
    }

    // The last available tag, equal to exact_tag when
    // the current commit is tagged
    let last_tag = run(&["git", "describe", "--abbrev=0", "--tags"])
        .map(|t| {
            println!("cargo:rustc-env=GIT_LAST_TAG={t}");
            t
        })
        .unwrap_or_else(|_| {
            println!("cargo:rustc-env=GIT_LAST_TAG=unknown");
            "unknown".to_string()
        });

    // The current branch name
    let branch = run(&["git", "rev-parse", "--abbrev-ref", "HEAD"])
        .map(|b| {
            println!("cargo:rustc-env=GIT_BRANCH={b}");
            b
        })
        .unwrap_or_else(|_| {
            println!("cargo:rustc-env=GIT_BRANCH=unknown");
            "unknown".to_string()
        });

    // The current git commit hash
    let rev = run(&["git", "rev-parse", "HEAD"])
        .inspect(|r| {
            let short = r.get(..8).unwrap_or_default();
            println!("cargo:rustc-env=GIT_REV={short}");
        })
        .unwrap_or_else(|_| {
            println!("cargo:rustc-env=GIT_REV=unknown");
            "unknown".to_string()
        });
    let rev_short = rev.get(..8).unwrap_or("unknown");

    // Combined version — always <tag>-<hash> format, matching Jenkins pipeline
    if last_tag == "unknown" && rev_short == "unknown" {
        // No git info at all — signal the caller to use the fallback
        Err(std::io::Error::other("No git information available"))
    } else if &branch != "main" && &branch != "master" && &branch != "unknown" {
        Ok(format!("{last_tag}-{rev_short} ({branch})"))
    } else {
        Ok(format!("{last_tag}-{rev_short}"))
    }
}

fn run(args: &[&str]) -> Result<String, std::io::Error> {
    let out = Command::new(args[0]).args(&args[1..]).output()?;
    if !out.status.success() {
        use std::io::Error;
        return Err(Error::other("Command not successful"));
    }
    Ok(String::from_utf8(out.stdout).unwrap().trim().to_string())
}

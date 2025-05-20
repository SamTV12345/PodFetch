// build.rs

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let result_of_git = version_from_git_info();
    match result_of_git {
        Ok(version) => {
            println!("cargo:rustc-env=VW_VERSION={version}");
            println!("cargo:rustc-env=CARGO_PKG_VERSION={version}");
            println!("cargo:rustc-env=GIT_EXACT_TAG={version}");
        }
        Err(_) => {
            println!("cargo:rustc-env=VW_VERSION=unknown");
            println!("cargo:rustc-env=CARGO_PKG_VERSION=unknown");
            println!("cargo:rustc-env=GIT_EXACT_TAG=unknown");
            println!("cargo:rustc-env=GIT_LAST_TAG=unknown");
            println!("cargo:rustc-env=GIT_BRANCH=unknown");
            println!("cargo:rustc-env=GIT_REV=unknown");
        }
    }

    create_git_sqlite();
}

fn create_git_sqlite() {
    let src = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
    println!("Path: {:?}", dst);
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
        use std::io::Error;
        return Err(Error::other("Command not successful"));
    }
    Ok(String::from_utf8(out.stdout).unwrap().trim().to_string())
}

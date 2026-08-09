use std::process::Command;

fn main() {
    // Exact tag for the current commit (empty if not on a tag)
    if let Some(exact) = run(&["git", "describe", "--abbrev=0", "--tags", "--exact-match"]).ok() {
        println!("cargo:rustc-env=GIT_EXACT_TAG={exact}");
    } else {
        println!("cargo:rustc-env=GIT_EXACT_TAG=unknown");
    }

    // Most recent tag
    let last_tag = run(&["git", "describe", "--abbrev=0", "--tags"])
        .map(|t| {
            println!("cargo:rustc-env=GIT_LAST_TAG={t}");
            t
        })
        .unwrap_or_else(|_| {
            println!("cargo:rustc-env=GIT_LAST_TAG=unknown");
            "unknown".to_string()
        });

    // Branch name
    let branch = run(&["git", "rev-parse", "--abbrev-ref", "HEAD"])
        .map(|b| {
            println!("cargo:rustc-env=GIT_BRANCH={b}");
            b
        })
        .unwrap_or_else(|_| {
            println!("cargo:rustc-env=GIT_BRANCH=unknown");
            "unknown".to_string()
        });

    // Commit hash
    let rev = run(&["git", "rev-parse", "HEAD"])
        .map(|r| {
            let short = r.get(..8).unwrap_or_default();
            println!("cargo:rustc-env=GIT_REV={short}");
            r
        })
        .unwrap_or_else(|_| {
            println!("cargo:rustc-env=GIT_REV=unknown");
            "unknown".to_string()
        });
    let rev_short = rev.get(..8).unwrap_or("unknown");

    // Combined version — always <tag>-<hash> format
    if last_tag != "unknown" || rev_short != "unknown" {
        let version = if &branch != "main" && &branch != "master" && &branch != "unknown" {
            format!("{last_tag}-{rev_short} ({branch})")
        } else {
            format!("{last_tag}-{rev_short}")
        };
        println!("cargo:rustc-env=VW_VERSION={version}");
    } else {
        println!("cargo:rustc-env=VW_VERSION=unknown");
    }
}

fn run(args: &[&str]) -> Result<String, std::io::Error> {
    let out = Command::new(args[0]).args(&args[1..]).output()?;
    if !out.status.success() {
        return Err(std::io::Error::other("Command not successful"));
    }
    Ok(String::from_utf8(out.stdout).unwrap().trim().to_string())
}

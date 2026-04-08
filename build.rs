use std::fs::File;
use std::process::Command;
use tar;

fn main() {
    let src = "neon/pg_install";
    let out = "neon/target/pg_install.tar";

    let file = File::create(out).unwrap();
    let mut builder = tar::Builder::new(file);
    builder.append_dir_all(".", src).unwrap();
    builder.finish().unwrap();

    println!("cargo:rerun-if-changed={}", src);

    let git_commit_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|hash| hash.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_commit_hash);
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
}
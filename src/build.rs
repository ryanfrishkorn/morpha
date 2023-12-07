// Obtain current git hash and pass it to the compiler as an environment variable
use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    // let git_hash = git_hash.get(0..7).unwrap();
    print!("cargo:rustc-env=GIT_HASH={}", git_hash);
}

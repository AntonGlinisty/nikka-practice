use std::{
    env,
    error::Error,
    process::Command,
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=../Cargo.toml");
    println!("cargo:rerun-if-changed=../Cargo.lock");
    println!("cargo:rerun-if-changed=../ku/src");
    println!("cargo:rerun-if-changed=../ku/Cargo.toml");

    let binaries = vec![
        "cow_fork",
        "eager_fork",
        "exit",
        "loop",
        "page_fault",
        "sched_yield",
    ];

    for bin in binaries {
        println!("cargo:rerun-if-changed=../user/{bin}/Cargo.toml");
        println!("cargo:rerun-if-changed=../user/{bin}/src");

        Command::new("cargo")
            .args(["build", "--profile", "user"])
            .current_dir(format!("../user/{bin}"))
            .env_clear()
            .env("PATH", env::var("PATH")?)
            .output()
            .unwrap_or_else(|_| panic!("failed to build user binary {bin}"));
    }

    Ok(())
}

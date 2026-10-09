#![feature(exit_status_error)]

use std::{
    env,
    fs::{
        self,
        File,
    },
    io::{
        BufWriter,
        Write as _,
    },
    path::PathBuf,
    process::Command,
};

use anyhow::{
    Context,
    Result,
    anyhow,
    bail,
};
use bootloader::UefiBoot;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=fs.img");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=ku/Cargo.toml");
    println!("cargo:rerun-if-changed=ku/src");

    let binaries = vec![
        "cow_fork",
        "eager_fork",
        "exit",
        "log_value",
        "loop",
        "check_context",
        "memory_syscalls",
        "page_fault",
        "sched_yield",
        "trap_handler",
    ];

    for bin in binaries {
        build(&format!("user/{bin}"))?;
    }

    let kernel = &build("kernel")?;
    let stripped_kernel = format!("{kernel}.stripped");

    let out_dir =
        PathBuf::from(env::var_os("OUT_DIR").ok_or(anyhow!("no OUT_DIR environment variable"))?);

    let dest_path = out_dir.join("kernel.path");
    let mut f = BufWriter::new(File::create(&dest_path)?);
    write!(f, "{kernel}")?;

    let uefi_path = out_dir.join("uefi.img");
    UefiBoot::new(&PathBuf::from(stripped_kernel)).create_disk_image(&uefi_path)?;

    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());

    make_fs_image("fs.img", 32 << 20)
        .with_context(|| "failed to create a disk image for the file system")
}

fn build(path: &str) -> Result<String> {
    println!("cargo:rerun-if-changed={path}/Cargo.toml");
    println!("cargo:rerun-if-changed={path}/src");

    let message = format!("failed to build {path}");

    let mut command = Command::new("cargo");
    command.current_dir(path).env_clear().env("PATH", env::var("PATH")?);

    let user = path.starts_with("user/");
    let mut profile = "--profile=ci";
    if user {
        command.arg("-Zjson-target-spec");
        profile = "--profile=user";
    }
    command.arg("build").arg("--color=always").arg(profile);
    if path == "kernel" &&
        let Ok(features) = env::var("CARGO_CFG_FEATURE")
    {
        command.arg(format!("--features={features}"));
    }

    let output = command.output().with_context(|| message.clone())?;

    if !output.status.success() {
        println!(
            "cargo:warning=\n{}\n{}",
            str::from_utf8(output.stdout.as_slice()).with_context(|| "<some invalid UTF-8>")?,
            str::from_utf8(output.stderr.as_slice()).with_context(|| "<some invalid UTF-8>")?,
        );
        bail!(message);
    }

    let bin = executable(path);
    let stripped_bin = format!("{bin}.stripped");
    Command::new("strip").args(["-o", &stripped_bin, &bin]).output()?.exit_ok()?;

    Ok(bin)
}

fn executable(path: &str) -> String {
    let path = path.replace('_', "-");
    let user = path.starts_with("user/");
    if user {
        format!("target/kernel/{path}")
    } else {
        format!("target/x86_64-unknown-none/ci/{path}")
    }
}

fn make_fs_image(
    path: &str,
    size: usize,
) -> Result<(), std::io::Error> {
    if fs::exists(path).unwrap_or(false) {
        Ok(())
    } else {
        fs::write(path, vec![INVALID_DISK_PATTERN; size])
    }
}

const INVALID_DISK_PATTERN: u8 = 0xAA;

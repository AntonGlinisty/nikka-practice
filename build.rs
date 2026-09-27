use std::{
    env,
    // error::Error,
    fs::File,
    io::{
        BufWriter,
        Write as _,
    },
    path::PathBuf,
    // process::Command,
};

use bootloader::UefiBoot;

// fn main() -> Result<(), Box<dyn Error>> {
fn main() {
    //    println!("cargo:rerun-if-changed=Cargo.lock");
    //    println!("cargo:rerun-if-changed=fs.img");
    //    println!("cargo:rerun-if-changed=ku/Cargo.toml");
    //    println!("cargo:rerun-if-changed=ku/src");
    //
    //    let binaries = vec![
    //        "lib",
    //        "cow_fork",
    //        "eager_fork",
    //        "exit",
    //        "log_value",
    //        "loop",
    //        "check_context",
    //        "memory_syscalls",
    //        "page_fault",
    //        "sched_yield",
    //        "trap_handler",
    //    ];
    //
    //    for bin in binaries {
    //        println!("cargo:rerun-if-changed=user/{bin}/Cargo.toml");
    //        println!("cargo:rerun-if-changed=user/{bin}/src");
    //
    //        let terminate = || panic!("failed to build user binary {bin}");
    //
    //        let output = Command::new("cargo")
    //            .args(["build", "--profile", "user"])
    //            .current_dir(format!("user/{bin}"))
    //            .env_clear()
    //            .env("PATH", env::var("PATH")?)
    //            .output()
    //            .unwrap_or_else(|_| terminate());
    //
    //        if !output.status.success() {
    //            println!(
    //                "cargo:warning=\n{}",
    //                str::from_utf8(output.stderr.as_slice()).unwrap_or("<some invalid UTF-8>"),
    //            );
    //            terminate();
    //        }
    //    }

    //    for (key, value) in std::env::vars() { println!("ENV {}={}", key, value); }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let kernel = PathBuf::from(env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());

    let dest_path = out_dir.join("kernel.path");
    let mut f = BufWriter::new(File::create(&dest_path).unwrap());

    write!(f, "{}", kernel.display()).unwrap();

    let uefi_path = out_dir.join("uefi.img");
    UefiBoot::new(&kernel).create_disk_image(&uefi_path).unwrap();

    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());

    // Ok(())
}

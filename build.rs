use std::{
    env,
    fs::File,
    io::{
        BufWriter,
        Write as _,
    },
    path::PathBuf,
};

use bootloader::UefiBoot;

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let kernel = PathBuf::from(env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());

    let dest_path = out_dir.join("kernel.path");
    let mut f = BufWriter::new(File::create(&dest_path).unwrap());

    write!(f, "{}", kernel.display()).unwrap();

    let uefi_path = out_dir.join("uefi.img");
    UefiBoot::new(&kernel).create_disk_image(&uefi_path).unwrap();

    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
}

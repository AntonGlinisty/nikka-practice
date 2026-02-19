#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::process::Command;

fn main() {
    let uefi_path = env!("UEFI_PATH");

    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.arg("-cpu").arg("max");
    cmd.arg("-m").arg("size=128M");
    cmd.arg("-smp").arg("cpus=4");
    cmd.arg("-device").arg("isa-debug-exit,iobase=0xF4,iosize=0x04");
    cmd.arg("-serial").arg("stdio");
    cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
    cmd.arg("-drive").arg(format!("format=raw,file={uefi_path}"));
    cmd.arg("-drive").arg("file=fs.img,index=2,media=disk,driver=raw");

    println!("Running: {:?}", cmd);

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}

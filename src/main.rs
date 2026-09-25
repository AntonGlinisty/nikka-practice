#![allow(dead_code)]
#![allow(unused_features)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![deny(warnings)]

use std::{
    fs,
    process::Command,
};

use anyhow::Result;
use ovmf_prebuilt::{
    Arch,
    FileType,
    Prebuilt,
    Source,
};
use structopt::StructOpt;

use anyhow::bail;

#[derive(StructOpt)]
struct Args {
    /// Start RTC at specific time
    #[structopt(long = "rtc-base")]
    rtc_base: Option<String>,

    #[cfg_attr(
        feature = "tests",
        doc = "Substring selecting tests to run, eg. \"1-hw-1-rtc\", \"rtc\", \"1-hw-\""
    )]
    #[cfg_attr(
        not(feature = "tests"),
        doc = "Substring selecting tests to run, requires feature \"tests\""
    )]
    #[structopt(long, short)]
    test: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::from_args();

    let test_config_image = "test-config.img";
    let uefi_path = env!("UEFI_PATH");

    #[cfg(not(feature = "tests"))]
    if !args.test.is_empty() {
        bail!(
            "run with \"tests\" feature: 'cargo t --test ...' or 'cargo run --features tests -- \
             --test ...",
        );
    }

    let ovmf = Prebuilt::fetch(Source::LATEST, "target/ovmf").expect("failed to update prebuilt");

    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.arg("-cpu").arg("max,+x2apic");
    cmd.arg("-m").arg("size=256M");
    cmd.arg("-smp").arg("cpus=4");
    cmd.arg("-device").arg("isa-debug-exit,iobase=0xF4,iosize=0x04");
    cmd.arg("-drive").arg(format!(
        "if=pflash,format=raw,readonly=on,file={}",
        ovmf.get_file(Arch::X64, FileType::Code).display(),
    ));
    cmd.arg("-drive").arg(format!(
        "if=pflash,format=raw,file={}",
        ovmf.get_file(Arch::X64, FileType::Vars).display(),
    ));
    cmd.arg("-drive").arg(format!("file={uefi_path},index=0,media=disk,format=raw"));
    cmd.arg("-drive").arg("file=fs.img,index=1,media=disk,format=raw");

    if args.test.is_empty() {
        cmd.arg("-serial").arg("stdio");
    } else {
        cmd.arg("-drive").arg(format!(
            "file={test_config_image},index=2,media=disk,format=raw",
        ));
        cmd.arg("-nographic");

        make_test_config_image(test_config_image, &args.test).expect("failed to make tests config");
    }

    if let Some(rtc_base) = args.rtc_base {
        cmd.arg("-rtc").arg(format!("base={rtc_base}"));
    }

    println!("Running: {cmd:?}");

    let mut child = cmd.spawn()?;
    let child_exit_status = child.wait()?;

    if !args.test.is_empty() {
        fs::remove_file(test_config_image).unwrap_or(());
    }

    if child_exit_status.code() != Some(TEST_SUCCESS_EXIT_CODE) {
        bail!("Test failed");
    }

    Ok(())
}

fn make_test_config_image(
    path: &str,
    config: &[String],
) -> Result<(), std::io::Error> {
    fs::remove_file(path).unwrap_or(());

    let mut config_sector = config.join("\n").replace('-', "_");
    let sector_len = 512;
    while config_sector.len() < sector_len {
        config_sector += "\n";
    }

    fs::write(path, config_sector.as_bytes())
}

const TEST_SUCCESS_EXIT_CODE: i32 = 3;

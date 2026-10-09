#![deny(warnings)]

use std::{
    fs,
    process::Stdio,
    sync::atomic::{
        AtomicBool,
        Ordering,
    },
};

use anyhow::Result;
use ovmf_prebuilt::{
    Arch,
    FileType,
    Prebuilt,
    Source,
};
use structopt::StructOpt;
use tokio::{
    io::{
        AsyncBufReadExt as _,
        BufReader,
    },
    process::Command,
};

use anyhow::bail;

#[derive(StructOpt)]
struct Args {
    /// Binary with debugging information for backtrace symbolization
    /// (default to the kernel binary)
    #[structopt(long)]
    exe: Option<String>,

    /// Attach gdb to qemu
    #[structopt(long)]
    gdb: bool,

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

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum State {
    Backtrace,

    #[default]
    Normal,
}

impl State {
    fn step(
        &mut self,
        line: &str,
    ) {
        match self {
            State::Backtrace =>
                if !line.starts_with("  0") {
                    *self = State::Normal;
                },
            State::Normal =>
                if line == "Backtrace:" {
                    *self = State::Backtrace;
                },
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
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
    cmd.arg("-m").arg("size=128M");
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

    if args.gdb {
        cmd.stdin(Stdio::null());
        cmd.arg("-gdb").arg("tcp::1234").arg("-S");
    }

    println!("Nikka kernel binary: {KERNEL}");
    println!("Running: {cmd:?}");

    let _gdb = if args.gdb {
        let mut gdb_command = Command::new("gdb");
        gdb_command.arg("--command=gdbinit").arg(format!("--symbols={KERNEL}"));
        if let Some(exe) = &args.exe {
            gdb_command.arg(format!("--symbols={exe}"));
        }

        Some(gdb_command.kill_on_drop(true).spawn())
    } else {
        None
    };

    let exe = args.exe.unwrap_or(KERNEL.into());

    let mut qemu = cmd.stdout(Stdio::piped()).kill_on_drop(true).spawn()?;

    let stdout = qemu.stdout.take().expect("stdout should be captured");

    let mut lines = BufReader::new(stdout).lines();
    let mut state = State::default();

    while let Some(line) = lines.next_line().await? {
        print!("{}", str::from_utf8(&strip_ansi_escapes::strip(&line))?);
        if state == State::Backtrace && line.starts_with("  0") {
            println!(
                " {}",
                symbolize_backtrace(line.trim(), &exe, false).await?.replace('\n', " "),
            );
        } else {
            println!();
        };
        if state == State::Normal &&
            let Some(backtrace) = substring(&line, "backtrace: [", "]")
        {
            println!("{}", symbolize_backtrace(backtrace, &exe, true).await?);
        }
        state.step(&line);
    }

    let qemu_exit_status = qemu.wait().await?;

    if !args.test.is_empty() {
        fs::remove_file(test_config_image).unwrap_or(());
    }

    if qemu_exit_status.code() != Some(TEST_SUCCESS_EXIT_CODE) {
        if args.test.is_empty() {
            bail!("Emulation failed");
        } else {
            bail!("Test failed");
        }
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

fn substring<'a>(
    value: &'a str,
    start: &str,
    end: &str,
) -> Option<&'a str> {
    let start = start.len() + value.find(start)?;

    let end = if end.is_empty() {
        value.len()
    } else {
        start + value[start ..].find(end)?
    };

    Some(&value[start .. end])
}

async fn symbolize_backtrace(
    backtrace: &str,
    exe: &str,
    print_address: bool,
) -> Result<String> {
    static SYMBOLIZER_NOT_INSTALLED: AtomicBool = AtomicBool::new(false);

    let mut cmd = Command::new(SYMBOLIZER);
    cmd.arg(format!("--exe={exe}"));
    cmd.arg("--demangle");
    cmd.arg("--output-style").arg("GNU");
    cmd.arg("--pretty-print");
    cmd.arg("--relativenames");
    if print_address {
        cmd.arg("--print-address");
    }

    let backtrace = backtrace.replace('v', "x").replace(['_', ','], "");
    cmd.args(backtrace.split(' ').collect::<Vec<_>>());

    let symbolized_backtrace = match cmd.stdout(Stdio::piped()).spawn() {
        Ok(symbolizer) => {
            let output = symbolizer.wait_with_output().await?;
            str::from_utf8(&output.stdout)?.trim().into()
        },
        Err(error) =>
            if !SYMBOLIZER_NOT_INSTALLED.load(Ordering::Relaxed) {
                SYMBOLIZER_NOT_INSTALLED.store(true, Ordering::Relaxed);
                format!("failed to run {SYMBOLIZER}, maybe it is not installed: {error}")
            } else {
                String::new()
            },
    };

    Ok(symbolized_backtrace)
}

static KERNEL: &str = include_str!(concat!(env!("OUT_DIR"), "/kernel.path"));

const SYMBOLIZER: &str = "llvm-symbolizer";
const TEST_SUCCESS_EXIT_CODE: i32 = 3;

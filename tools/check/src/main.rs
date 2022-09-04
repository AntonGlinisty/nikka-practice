#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::{
    env,
    ffi::OsString,
    fs,
    io::{
        self,
        BufRead,
    },
    path::{
        Path,
        PathBuf,
    },
    process::Command,
};

use anyhow::{
    Context,
    Result,
    bail,
};
use structopt::StructOpt;

////////////////////////////////////////////////////////////////////////////////

const ALLOW_LIST_FILE: &str = ".allowlist";
const REPORT_URL: &str = "https://os.manytask.org/api/report";
const SUDO_PRESERVE_ENVS: &[&str] = &["PATH", "HOME", "CARGO_HOME", "RUSTUP_HOME"];
const TESTER_TOKEN_ENV: &str = "TESTER_TOKEN";

////////////////////////////////////////////////////////////////////////////////

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opts {
    /// Path to the student repository.
    #[structopt(short = "s", long = "student-repo")]
    student_repo: PathBuf,
    /// Path to the original repo.
    #[structopt(short = "o", long = "original-repo")]
    original_repo: PathBuf,
    /// CI branch name (e.g. "submit/add")
    #[structopt(short = "b", long = "ci-branch-name")]
    ci_branch_name: String,
    /// Disable file processing.
    #[structopt(short = "u", long = "user-id")]
    user_id: String,
}

////////////////////////////////////////////////////////////////////////////////

fn parse_task_name(branch: &str) -> Result<&str> {
    const PREFIX: &str = "submit/";
    if !branch.starts_with(PREFIX) {
        bail!("branch '{}' does not start with '{}'", branch, PREFIX);
    }

    let task = &branch[PREFIX.len() ..];
    if task.contains('/') {
        bail!("task '{}' contains '/'", task);
    }

    Ok(task)
}

fn read_allow_list(path: &Path) -> Result<Vec<OsString>> {
    let file = fs::File::open(path)?;

    let mut list = vec![];
    for mb_line in io::BufReader::new(file).lines() {
        list.push(mb_line.context("failed to read line")?.into())
    }

    Ok(list)
}

fn check_task(path: &Path) -> Result<()> {
    let status = Command::new("sudo")
        .args(SUDO_PRESERVE_ENVS.iter().map(|e| format!("--preserve-env={}", e)))
        .args(["-u", "nobody", "make", "-C"])
        .arg(path)
        .spawn()?
        .wait()?;
    if !status.success() {
        bail!("'make' command terminated with error");
    }
    Ok(())
}

fn submit_report(
    user_id: &str,
    task_name: &str,
    tester_token: &str,
) -> Result<()> {
    let form = reqwest::blocking::multipart::Form::new()
        .text("user_id", user_id.to_string())
        .text("task", task_name.to_string())
        .text("token", tester_token.to_string());
    reqwest::blocking::Client::new()
        .post(REPORT_URL)
        .multipart(form)
        .send()?
        .error_for_status()?;
    Ok(())
}

fn do_main(opts: Opts) -> Result<()> {
    let task_name = parse_task_name(&opts.ci_branch_name).context("failed to parse task name")?;

    let orig_path = opts.original_repo.join(&task_name);
    let stud_path = opts.student_repo.join(&task_name);

    let allow_list_path = orig_path.join(ALLOW_LIST_FILE);
    let allow_list = read_allow_list(&allow_list_path).with_context(|| {
        format!(
            "failed to read allow list from {}",
            allow_list_path.display()
        )
    })?;

    for file in allow_list {
        let src = stud_path.join(&file);
        let dst = orig_path.join(&file);
        fs::copy(&src, &dst)
            .with_context(|| format!("failed to copy {} to {}", src.display(), dst.display()))?;
    }

    check_task(&orig_path).context("task check failed")?;

    let tester_token = env::var(TESTER_TOKEN_ENV)
        .with_context(|| format!("envirnnment variable '{}' is not set", TESTER_TOKEN_ENV))?;
    submit_report(&opts.user_id, task_name, &tester_token).context("failed to submit report")
}

fn main() {
    let args = Opts::from_args();

    if let Err(err) = do_main(args) {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}

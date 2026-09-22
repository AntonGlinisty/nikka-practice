#![allow(dead_code)]
#![allow(unused_features)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::{
    collections::{
        HashMap,
        HashSet,
    },
    env,
    fs,
    path::PathBuf,
    process::Command,
    vec::Vec,
};

use anyhow::{
    Context,
    Result,
    bail,
};
use glob::glob;
use regex::Regex;
use serde::Deserialize;
use structopt::StructOpt;

////////////////////////////////////////////////////////////////////////////////

const CONFIG_NAME: &str = ".manytask.yml";
const SUDO_PRESERVE_ENVS: &[&str] = &["PATH", "HOME", "CARGO_HOME", "RUSTUP_HOME"];
const TESTER_TOKEN_ENV: &str = "TESTER_TOKEN";

#[derive(Deserialize, Clone)]
struct Task {
    task: String,
    score: u32,
    is_bonus: Option<bool>,
    allow_list: Option<Vec<String>>,
    dependencies: Option<Vec<String>>,
    prerequisites: Option<Vec<String>>,
    check: Vec<String>,
}

#[derive(Deserialize, Clone)]
struct Group {
    group: String,
    tasks: Vec<Task>,
}
#[derive(Deserialize, Clone)]
struct Deadlines {
    schedule: Vec<Group>,
}
#[derive(Deserialize, Clone)]
struct Config {
    allow_list: Option<Vec<String>>,
    deadlines: Deadlines,
    report_url: String,
}

impl Config {
    fn find_task(
        &self,
        task_name: &str,
    ) -> Option<&Task> {
        for group in &self.deadlines.schedule {
            for task in &group.tasks {
                if task.task == task_name {
                    return Some(task);
                }
            }
        }

        None
    }

    fn task_checks(
        &self,
        task_name: &str,
        prerequisites_check: bool,
    ) -> Vec<&str> {
        let mut dependencies = HashMap::<&str, Vec<&str>>::new();
        let mut checks = HashMap::<&str, &Vec<String>>::new();

        for group in &self.deadlines.schedule {
            for task in &group.tasks {
                checks.insert(&task.task, &task.check);
                if prerequisites_check && let Some(task_dependencies) = &task.dependencies {
                    for dependency in task_dependencies {
                        dependencies.entry(&task.task).or_default().push(&dependency);
                    }
                }
            }
        }

        let mut visited = HashSet::<&str>::new();
        let mut topological_order = Vec::<&str>::new();
        dfs(
            task_name,
            &dependencies,
            &mut visited,
            &mut topological_order,
        );

        let mut task_checks = Vec::<&str>::new();
        for dependency in topological_order {
            for check in checks.get(dependency).unwrap().iter() {
                task_checks.push(check);
            }
        }

        return task_checks;

        fn dfs<'a>(
            task: &'a str,
            dependencies: &HashMap<&'a str, Vec<&'a str>>,
            visited: &mut HashSet<&'a str>,
            topological_order: &mut Vec<&'a str>,
        ) {
            if visited.contains(task) {
                return;
            }
            visited.insert(task);

            if let Some(transitive_dependencies) = dependencies.get(task) {
                for transitive_dependency in transitive_dependencies {
                    dfs(
                        transitive_dependency,
                        dependencies,
                        visited,
                        topological_order,
                    );
                }
            }

            topological_order.push(task);
        }
    }

    fn dump_dependencies(
        &self,
        target_group: Option<String>,
    ) {
        println!("digraph dependencies {{");
        println!("    ranksep = 1");
        println!("    fontname = \"Helvetica,Arial,sans-serif\"");
        println!("    node [shape = rectangle, style = filled, fillcolor = \"#FFFFFF\"]\n");

        let external_dependencies = if let Some(target_group) = &target_group {
            self.group_dependencies(target_group).1
        } else {
            HashSet::new()
        };

        for (id, group) in self.deadlines.schedule.iter().enumerate() {
            if !group.group.starts_with("lab-") {
                continue;
            }

            println!("    subgraph cluster_{id} {{");
            println!("        label = \"{}\"", group.group);
            println!("        fontsize = \"20pt\"");
            println!("        style = \"rounded, filled\"");

            if group.group.as_str() < "lab-5" {
                println!("        fillcolor = \"#CADAF6\"");
            } else {
                println!("        fillcolor = \"#DDFDEA\"");
            }

            if let Some(target_group) = &target_group &&
                *target_group != group.group
            {
                for task in &group.tasks {
                    if external_dependencies.contains(task.task.as_str()) {
                        println!("        \"{}\"", task.task);
                    }
                }
            } else {
                for task in &group.tasks {
                    println!("        \"{}\"", task.task);
                }

                let dependencies = self.group_dependencies(&group.group).0;
                for (prerequisite, task) in &dependencies {
                    println!("        \"{prerequisite}\" -> \"{task}\"");
                }
            }

            println!("    }}\n");
        }

        if target_group.is_none() {
            println!("    \"allocator-cache\" -> \"elf\" [style = invis]");
            println!("    \"no-leaks-in-process\" -> \"memory-syscalls\" [style = invis]");
            println!("    \"allocator-cache\" -> \"block-cache\" [style = invis]");
        }

        println!("}}");
    }

    fn group_dependencies(
        &self,
        target_group: &str,
    ) -> (HashSet<(&str, &str)>, HashSet<&str>) {
        let re = Regex::new("--test ([^ ]*)").unwrap();
        let mut checks = HashMap::<String, &str>::new();
        let mut dependencies = HashSet::<(&str, &str)>::new();
        let mut external_dependencies = HashSet::<&str>::new();
        let mut internal_dependencies = HashSet::<&str>::new();

        for group in &self.deadlines.schedule {
            if target_group != group.group {
                continue;
            }

            for task in &group.tasks {
                for command in &task.check {
                    for check in re.captures_iter(command) {
                        let check = check.extract::<1>().1[0];
                        if !checks.contains_key(check) {
                            checks.insert(check.into(), &task.task);
                        }
                    }
                }

                internal_dependencies.insert(&task.task);
            }
        }

        for group in &self.deadlines.schedule {
            if target_group != group.group {
                continue;
            }

            for task in &group.tasks {
                if let Some(task_dependencies) = &task.dependencies {
                    for dependency in task_dependencies {
                        dependencies.insert((dependency, &task.task));
                    }
                } else if let Some(prerequisites) = &task.prerequisites {
                    for command in prerequisites {
                        for prerequisite in re.captures_iter(command) {
                            let dependency = checks.get(prerequisite.extract::<1>().1[0]);
                            if let Some(dependency) = dependency &&
                                dependency != &task.task
                            {
                                dependencies.insert((dependency, &task.task));
                            }
                        }
                    }
                }
            }
        }

        for (prerequisite, _) in &dependencies {
            if !internal_dependencies.contains(prerequisite) {
                external_dependencies.insert(prerequisite);
            }
        }

        (dependencies, external_dependencies)
    }

    fn dump_score(&self) {
        let mut mandatory_sum = 0;
        let mut bonus_sum = 0;
        for group in &self.deadlines.schedule {
            for task in &group.tasks {
                if let Some(is_bonus) = task.is_bonus &&
                    is_bonus
                {
                    bonus_sum += task.score;
                } else {
                    mandatory_sum += task.score;
                }
            }
        }

        println!(
            "100% score = {mandatory_sum}, bonus score = {bonus_sum}, max score = {}",
            mandatory_sum + bonus_sum,
        );
    }
}

fn read_config(opts: &Opts) -> Result<Config> {
    let config_path = format!("{}/{}", opts.original_repo.display(), CONFIG_NAME);
    let config = fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read config from {config_path}"))?;
    Ok(serde_yaml::from_str(config.as_str())?)
}

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opts {
    /// Path to the student repository.
    #[structopt(short = "s", long = "student-repo", default_value = ".")]
    student_repo: PathBuf,
    /// Path to the original repo.
    #[structopt(short = "o", long = "original-repo", default_value = ".")]
    original_repo: PathBuf,
    /// CI branch name (e.g. "submit/add")
    #[structopt(short = "b", long = "ci-branch-name")]
    ci_branch_name: String,
    /// Gitlab User ID
    #[structopt(short = "u", long = "user-id", default_value = ".")]
    user_id: String,

    #[structopt(long = "sandbox")]
    sandbox: bool,

    #[structopt(long = "dry-run")]
    dry_run: bool,

    #[structopt(long = "no-prerequisites-check")]
    no_prerequisites_check: bool,

    #[structopt(long = "dump-dependencies")]
    dump_dependencies: bool,

    #[structopt(long = "dump-group-dependencies")]
    dump_group_dependencies: Option<String>,

    #[structopt(long = "dump-score")]
    dump_score: bool,
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

fn check_task(
    opts: &Opts,
    task_checks: Vec<&str>,
) -> Result<()> {
    for line in task_checks {
        let mut cmd = if opts.sandbox {
            Command::new("sudo")
        } else {
            Command::new("sh")
        };

        if opts.sandbox {
            cmd.args(SUDO_PRESERVE_ENVS.iter().map(|e| format!("--preserve-env={}", e)))
                .args(["-u", "nobody", "sh"]);
        }

        cmd.args(["-cex"]).arg(line).current_dir(&opts.original_repo);

        println!(
            "> {} {}",
            cmd.get_program().to_str().unwrap(),
            cmd.get_args()
                .map(|s| format!("\"{}\"", s.to_str().unwrap()))
                .collect::<Vec<_>>()
                .join(" ")
        );

        let status = cmd.spawn()?.wait()?;

        if !status.success() {
            bail!("'{}' command terminated with error", line);
        }
    }

    Ok(())
}

fn submit_report(
    user_id: &str,
    task_name: &str,
    tester_token: &str,
    report_url: &str,
) -> Result<()> {
    let form = reqwest::blocking::multipart::Form::new()
        .text("user_id", user_id.to_string())
        .text("task", task_name.to_string())
        .text("token", tester_token.to_string());
    reqwest::blocking::Client::new()
        .post(report_url)
        .multipart(form)
        .send()?
        .error_for_status()?;
    Ok(())
}

fn do_main(opts: Opts) -> Result<()> {
    let task_name = parse_task_name(&opts.ci_branch_name).context("failed to parse task name")?;
    let config = read_config(&opts)?;

    if opts.dump_dependencies {
        config.dump_dependencies(None);
        return Ok(());
    }

    if opts.dump_group_dependencies.is_some() {
        config.dump_dependencies(opts.dump_group_dependencies);
        return Ok(());
    }

    if opts.dump_score {
        config.dump_score();
        return Ok(());
    }

    let default_allow_list = if opts.dry_run {
        Some(vec![])
    } else {
        None
    };

    if let Some(task) = config.find_task(task_name) {
        let orig_path = &opts.original_repo;
        let stud_path = &opts.student_repo;

        for pattern in &task
            .allow_list
            .clone()
            .or(config.allow_list.clone())
            .or(default_allow_list)
            .with_context(|| format!("no allow_list for task {task_name}"))?
        {
            let path = orig_path.join(pattern.as_str());

            for entry in glob(path.to_str().unwrap())? {
                let dst = entry?;

                let relative = if orig_path.as_os_str() == "." {
                    dst.as_path()
                } else {
                    dst.strip_prefix(orig_path).with_context(|| {
                        format!("orig_path: {}, dst: {}", orig_path.display(), dst.display())
                    })?
                };

                let src = stud_path.join(relative);

                println!("* copy {}", relative.display());
                if orig_path != stud_path {
                    let exists = fs::exists(&src).with_context(|| {
                        format!("failed to check presence of file {}", src.display())
                    })?;
                    if exists {
                        fs::copy(&src, &dst).with_context(|| {
                            format!("failed to copy {} to {}", src.display(), dst.display())
                        })?;
                    }
                }
            }
        }

        let task_checks = config.task_checks(&task.task, !opts.no_prerequisites_check);

        check_task(&opts, task_checks).context("task check failed")?;
        if opts.dry_run {
            return Ok(());
        }

        let tester_token = env::var(TESTER_TOKEN_ENV)
            .with_context(|| format!("envirnnment variable '{}' is not set", TESTER_TOKEN_ENV))?;
        submit_report(&opts.user_id, task_name, &tester_token, &config.report_url)
            .context("failed to submit report")
    } else {
        bail!("task {} is not defined in {}", task_name, CONFIG_NAME)
    }
}

fn main() {
    let args = Opts::from_args();

    if let Err(err) = do_main(args) {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}

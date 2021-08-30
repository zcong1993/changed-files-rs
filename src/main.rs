use clap::{crate_version, App, Arg};
use regex::Regex;
use std::{collections, env, path, thread};
use subprocess::{Exec, Redirection};

#[derive(Debug)]
struct Opt {
    last_commit: bool,
    with_ancestor: bool,
    changed_since: Option<String>,
}

#[derive(Debug)]
struct GitCmd {
    cwd: path::PathBuf,
    args: Vec<String>,
    reg: Option<Regex>,
}

fn get_changed_and_filter(cmd: GitCmd) -> Vec<String> {
    let capture = Exec::cmd("git")
        .args(&cmd.args)
        .cwd(&cmd.cwd)
        .stderr(Redirection::None)
        .capture();

    match capture {
        Ok(capture) => capture
            .stdout_str()
            .split("\n")
            .filter(|&x| {
                if x == "" {
                    return false;
                }

                match &cmd.reg {
                    Some(re) => re.is_match(x),
                    None => true,
                }
            })
            .map(|x| cmd.cwd.join(x).as_path().to_str().unwrap().to_string())
            .collect(),
        Err(e) => {
            println!("run error {}", e);
            vec![]
        }
    }
}

fn combine_unique(vecs: Vec<Vec<String>>) -> Vec<String> {
    let mut res = vec![];
    for v in &vecs {
        res.extend(v);
    }
    res.iter()
        .map(|x| x.to_string())
        .collect::<collections::HashSet<_>>()
        .into_iter()
        .collect()
}

fn find_changed_files(cwd: path::PathBuf, opt: &Opt, reg: Option<Regex>) -> Vec<String> {
    let mut cmds: Vec<GitCmd> = Vec::new();

    // staged
    cmds.push(GitCmd {
        cwd: cwd.clone(),
        args: Vec::from([
            "diff".to_string(),
            "--cached".to_string(),
            "--name-only".to_string(),
        ]),
        reg: reg.clone(),
    });

    // unstaged
    cmds.push(GitCmd {
        cwd: cwd.clone(),
        args: Vec::from([
            "ls-files".to_string(),
            "--other".to_string(),
            "--modified".to_string(),
            "--exclude-standard".to_string(),
        ]),
        reg: reg.clone(),
    });

    if !(opt.changed_since.is_none() && !opt.last_commit && !opt.with_ancestor) {
        if opt.last_commit {
            cmds.push(GitCmd {
                cwd: cwd.clone(),
                args: Vec::from([
                    "show".to_string(),
                    "--name-only".to_string(),
                    "--pretty=format:".to_string(),
                    "HEAD".to_string(),
                ]),
                reg: reg.clone(),
            });
        }

        let changed_since = opt.changed_since.clone().unwrap_or("HAED^".to_string());
        cmds.push(GitCmd {
            cwd: cwd.clone(),
            args: Vec::from(["diff".to_string(), "--name-only".to_string(), changed_since]),
            reg: reg.clone(),
        });
    }

    let mut children = vec![];

    for cmd in cmds {
        children.push(thread::spawn(move || get_changed_and_filter(cmd)));
    }

    let mut res = vec![];

    for child in children {
        res.push(child.join().unwrap());
    }

    combine_unique(res)
}

fn main() {
    let matches = App::new("changed-files-rs")
        .about("get git repo modified files")
        .arg(
            Arg::new("last_commit")
                .short('l')
                .long("last-commit")
                .about("If since lastCommit."),
        )
        .arg(
            Arg::new("with_ancestor")
                .short('w')
                .long("with-ancestor")
                .about("If with ancestor."),
        )
        .arg(
            Arg::new("changed_since")
                .short('s')
                .long("since")
                .about("Get changed since commit.")
                .takes_value(true),
        )
        .arg(
            Arg::new("folder")
                .long("folder")
                .about("If return folder path."),
        )
        .arg(
            Arg::new("filter")
                .short('f')
                .long("filter")
                .about("Filter regex."),
        )
        .arg(Arg::new("command").about("Command prefix.").index(1))
        .version(crate_version!())
        .get_matches();

    let o = &Opt {
        last_commit: matches.is_present("last_commit"),
        with_ancestor: matches.is_present("with_ancestor"),
        changed_since: matches.value_of("changed_since").map(|x| x.to_string()),
    };

    let command = matches.value_of("command");
    let reg = matches.value_of("filter").map(|x| Regex::new(x).unwrap());
    let cwd = env::current_dir().unwrap();

    let mut res = find_changed_files(cwd, o, reg);

    if matches.is_present("folder") {
        res = res
            .into_iter()
            .map(|x| {
                x.replace(
                    path::Path::new(&x).file_name().unwrap().to_str().unwrap(),
                    "",
                )
            })
            .collect();
    }

    let files_str = res.join(" ");

    match command {
        None => println!("{}", files_str),
        Some(command) => println!("{} {}", command, files_str),
    }
}

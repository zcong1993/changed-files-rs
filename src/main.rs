use clap::{crate_version, App, Arg};
use regex::Regex;
use std::{collections, env, error::Error, path, thread};
use subprocess::Exec;

#[derive(Debug)]
struct Opt {
    last_commit: bool,
    with_ancestor: bool,
    changed_since: Option<String>,
}

fn get_changed_and_filter(cwd: &path::Path, args: &[&str], re: &Option<Regex>) -> Vec<String> {
    let files: Vec<String> = Exec::cmd("git")
        .args(args)
        .cwd(cwd)
        .capture()
        .unwrap()
        .stdout_str()
        .split("\n")
        .filter(|x| {
            let r = re.clone();
            match r {
                Some(re) => re.is_match(x),
                None => true,
            }
        })
        .map(|x| cwd.join(x).as_path().to_str().unwrap().to_string())
        .collect();
    files
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

fn find_changed_files(cwd: &path::PathBuf, opt: &Opt, reg: &Option<Regex>) -> Vec<String> {
    if opt.last_commit {
        get_changed_and_filter(
            cwd,
            &["show", "--name-only", "--pretty=format:", "HEAD"],
            reg,
        )
    } else {
        vec![]
    }
}

fn main() {
    let matches = App::new("cf-rs")
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

    let command = matches.value_of("command").unwrap_or("");
    let reg = matches.value_of("filter").map(|x| Regex::new(x).unwrap());
    let cwd = env::current_dir().unwrap();

    let res = find_changed_files(&cwd, o, &reg);

    println!("o: {:?}", o);
    println!("command: {:?}", res);
}

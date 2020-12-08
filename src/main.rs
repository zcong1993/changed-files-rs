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

fn get_changed_and_filter(cwd: &path::Path, args: &[&str], re: &Option<Regex>) -> Vec<String> {
    let capture = Exec::cmd("git")
        .args(args)
        .cwd(cwd)
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
                let r = re.clone();
                match r {
                    Some(re) => re.is_match(x),
                    None => true,
                }
            })
            .map(|x| cwd.join(x).as_path().to_str().unwrap().to_string())
            .collect(),
        Err(e) => {
            println!("run error {}", e);
            return vec![];
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

fn find_changed_files(cwd: &path::PathBuf, opt: &Opt, reg: &Option<Regex>) -> Vec<String> {
    if opt.changed_since.is_none() && !opt.last_commit && !opt.with_ancestor {
        let c1 = cwd.clone();
        let c2 = cwd.clone();
        let r1 = reg.clone();
        let r2 = reg.clone();
        let staged_t = thread::spawn(move || {
            get_changed_and_filter(c1.as_path(), &["diff", "--cached", "--name-only"], &r1)
        });
        let unstaged_t = thread::spawn(move || {
            get_changed_and_filter(
                c2.as_path(),
                &["ls-files", "--other", "--modified", "--exclude-standard"],
                &r2,
            )
        });

        return combine_unique(vec![staged_t.join().unwrap(), unstaged_t.join().unwrap()]);
    }

    if opt.last_commit {
        return get_changed_and_filter(
            cwd,
            &["show", "--name-only", "--pretty=format:", "HEAD"],
            reg,
        );
    }

    let c1 = cwd.clone();
    let c2 = cwd.clone();
    let c3 = cwd.clone();
    let r1 = reg.clone();
    let r2 = reg.clone();
    let r3 = reg.clone();

    let changed_since = opt.changed_since.clone().unwrap_or("HAED^".to_string());

    let committed_t = thread::spawn(move || {
        get_changed_and_filter(
            c1.as_path(),
            &[
                "diff",
                "--name-only",
                format!("{}...HEAD", changed_since.as_str()).as_str(),
            ],
            &r1,
        )
    });
    let staged_t = thread::spawn(move || {
        get_changed_and_filter(c2.as_path(), &["diff", "--cached", "--name-only"], &r2)
    });
    let unstaged_t = thread::spawn(move || {
        get_changed_and_filter(
            c3.as_path(),
            &["ls-files", "--other", "--modified", "--exclude-standard"],
            &r3,
        )
    });

    combine_unique(vec![
        committed_t.join().unwrap(),
        staged_t.join().unwrap(),
        unstaged_t.join().unwrap(),
    ])
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

    let command = matches.value_of("command");
    let reg = matches.value_of("filter").map(|x| Regex::new(x).unwrap());
    let cwd = env::current_dir().unwrap();

    let mut res = find_changed_files(&cwd, o, &reg);

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

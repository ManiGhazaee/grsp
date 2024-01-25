#![allow(dead_code)]
use std::{
    cmp::min,
    fmt::Error,
    fs::{read_dir, DirEntry, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

use rayon::{
    iter::{
        IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelBridge,
        ParallelIterator,
    },
    str::ParallelString,
};
use regex::Regex;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Debug)]
struct Match {
    ln: usize,
    col: Vec<usize>,
    ln_str: String,
}

#[derive(Debug)]
struct Matches {
    path: PathBuf,
    m: Vec<Match>,
}

fn main() {
    let inst = Instant::now();

    // let mut file_string = String::new();
    // let path = "../typescript-typechecker/";
    let path = "../typescript-typechecker/";
    let pat = "fn";
    let pat_len = pat.len();

    // let mut paths: Vec<PathBuf> = Vec::new();
    // get_paths(&mut paths, Path::new(path).into());

    // for path in paths {
    //     if path.file_name().unwrap() == ".gitignore" {
    //         println!("{:?}", path);
    //     }
    // }

    // let matches_res: Arc<Mutex<Vec<Matches>>> = Arc::new(Mutex::new(Vec::new()));
    // paths.par_iter().for_each(|path| {
    //     let mut file_string = String::new();
    //     let mut file = match OpenOptions::new().read(true).open(&path) {
    //         Ok(v) => v,
    //         Err(_) => return,
    //     };
    //     if is_binary(&mut file) {
    //         return;
    //     }
    //     if let Err(_) = file.read_to_string(&mut file_string) {
    //         return;
    //     }
    //     let matches = find_matches_par(&mut file_string, &pat, pat_len);
    //     matches_res.lock().unwrap().push(Matches {
    //         path: path.to_owned(),
    //         m: matches,
    //     });
    // });

    // match_recursively(&Path::new(path), &mut file_string, pat, pat_len);
    match_par(Path::new(path), pat, pat_len);
    // let matches_res: Arc<Mutex<Vec<Matches>>> = Arc::new(Mutex::new(Vec::new()));
    // match_par_2(&Path::new(path), pat, pat_len, &matches_res);
    // matches_res.lock().unwrap().par_iter().for_each(|matches| {
    //     print_matches(&matches.m, &matches.path, pat_len);
    // });
    println!("{}ms", inst.elapsed().as_millis())
}

fn get_paths(result: &mut Vec<PathBuf>, path: PathBuf) {
    if path.is_file() {
        result.push(path);
    } else if path.is_dir() {
        for entry in read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            get_paths(result, path);
        }
    }
}

fn match_par(path: &Path, pat: &str, pat_len: usize) {
    if path.is_file() {
        let mut file_string = String::new();
        let mut file = match OpenOptions::new().read(true).open(&path) {
            Ok(v) => v,
            Err(_) => return,
        };
        if is_binary(&mut file) {
            return;
        }
        if let Err(_) = file.read_to_string(&mut file_string) {
            return;
        }
        let matches = find_matches_par(&file_string, &pat, pat_len);
        print_matches(&matches, &path, pat_len);
        return;
    } else if path.is_dir() {
        read_dir(path)
            .unwrap()
            .into_iter()
            .par_bridge()
            .for_each(|entry| {
                let path = entry.unwrap().path();
                match_par(&path, pat, pat_len);
            });
        // let entires: Vec<_> = read_dir(path).unwrap().filter_map(|e| e.ok()).collect();
        // entires.par_iter().for_each(|entry| {
        //     let path = entry.path();
        //     match_par(&path, pat, pat_len);
        // })
    }
}

fn match_par_2(path: &Path, pat: &str, pat_len: usize, matches_res: &Arc<Mutex<Vec<Matches>>>) {
    if path.is_file() {
        let mut file_string = String::new();
        let mut file = match OpenOptions::new().read(true).open(&path) {
            Ok(v) => v,
            Err(_) => return,
        };
        if is_binary(&mut file) {
            return;
        }
        if let Err(_) = file.read_to_string(&mut file_string) {
            return;
        }
        let matches = find_matches_par(&file_string, &pat, pat_len);
        matches_res.lock().unwrap().push(Matches {
            m: matches,
            path: path.into(),
        });
    } else if path.is_dir() {
        read_dir(path)
            .unwrap()
            .into_iter()
            .par_bridge()
            .for_each(|entry| {
                let path = entry.unwrap().path();
                match_par_2(&path, pat, pat_len, matches_res);
            });
        // let entires: Vec<_> = read_dir(path).unwrap().filter_map(|e| e.ok()).collect();
        // entires.par_iter().for_each(|entry| {
        //     let path = entry.path();
        //     match_par_2(&path, pat, pat_len, matches_res);
        // })
    }
}

fn find_matches(file_string: &String, pat: &str, pat_len: usize) -> Vec<Match> {
    let mut matches: Vec<Match> = Vec::new();
    for (line_idx, line) in file_string.lines().enumerate() {
        if pat_len > line.len() {
            continue;
        }
        for i in 0..=line.len() - pat_len {
            if &line[i..i + pat_len] == pat {
                if let Some(last) = matches.last() {
                    if last.ln == line_idx + 1 {
                        let last_idx = matches.len() - 1;
                        matches[last_idx].col.push(i + 1);
                        continue;
                    }
                }
                matches.push(Match {
                    ln: line_idx + 1,
                    col: Vec::from([i + 1]),
                    ln_str: line.to_string(),
                });
            }
        }
    }
    matches
}

fn find_matches_par(file_string: &str, pat: &str, pat_len: usize) -> Vec<Match> {
    let matches: Arc<Mutex<Vec<Match>>> = Arc::new(Mutex::new(Vec::new()));
    file_string
        .lines()
        .enumerate()
        .par_bridge()
        .for_each(|(line_idx, line)| {
            if pat_len > line.len() {
                return;
            }
            let mut col: Vec<usize> = Vec::new();
            let mut i = 0;
            while i < line.len() - pat_len {
                if &line[i..i + pat_len] == pat {
                    col.push(i + 1);
                }
                i += 1;
            }
            if col.is_empty() {
                return;
            }
            matches.lock().unwrap().push(Match {
                ln: line_idx + 1,
                col,
                ln_str: line.to_string(),
            })
        });
    Arc::try_unwrap(matches).unwrap().into_inner().unwrap()
}

fn print_matches(matches: &Vec<Match>, path: &Path, pat_len: usize) {
    if matches.is_empty() {
        return;
    }

    let stdout = StandardStream::stdout(ColorChoice::Always);
    let mut lock = stdout.lock();
    lock.set_color(ColorSpec::new().set_fg(Some(Color::Red)))
        .unwrap();
    writeln!(lock, "{}", path.to_str().unwrap()).unwrap();
    for m in matches {
        lock.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .unwrap();
        write!(lock, "{}", m.ln).unwrap();
        lock.reset().unwrap();
        write!(lock, ":").unwrap();
        let mut j = 0;
        for i in 0..m.col.len() {
            write!(lock, "{}", m.ln_str[j..m.col[i] - 1].to_string()).unwrap();
            lock.set_color(ColorSpec::new().set_fg(Some(Color::Green)))
                .unwrap();
            write!(
                lock,
                "{}",
                m.ln_str[m.col[i] - 1..m.col[i] - 1 + pat_len].to_string()
            )
            .unwrap();
            lock.reset().unwrap();
            j = m.col[i] - 1 + pat_len;
        }
        write!(
            lock,
            "{}",
            m.ln_str[m.col[m.col.len() - 1] - 1 + pat_len..].to_string()
        )
        .unwrap();
        writeln!(lock, "").unwrap();
    }
    writeln!(lock, "").unwrap();
}

fn is_binary(file: &mut File) -> bool {
    let mut buffer = [0u8; 1024];

    if let Ok(bytes_read) = file.read(&mut buffer) {
        for i in 0..bytes_read {
            if buffer[i] == 0
                || (buffer[i] < 32 && buffer[i] != 9 && buffer[i] != 10 && buffer[i] != 13)
            {
                return true;
            }
        }
    } else {
        return true;
    }

    if let Err(_) = file.seek(SeekFrom::Start(0)) {
        return true;
    }

    false
}

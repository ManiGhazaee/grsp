#![allow(dead_code)]
use std::{
    cmp::min,
    fs::{self, read_dir, File, OpenOptions},
    io::{stdout, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

use rayon::iter::{
    IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, StandardStreamLock, WriteColor};

#[derive(Debug)]
struct Match {
    ln: usize,
    col: Vec<usize>,
    buf_s: Vec<usize>,
    buf_e: Vec<usize>,
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
    let pat = "Vec";
    let pat_len = pat.len();

    // let mut paths: Vec<PathBuf> = Vec::new();
    // get_paths(&mut paths, Path::new(path).into());

    // paths.par_iter_mut().for_each(|path| {
    //     let mut file_string = String::new();
    //     let mut file = match OpenOptions::new().read(true).open(&path) {
    //         Ok(v) => v,
    //         Err(_) => return,
    //     };
    //     if let Err(_) = file.read_to_string(&mut file_string) {
    //         return;
    //     }
    //     if file_string[0..min(file_string.len(), 1024)].contains('\x00') {
    //         return;
    //     }
    //     let matches = find_matches(&mut file_string, &pat, pat_len);
    //     print_matches(&matches, &path);
    // });

    // match_recursively(&Path::new(path), &mut file_string, pat, pat_len);
    let matches_res: Arc<Mutex<Vec<Matches>>> = Arc::new(Mutex::new(Vec::new()));
    match_par_2(&Path::new(path), pat, pat_len, &matches_res);
    matches_res.lock().unwrap().par_iter().for_each(|matches| {
        print_matches(&matches.m, &matches.path, pat_len);
    });
    println!("{}ms", inst.elapsed().as_millis())
}

fn get_paths(result: &mut Vec<PathBuf>, path: PathBuf) {
    if path.is_file() {
        result.push(path);
    } else if path.is_dir() {
        for entry in read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                get_paths(result, path);
            } else if path.is_file() {
                result.push(path);
            }
        }
    }
}

fn match_recursively(path: &Path, file_string: &mut String, pat: &str, pat_len: usize) {
    file_string.clear();
    if path.is_file() {
        let mut file = match OpenOptions::new().read(true).open(&path) {
            Ok(v) => v,
            Err(_) => return,
        };
        if let Err(_) = file.read_to_string(file_string) {
            return;
        }
        if file_string[0..min(file_string.len(), 1024)].contains('\x00') {
            return;
        }
        let matches = find_matches(file_string, &pat, pat_len);
        print_matches(&matches, &path, pat_len);
        file_string.clear();
        return;
    } else if path.is_dir() {
        for entry in read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                match_recursively(&path, file_string, pat, pat_len);
            } else if path.is_file() {
                let mut file = match OpenOptions::new().read(true).open(&path) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if let Err(_) = file.read_to_string(file_string) {
                    continue;
                }
                if file_string[0..min(file_string.len(), 1024)].contains('\x00') {
                    return;
                }

                let matches = find_matches(file_string, &pat, pat_len);
                print_matches(&matches, &path, pat_len);
                file_string.clear()
            }
        }
    }
}

fn match_recursively_par(path: &Path, pat: &str, pat_len: usize) {
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

        let matches = find_matches(&file_string, &pat, pat_len);
        print_matches(&matches, &path, pat_len);
        return;
    } else if path.is_dir() {
        let entires: Vec<_> = read_dir(path).unwrap().filter_map(|e| e.ok()).collect();
        entires.par_iter().for_each(|entry| {
            let path = entry.path();

            // if path.is_dir() {
            match_recursively_par(&path, pat, pat_len);
            // }
            // let mut file = match OpenOptions::new().read(true).open(&path) {
            //     Ok(v) => v,
            //     Err(_) => return,
            // };
            // if is_binary(&mut file) {
            //     return;
            // }
            // if let Err(_) = file.read_to_string(&mut file_string) {
            //     return;
            // }

            // let matches = find_matches(&file_string, &pat, pat_len);
            // print_matches(&matches, &path);
        })
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

        let matches = find_matches(&file_string, &pat, pat_len);
        let mut lock = matches_res.lock().unwrap();
        lock.push(Matches {
            m: matches,
            path: path.into(),
        });
        return;
    } else if path.is_dir() {
        let entires: Vec<_> = read_dir(path).unwrap().filter_map(|e| e.ok()).collect();
        entires.par_iter().for_each(|entry| {
            let path = entry.path();
            match_par_2(&path, pat, pat_len, matches_res);
        })
    }
}

fn find_matches(file_string: &String, pat: &str, pat_len: usize) -> Vec<Match> {
    let mut matches: Vec<Match> = Vec::new();
    let mut buf_idx = 0;
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
                        matches[last_idx].buf_s.push(buf_idx);
                        matches[last_idx].buf_e.push(buf_idx + pat_len);
                        buf_idx += 1;
                        continue;
                    }
                }
                matches.push(Match {
                    ln: line_idx + 1,
                    col: Vec::from([i + 1]),
                    buf_s: Vec::from([buf_idx]),
                    buf_e: Vec::from([buf_idx + pat_len]),
                    ln_str: line.to_string(),
                });
            }
            buf_idx += 1;
        }
    }
    matches
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

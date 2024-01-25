#![allow(dead_code)]
use std::{
    cmp::min,
    fs::{read_dir, File, OpenOptions},
    io::{stdout, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};

#[derive(Debug)]
struct Match {
    ln: usize,
    col: usize,
    buf_s: usize,
    buf_e: usize,
    ln_str: String,
}

fn main() {
    let inst = Instant::now();

    let mut file_string = String::new();
    let path = "../typescript-typechecker/";
    let pat = "Vec::";
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
    match_recursively_par(&Path::new(path), pat, pat_len);
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
        print_matches(&matches, &path);
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
                print_matches(&matches, &path);
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
        print_matches(&matches, &path);
        return;
    }
    let entires: Vec<_> = read_dir(path).unwrap().filter_map(|e| e.ok()).collect();
    entires.par_iter().for_each(|entry| {
        let mut file_string = String::new();

        let path = entry.path();

        if path.is_dir() {
            match_recursively_par(&path, pat, pat_len);
        }
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
        print_matches(&matches, &path);
    })
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
                matches.push(Match {
                    ln: line_idx + 1,
                    col: i + 1,
                    buf_s: buf_idx,
                    buf_e: buf_idx + pat_len,
                    ln_str: line.to_owned(),
                });
            }
            buf_idx += 1;
        }
    }
    matches
}

fn print_matches(matches: &Vec<Match>, path: &Path) {
    if !matches.is_empty() {
        println!("{}", path.to_str().unwrap());
        let mut lock = stdout().lock();
        for m in matches {
            writeln!(lock, "{}:{}", m.ln, m.ln_str).unwrap();
        }
    }
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

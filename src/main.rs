#![allow(dead_code)]
use std::{
    cmp::min,
    fs::{read_dir, File, OpenOptions},
    io::Read,
    path::Path,
    time::Instant,
};

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
    let pat = "Vec::";

    // let path = "../typescript-typechecker";
    match_recursively(
        &Path::new("../typescript-typechecker"),
        &mut file_string,
        pat,
        &pat.len(),
    );

    println!("{}ms", inst.elapsed().as_millis())
}

fn match_recursively(path: &Path, file_string: &mut String, pat: &str, pat_len: &usize) {
    if path.is_file() {
        let mut file = match OpenOptions::new().read(true).open(&path) {
            Ok(v) => v,
            Err(_) => return,
        };
        if let Err(_) = file.read_to_string(file_string) {
            return;
        }
        if file_string[0..min(file_string.len(), 1024)]
            .as_bytes()
            .contains(&b'\x00')
        {
            return;
        }
        let matches = find_matches(file_string, &pat, &pat_len);
        if !matches.is_empty() {
            println!("{:?}:{}", matches[0].ln, matches[0].ln_str);
        }
        return;
    }
    for entry in read_dir(path).unwrap().into_iter() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            match_recursively(&path, file_string, pat, pat_len);
        }
        let mut file = match OpenOptions::new().read(true).open(&path) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Err(_) = file.read_to_string(file_string) {
            continue;
        }
        if file_string[0..min(file_string.len(), 1024)]
            .as_bytes()
            .contains(&b'\x00')
        {
            return;
        }

        let matches = find_matches(file_string, &pat, &pat_len);
        if !matches.is_empty() {
            println!("{}", path.to_str().unwrap());
            for m in matches {
                println!("{:?}:{}", m.ln, m.ln_str);
            }
        }
        file_string.clear()
    }
}

fn find_matches(file_string: &mut String, pat: &str, pat_len: &usize) -> Vec<Match> {
    let mut matches: Vec<Match> = Vec::new();
    let mut buf_idx = 0;
    for (line_idx, line) in file_string.lines().enumerate() {
        if pat_len > &line.len() {
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

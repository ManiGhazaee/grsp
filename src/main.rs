#![allow(dead_code)]

use std::{env, path::Path, time::Instant};

use grsp::match_par;

fn main() {
    let inst = Instant::now();
    let args: Vec<String> = env::args().collect();
    let path = if let Some(path) = args.get(2) {
        path
    } else {
        "./"
    };
    let pat = if let Some(pat) = args.get(1) {
        if pat.is_empty() {
            panic!("Pattern is empty");
        }
        pat
    } else {
        panic!("Pattern not provided");
    };
    let pat_len = pat.len();

    match_par(Path::new(path), pat, pat_len);

    println!("{}ms", inst.elapsed().as_millis())
}

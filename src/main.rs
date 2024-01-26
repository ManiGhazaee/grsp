#![allow(dead_code)]

use std::{env, path::Path, time::Instant};

use grsp::search_par;

fn main() {
    let inst = Instant::now();
    let args: Vec<String> = env::args().collect();
    let path = match args.get(2) {
        Some(path) => path,
        _ => "./",
    };
    let pat = match args.get(1) {
        Some(pat) if !pat.is_empty() => pat.as_bytes(),
        _ => panic!("String not provided"),
    };
    let pat_len = pat.len();

    search_par(Path::new(path), pat, pat_len);

    println!("{}ms", inst.elapsed().as_millis());
}

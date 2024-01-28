use std::{
    fs::{read_dir, File},
    io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write},
    path::Path,
};

use rayon::prelude::*;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

#[derive(Debug)]
pub struct Match {
    pub ln: usize,
    pub col: Vec<usize>,
    pub ln_str: Vec<u8>,
}

pub fn search_par(path: &Path, pat: &[u8], pat_len: usize) {
    if path.is_file() {
        let mut file = match File::open(path) {
            Ok(v) => v,
            _ => return,
        };
        if is_binary(&mut file) {
            return;
        }
        let reader = BufReader::new(file);
        let mut matches: Vec<Match> = Vec::new();
        for (line_idx, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(l) => l,
                _ => continue,
            };
            if pat_len > line.len() {
                continue;
            }
            let line = line.as_bytes();
            let mut col: Vec<usize> = Vec::new();
            let mut i = 0;
            while i <= line.len() - pat_len {
                if &line[i..i + pat_len] == pat {
                    col.push(i + 1);
                    i += pat_len;
                    continue;
                }
                i += 1;
            }
            if col.is_empty() {
                continue;
            }
            matches.push(Match {
                ln: line_idx + 1,
                col,
                ln_str: line.to_vec(),
            });
        }
        let _ = print_matches(&matches, &path, pat_len);
    } else if path.is_dir() {
        if let Ok(d) = read_dir(path) {
            d.into_iter().par_bridge().for_each(|entry| {
                let path = match entry {
                    Ok(e) => e.path(),
                    _ => return,
                };
                search_par(&path, pat, pat_len);
            });
        } else {
            return;
        }
    };
}

pub fn print_matches(matches: &Vec<Match>, path: &Path, pat_len: usize) -> io::Result<()> {
    if matches.is_empty() {
        return Ok(());
    }

    let mut binding = ColorSpec::new();
    let cyan = binding.set_fg(Some(Color::Cyan));
    let mut binding = ColorSpec::new();
    let red = binding.set_fg(Some(Color::Red));
    let mut binding = ColorSpec::new();
    let green = binding.set_fg(Some(Color::Green));

    let bw = BufferWriter::stdout(ColorChoice::Always);
    let mut b = bw.buffer();
    b.set_color(&green)?;
    writeln!(b, "{}", path.to_str().unwrap_or(""))?;

    for m in matches {
        b.set_color(&cyan)?;
        write!(b, "{}", m.ln)?;
        b.reset()?;
        write!(b, ":")?;
        let mut j = 0;
        for i in m.col.iter() {
            let col = i - 1;
            b.write(&m.ln_str[j..col])?;
            b.set_color(&red)?;
            b.write(&m.ln_str[col..col + pat_len])?;
            b.reset()?;
            j = col + pat_len;
        }
        b.write(&m.ln_str[m.col[m.col.len() - 1] - 1 + pat_len..])?;
        write!(b, "\n")?;
    }
    write!(b, "\n")?;
    bw.print(&b)?;

    Ok(())
}

pub fn is_binary(file: &mut File) -> bool {
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

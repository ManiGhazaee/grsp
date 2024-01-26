use std::{
    fs::{read_dir, File},
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

#[derive(Debug)]
pub struct Match {
    ln: usize,
    col: Vec<usize>,
    ln_str: Vec<u8>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Matches {
    path: PathBuf,
    m: Vec<Match>,
}

pub fn get_paths(result: &mut Vec<PathBuf>, path: PathBuf) {
    if path.is_file() {
        result.push(path);
    } else if path.is_dir() {
        for entry in read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            get_paths(result, path);
        }
    }
}

pub fn match_par(path: &Path, pat: &[u8], pat_len: usize) {
    if path.is_file() {
        let mut file = match File::open(path) {
            Ok(v) => v,
            Err(_) => return,
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
        print_matches(&matches, &path, pat_len);
    } else if path.is_dir() {
        read_dir(path)
            .unwrap()
            .into_iter()
            .par_bridge()
            .for_each(|entry| {
                let path = entry.unwrap().path();
                match_par(&path, pat, pat_len);
            });
    }
}

// pub fn match_par_x(path: &Path, pat: &[u8], pat_len: usize) {
//     if path.is_file() {
//         let mut file_string = String::new();
//         let mut file = match OpenOptions::new().read(true).open(&path) {
//             Ok(v) => v,
//             Err(_) => return,
//         };
//         if is_binary(&mut file) {
//             return;
//         }
//         if let Err(_) = file.read_to_string(&mut file_string) {
//             return;
//         }
//         let matches = find_matches_par(&file_string, &pat, pat_len);
//         print_matches(&matches, &path, pat_len);
//     } else if path.is_dir() {
//         read_dir(path)
//             .unwrap()
//             .into_iter()
//             .par_bridge()
//             .for_each(|entry| {
//                 let path = entry.unwrap().path();
//                 match_par(&path, pat, pat_len);
//             });
//     }
// }

// pub fn _match_par(path: &Path, pat: &[u8], pat_len: usize, matches_res: &Arc<Mutex<Vec<Matches>>>) {
//     if path.is_file() {
//         let mut file_string = String::new();
//         let mut file = match OpenOptions::new().read(true).open(&path) {
//             Ok(v) => v,
//             Err(_) => return,
//         };
//         if is_binary(&mut file) {
//             return;
//         }
//         if let Err(_) = file.read_to_string(&mut file_string) {
//             return;
//         }
//         let matches = find_matches_par(&file_string, &pat, pat_len);
//         matches_res.lock().unwrap().push(Matches {
//             m: matches,
//             path: path.into(),
//         });
//     } else if path.is_dir() {
//         read_dir(path)
//             .unwrap()
//             .into_iter()
//             .par_bridge()
//             .for_each(|entry| {
//                 let path = entry.unwrap().path();
//                 _match_par(&path, pat, pat_len, matches_res);
//             });
//     }
// }

// pub fn find_matches_par(file_string: &str, pat: &[u8], pat_len: usize) -> Vec<Match> {
//     let matches: Arc<Mutex<Vec<Match>>> = Arc::new(Mutex::new(Vec::new()));
//     file_string
//         .lines()
//         .enumerate()
//         .par_bridge()
//         .for_each(|(line_idx, line)| {
//             if pat_len > line.len() {
//                 return;
//             }
//             let line = line.as_bytes();
//             let mut col: Vec<usize> = Vec::new();
//             let mut i = 0;
//             while i <= line.len() - pat_len {
//                 if &line[i..i + pat_len] == pat {
//                     col.push(i + 1);
//                 }
//                 i += 1;
//             }
//             if col.is_empty() {
//                 return;
//             }
//             matches.lock().unwrap().push(Match {
//                 ln: line_idx + 1,
//                 col,
//                 ln_str: String::from_utf8_lossy(line).to_string(),
//             })
//         });
//     Arc::try_unwrap(matches).unwrap().into_inner().unwrap()
// }

pub fn print_matches(matches: &Vec<Match>, path: &Path, pat_len: usize) {
    if matches.is_empty() {
        return;
    }

    let mut binding = ColorSpec::new();
    let cyan = binding.set_fg(Some(Color::Cyan));
    let mut binding = ColorSpec::new();
    let red = binding.set_fg(Some(Color::Red));
    let mut binding = ColorSpec::new();
    let green = binding.set_fg(Some(Color::Green));

    let bw = BufferWriter::stdout(ColorChoice::Always);
    let mut b = bw.buffer();
    b.set_color(&green).unwrap();
    writeln!(b, "{}", path.to_str().unwrap()).unwrap();
    for m in matches {
        b.set_color(&cyan).unwrap();
        write!(b, "{}", m.ln).unwrap();
        b.reset().unwrap();
        write!(b, ":").unwrap();
        let mut j = 0;
        for i in m.col.iter() {
            let col = i - 1;
            b.write(&m.ln_str[j..col]).unwrap();
            // write!(b, "{}", &m.ln_str[j..col]).unwrap();
            b.set_color(&red).unwrap();
            b.write(&m.ln_str[col..col + pat_len]).unwrap();
            // write!(b, "{}", &m.ln_str[col..col + pat_len]).unwrap();
            b.reset().unwrap();
            j = col + pat_len;
        }
        b.write(&m.ln_str[m.col[m.col.len() - 1] - 1 + pat_len..])
            .unwrap();
        // write!(b, "{}", &m.ln_str[m.col[m.col.len() - 1] - 1 + pat_len..]).unwrap();
        write!(b, "\n").unwrap();
    }
    write!(b, "\n").unwrap();
    bw.print(&b).unwrap();
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

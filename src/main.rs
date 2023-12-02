use std::fmt::Display;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Instant;

use md5::Digest;
use rayon::prelude::*;

#[derive(Clone, Debug)]
struct PathWithInfo(PathBuf, u64);

#[derive(Clone, Debug)]
struct HashingResults(PathBuf, String);

impl Display for HashingResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{}: {}", self.0.to_str().unwrap(), self.1))
    }
}

fn recurse_dirs(path: &Path, paths: &mut Vec<PathWithInfo>) -> Vec<PathWithInfo> {
    for thing in std::fs::read_dir(path).unwrap() {
        let thing = thing.unwrap();
        if thing.file_type().unwrap().is_dir() {
            recurse_dirs(&*thing.path(), paths);
        } else {
            let size = File::open(thing.path()).unwrap().metadata().unwrap().len();
            paths.push(PathWithInfo(thing.path(), size));
        }
    }
    paths.clone()
}

fn main() {
    let files = recurse_dirs(Path::new("."), &mut Vec::new());
    let (sender, receiver) = channel();
    files.into_par_iter().for_each_with(sender, |s, file| {
        let path = file.0.clone();
        let path = path.as_path();
        let start = Instant::now();
        let mut hasher = md5::Md5::new();
        io::copy(&mut File::open(path).unwrap(), &mut hasher).expect("TODO: panic message");
        let hex_hash = base16ct::lower::encode_string(&hasher.finalize());
        let result = HashingResults(file.0, hex_hash);
        let duration = Instant::now() - start;
        println!("{:?} has finished hashing in {} seconds, with a checksum of {}", result.0, duration.as_secs(), result.1);
        s.send(result).expect("TODO: panic message");
    });

    let mut res: Vec<_> = receiver.iter().collect();
    res.sort_by(|file1, file2| file1.0.cmp(&file2.0));
    let mut results_file = File::create("md5sums.txt").unwrap();
    results_file.write(res.iter().map(|result| result.to_string()).collect::<Vec<String>>().join("\n").as_ref()).unwrap();
}

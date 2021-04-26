use clap::Clap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::env;

#[derive(Clap, Debug)]
#[clap(
    name = "CPSTT",
    version = "1.0.0",
    author = "xryuseix",
    about = "Competitive Programming Stress Test Tools"
)]
struct Opts {}

fn main() {
    Opts::parse();
    let path = env::current_exe().unwrap();
    print_logo(path);
}

fn print_logo(mut path: PathBuf) {
    for _i in 0..3 {
        path.pop();
    }
    path.push("logo.txt");
    let f = File::open(path).unwrap();
    for line in BufReader::new(f).lines() {
        println!("{}", line.unwrap());
    }
}

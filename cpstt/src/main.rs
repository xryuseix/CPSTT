use clap::Clap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use std::env;
use std::path::Path;

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
    print_logo();
}

fn print_logo() {
    let mut path = env::current_exe().unwrap();
    path.pop();
    path.pop();
    path.pop();
    path.push("logo.txt");
    let f = File::open(path).unwrap();
    let reader = BufReader::new(f);
    for line in reader.lines() {
        let line = line.unwrap();
        println!("{}", line);
    }
}

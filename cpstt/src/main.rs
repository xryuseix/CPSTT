use clap::Clap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::env;
use anyhow::{Result};

#[derive(Clap, Debug)]
#[clap(
    name = "CPSTT",
    version = "1.0.0",
    author = "xryuseix",
    about = "Competitive Programming Stress Test Tools"
)]
struct Opts {}

fn main() -> Result<()> {
    Opts::parse();
    let path = env::current_exe().unwrap();
    print_logo(path)?;
    Ok(())
}

/**
 * CPSTTのロゴを出力
 * @param path 実行形式ファイルへの絶対パス
 * @return 正常終了の有無
*/
fn print_logo(mut path: PathBuf) -> Result<()> {
    for _i in 0..3 {
        path.pop();
    }
    path.push("logo.txt");
    let file = File::open(path)?;
    for line in BufReader::new(file).lines() {
        println!("{}", line.unwrap());
    }
    Ok(())
}

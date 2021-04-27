use ansi_term::Colour::{Red, Yellow};
use anyhow::{bail, Result};
use clap::Clap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;

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
    /* プロジェクトのルートパスを取得 */
    let root_path = get_root_path();
    /* ロゴを出力 */
    print_logo(root_path.clone())?;
    /* generatorを実行 */
    generator(root_path.clone())?;
    Ok(())
}

/**
 * プロジェクトのルートパスを取得
 * @return プロジェクトのルートパス
 */
fn get_root_path() -> PathBuf {
    let mut exec_path = env::current_exe().unwrap();
    for _i in 0..3 {
        exec_path.pop();
    }
    exec_path
}

/**
 * CPSTTのロゴを出力
 * @param path 実行形式ファイルへの絶対パス
 * @return 正常終了の有無
 */
fn print_logo(mut root_path: PathBuf) -> Result<()> {
    root_path.push("logo.txt");
    let file = File::open(root_path)?;
    for line in BufReader::new(file).lines() {
        println!("{}", line.unwrap());
    }
    Ok(())
}

/**
 * generatorを実行
 * @param path 実行形式ファイルへの絶対パス
 * @return 正常終了の有無
 */
fn generator(mut generator_path: PathBuf) -> Result<()> {
    generator_path.push("test/generator_err.cpp");
    println!("{}", generator_path.display());
    exec_cpp_program(generator_path)?;
    Ok(())
}

/*
 * C++のファイルを指定し，そのプログラムを実行する
 */
fn exec_cpp_program(root_path: PathBuf) -> Result<()> {
    let compile_output = Command::new("g++")
        .args(&[
            "-std=c++1z",
            "-O3",
            "-fsanitize=undefined",
            "-I",
            ".",
            root_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to compile C++ program");

    let compile_stdout = String::from_utf8_lossy(&compile_output.stdout);
    let compile_stderr = String::from_utf8_lossy(&compile_output.stderr);
    println!("output: {}", compile_stderr);
    if compile_stderr != String::from("") {
        println!("{}", compile_stderr);
        print_error(String::from("It seems compile error"));
        bail!("Some Error is occurred!");
    } else {
        println!("AAAAAAAAAAAA");
    }
    Ok(())
}

/**
 * エラーを出力
 * @param msg エラー内容
 */
fn print_error(msg: String) {
    eprint!("{}: ", Red.bold().paint("Error"));
    eprintln!("{}", msg);
}

/**
 * WARNINGを出力
 * @param msg WARNING内容
 */
fn print_warning(msg: String) {
    eprint!("{}: ", Yellow.bold().paint("Warning"));
    eprintln!("{}", msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /**
     * ファイル読み込みテスト
     */
    fn print_logo_test() {
        let path =
            PathBuf::from(r"/Users/ryuse/Desktop/Algorithm Library/cpstt/cpstt/target/debug/cpstt");
        let result_ok = print_logo(path);
        assert!(result_ok.is_ok());
        let path = PathBuf::from(r"/path/to");
        let result_ok = print_logo(path);
        assert!(result_ok.is_err());
    }

    // #[test]
    // fn exec_cpp_program_test() {
    //     let path =
    //         PathBuf::from(r"/Users/ryuse/Desktop/Algorithm Library/cpstt/cpstt/test/smart.cpp");
    //     let result_ok = exec_cpp_program(path);
    //     assert!(result_ok.is_ok());
    // }
}

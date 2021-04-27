use anyhow::Result;
use clap::Clap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
// use std::process::Command;

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
fn generator(mut root_path: PathBuf) -> Result<()> {
    root_path.push("test/generator.cpp");
    println!("{}", root_path.display());
    // exec_cpp_program(root_path)?;
    Ok(())
}

/*
 * C++のファイルを指定し，そのプログラムを実行する
 */
// fn exec_cpp_program(_path: PathBuf) -> Result<()> {
//     let mut output = Command::new("ls")
//         .args(&["-l", "-a"])
//         .spawn()
//         .expect("failed to start `ls`");
//     let stdout = output.stdout.take().unwrap();

//     let reader = BufReader::new(stdout);
//     for line in reader.lines() {
//         println!("{}", line?);
//     }
//     Ok(())
// }

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

use anyhow::{bail, Result};
use clap::Clap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::PathBuf;
use std::process::{Command, Stdio};

mod fileio;
mod print_error;
pub use crate::fileio::MyFileIO;
pub use crate::print_error::PrintError;

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
    let root_path = MyFileIO::get_root_path();

    /* ロゴを出力 */
    print_logo(root_path.clone())?;

    /* プログラムの初期化 */
    init(root_path.clone())?;

    /* generatorを実行 */
    generator(root_path.clone())?;

    /* generatorで生成したファイルパスの取得 */
    let mut testcase_dir_path = root_path.clone();
    testcase_dir_path.push("test/testcase");
    let testcase_path_list = MyFileIO::get_path_list(testcase_dir_path.clone())?;

    /* smartなプログラムを実行 */
    smart(root_path.clone(), &testcase_path_list)?;

    Ok(())
}

/**
 * CPSTTのロゴを出力
 * @param path 本プログラムへの絶対パス
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
 * プログラムの初期化
 * @param root_path 本プログラムへの絶対パス
 * @return 正常終了の有無
 */
fn init(root_path: PathBuf) -> Result<()> {
    let mut test_path = root_path.clone();
    test_path.push("test");
    
    let mut testcase_path = test_path.clone();
    testcase_path.push("testcase");
    MyFileIO::file_clean(testcase_path)?;

    let mut output_smart_path = test_path.clone();
    output_smart_path.push("cpstt_out/smart");
    MyFileIO::file_clean(output_smart_path)?;

    let mut output_stupid_path = test_path.clone();
    output_stupid_path.push("cpstt_out/stupid");
    MyFileIO::file_clean(output_stupid_path)?;
    Ok(())
}

/**
 * generatorを実行
 * @param generator_path 本プログラムへの絶対パス
 * @return 正常終了の有無
 */
fn generator(mut generator_path: PathBuf) -> Result<()> {
    // パスの作成
    generator_path.push("test/generator.cpp");
    let mut generator_root_path = generator_path.clone();
    generator_root_path.pop();

    // generatorを実行
    let args = vec![String::from(generator_root_path.to_str().unwrap())];
    let exec_output = exec_generator(generator_path.clone(), &args, &generator_root_path)?;
    println!("=== generator output ===");
    println!("{}\n", exec_output);
    Ok(())
}

/**
 * smartを実行
 * @param smart_path 本プログラムへの絶対パス
 * @param testcase_paths テストケースのパス一覧
 * @return 正常終了の有無
 */
fn smart(mut smart_path: PathBuf, testcase_paths: &Vec<PathBuf>) -> Result<()> {
    smart_path.push("test/smart.cpp");
    let mut smart_root_path = smart_path.clone();
    smart_root_path.pop();

    for test_num in 0..testcase_paths.len() {
        let args = vec![
            String::from("<"),
            String::from(testcase_paths[test_num].to_str().unwrap()),
        ];
        let exec_output = exec_cpp_program(smart_path.clone(), &args, &smart_root_path)?;
        println!(
            "=== smart output ({}/{}) ===",
            test_num + 1,
            testcase_paths.len()
        );
        if exec_output.len() < 50 {
            println!("{}\n", exec_output);
        } else {
            println!(
                "Output data is too large. (content-size: {})\n",
                exec_output.len()
            );
            println!("{}\n......", &exec_output[0..50]);
        }
        let mut output_path = smart_root_path.clone();
        output_path.push("cpstt_out/smart");
        output_path.push(
            testcase_paths[test_num]
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
        );
        output_path.set_extension("out");
        MyFileIO::file_write(&output_path, &exec_output)?;
    }
    Ok(())
}

/**
 * C++のファイルを指定し，そのプログラムをコンパイルする
 * @param cpp_path C++ファイルへのパス
 * @return 異常終了: エラー
 *         正常終了: 実行結果の文字列
 */
fn compile(cpp_path: &PathBuf) -> Result<(), anyhow::Error> {
    let mut dir_root_path = cpp_path.clone();
    dir_root_path.pop();
    let compile_output = Command::new("g++")
        .args(&[
            "-std=c++1z",
            "-O3",
            "-fsanitize=undefined",
            "-I",
            ".",
            cpp_path.to_str().unwrap(),
        ])
        .current_dir(dir_root_path.to_str().unwrap())
        .output()
        .expect("Failed to compile C++ program");

    let compile_stderr = String::from_utf8_lossy(&compile_output.stderr);
    if compile_stderr != String::from("") {
        eprintln!("{}", compile_stderr);
        PrintError::print_error(String::from("It seems compile error"));
        bail!("Some Error is occurred!");
    }
    Ok(())
}

/**
 * C++のファイルを指定し，そのプログラムを実行する
 * @param cpp_path C++ファイルへのパス
 * @param exec_args C++実行形式ファイルのコマンドライン引数
 * @param root_path C++ファイルがあるディレクトリへのパス
 * @return 異常終了: エラー
 *         正常終了: 実行結果の文字列
 */
fn exec_generator(
    cpp_path: PathBuf,
    exec_args: &Vec<String>,
    root_path: &PathBuf,
) -> Result<String> {
    compile(&cpp_path)?;
    let exec_output = Command::new(format!("{}/a.out", root_path.to_str().unwrap()))
        .args(exec_args)
        .output()
        .expect("Failed to execution C++ program");

    let exec_stdout = String::from_utf8_lossy(&exec_output.stdout);
    let exec_stderr = String::from_utf8_lossy(&exec_output.stderr);
    if exec_stderr != String::from("") {
        eprintln!("{}", exec_stderr);
        PrintError::print_error(String::from("It seems execution error"));
        bail!("Some Error is occurred!");
    }
    Ok(exec_stdout.into_owned())
}

/**
 * C++のファイルを指定し，そのプログラムを実行する
 * @param cpp_path C++ファイルへのパス
 * @param exec_args C++実行形式ファイルのコマンドライン引数
 * @param root_path C++ファイルがあるディレクトリへのパス
 * @return 異常終了: エラー
 *         正常終了: 実行結果の文字列
 */
fn exec_cpp_program(
    cpp_path: PathBuf,
    exec_args: &Vec<String>,
    root_path: &PathBuf,
) -> Result<String> {
    compile(&cpp_path)?;
    let exec_cat = Command::new("cat")
        .args(&[exec_args[1].clone()])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to load testcase");
    let exec_cpp = Command::new(format!("{}/a.out", root_path.to_str().unwrap()))
        .args(exec_args)
        .stdin(unsafe { Stdio::from_raw_fd(exec_cat.stdout.as_ref().unwrap().as_raw_fd()) })
        .output()
        .expect("Failed to execution C++ program");
    let exec_stdout = String::from_utf8_lossy(&exec_cpp.stdout);
    let exec_stderr = String::from_utf8_lossy(&exec_cpp.stderr);
    if exec_stderr != String::from("") {
        eprintln!("{}", exec_stderr);
        PrintError::print_error(format!(
            "It seems execution error [{}]",
            cpp_path.to_str().unwrap()
        ));
        bail!("Some Error is occurred!");
    }
    Ok(exec_stdout.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /**
     * ファイル読み込みテスト
     */
    fn print_logo_test() {
        /* 正常パス */
        let mut root_path = MyFileIO::get_root_path();
        root_path.pop(); // testでは実行ファイルパスに'/target'が付く
        let result_ok = print_logo(root_path.clone());
        assert!(result_ok.is_ok());

        /* 異常パス */
        let invalid_path = PathBuf::from(r"/path/to");
        let result_ok = print_logo(invalid_path);
        assert!(result_ok.is_err());
    }

    #[test]
    /**
     * generatorファイルの実行テスト
     */
    fn exec_generator_test() {
        /* 正常ファイル */
        let mut generator_path = MyFileIO::get_root_path();
        generator_path.pop();
        generator_path.push("test/generator.cpp");
        let mut generator_root_path = generator_path.clone();
        generator_root_path.pop();
        let args = vec![String::from(generator_root_path.to_str().unwrap())];
        let exec_output =
            exec_generator(generator_path.clone(), &args, &generator_root_path).unwrap();
        assert_eq!(exec_output, String::from(""));

        /* コンパイルエラーファイル */
        let mut generator_path_com_err = MyFileIO::get_root_path();
        generator_path_com_err.pop();
        generator_path_com_err.push("test/generator_compile_err.cpp");
        let mut generator_root_path_com_err = generator_path_com_err.clone();
        generator_root_path_com_err.pop();
        let args_com_err = vec![String::from(generator_root_path_com_err.to_str().unwrap())];
        let exec_output_com_err = exec_generator(
            generator_path_com_err.clone(),
            &args_com_err,
            &generator_root_path_com_err,
        );
        assert!(exec_output_com_err.is_err());

        /* ランタイムエラーファイル */
        let mut generator_path_exec_err = MyFileIO::get_root_path();
        generator_path_exec_err.pop();
        generator_path_exec_err.push("test/generator_compile_err.cpp");
        let mut generator_root_path_exec_err = generator_path_exec_err.clone();
        generator_root_path_exec_err.pop();
        let args_exec_err = vec![String::from(generator_root_path_exec_err.to_str().unwrap())];
        let exec_output_exec_err = exec_generator(
            generator_path_exec_err.clone(),
            &args_exec_err,
            &generator_root_path_exec_err,
        );
        assert!(exec_output_exec_err.is_err());
    }
}

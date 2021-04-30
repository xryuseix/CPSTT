use anyhow::{bail, Result};
use clap::Clap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::PathBuf;
use std::process::{Command, Stdio};

mod fileio;
mod print_console;
pub use crate::fileio::{MyFileIO, SETTING};
pub use crate::print_console::{PrintColorize, PrintError};

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
    exec_user_program(
        root_path.clone(),
        &testcase_path_list,
        String::from("smart"),
    )?;

    /* stupidなプログラムを実行 */
    exec_user_program(
        root_path.clone(),
        &testcase_path_list,
        String::from("stupid"),
    )?;

    /* smartとstupidを比較 */
    compare_result(root_path.clone())?;

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
    /* 不要なファイルを削除 */
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
    /* パスの作成 */
    generator_path.push("test/generator.cpp");
    let mut generator_root_path = generator_path.clone();
    generator_root_path.pop();

    /* generatorを実行 */
    let args = vec![String::from(generator_root_path.to_str().unwrap())];
    let exec_output = exec_generator(generator_path.clone(), &args, &generator_root_path)?;
    if SETTING.logging.dump_exe_result {
        println!("=== generator output ===");
        println!("{}\n", exec_output);
    } else {
        println!("generator is done.",);
    }
    Ok(())
}

/**
 * smart/stupidを実行
 * @param program_path 本プログラムへの絶対パス
 * @param testcase_paths テストケースのパス一覧
 * @return 正常終了の有無
 */
fn exec_user_program(
    mut program_path: PathBuf,
    testcase_paths: &Vec<PathBuf>,
    program_type: String,
) -> Result<()> {
    program_path.push(format!("test/{}.cpp", program_type));
    let mut program_root_path = program_path.clone();
    program_root_path.pop();

    for (i, testcase) in testcase_paths.iter().enumerate() {
        let args = vec![String::from("<"), String::from(testcase.to_str().unwrap())];
        let exec_output = exec_cpp_program(program_path.clone(), &args, &program_root_path)?;
        if SETTING.logging.dump_exe_result {
            println!(
                "=== {} output ({}/{}) ===",
                program_type,
                i + 1,
                testcase_paths.len()
            );
            let max_len = SETTING.execution.max_output_len as usize;
            if exec_output.len() < max_len {
                /* 実行結果の文字列が短い場合 */
                println!("{}\n", exec_output);
            } else {
                /* 実行結果の文字列が長い場合 */
                let exec_output_format = exec_output.replace("\n", "\x1b[33m\\n\x1b[m").replacen(
                    "\x1b[33m\\n\x1b[m",
                    "\n",
                    (SETTING.execution.max_output_line - 1) as usize,
                );
                println!(
                    "Output data is too large. (content-size: {})\n",
                    exec_output.len()
                );
                let end = exec_output_format.char_indices().nth(max_len).unwrap().0;
                let sliced_output = &exec_output_format[0..end];
                println!("{}\x1b[m\n......\n", &sliced_output);
            }
        } else {
            println!(
                "{} output ({}/{}) is done.",
                program_type,
                i + 1,
                testcase_paths.len()
            );
        }
        /* 実行結果をファイル書き込み */
        let mut output_path = program_root_path.clone();
        output_path.push(format!("cpstt_out/{}", program_type));
        output_path.push(testcase.file_name().unwrap().to_str().unwrap());
        output_path.set_extension("out");
        MyFileIO::write_file(&output_path, &exec_output)?;
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

/**
 * smartとstupidの結果を比較する
 * @param root_path 本プログラムへの絶対パス
 * @return 異常終了: エラー
 *         正常終了: 実行結果の文字列
 */
fn compare_result(root_path: PathBuf) -> Result<()> {
    /* フォルダパスの生成 */
    let mut smart_test_path = root_path.clone();
    smart_test_path.push("test/cpstt_out/smart");
    let mut stupid_test_path = root_path.clone();
    stupid_test_path.push("test/cpstt_out/stupid");
    /* テストケースのパスの取得 */
    let smart_test = MyFileIO::get_path_list(smart_test_path).unwrap();
    let stupid_test = MyFileIO::get_path_list(stupid_test_path).unwrap();

    let mut accepted = 0;
    let mut wrong_answer = 0;

    /* ファイルを読み込みながら比較 */
    for (smart, stupid) in (smart_test.iter()).zip(stupid_test.iter()) {
        /* ファイルを読み込み */
        let smart_content = MyFileIO::read_file(String::from(smart.to_str().unwrap()))?;
        let stupid_content = MyFileIO::read_file(String::from(stupid.to_str().unwrap()))?;
        /* 比較 */
        if smart_content == stupid_content {
            accepted += 1;
            println!(
                "[ test ] {}: {}",
                PrintColorize::print_green(String::from("AC")),
                smart.file_name().unwrap().to_str().unwrap()
            );
        } else {
            wrong_answer += 1;
            println!(
                "[ test ] {}: {}",
                PrintColorize::print_yellow(String::from("WA")),
                smart.file_name().unwrap().to_str().unwrap()
            );
        }
    }
    /* 結果を出力 */
    println!(
        "[result]  {}: {}, {}: {}, testcase: {}",
        PrintColorize::print_green(String::from("AC")),
        accepted,
        PrintColorize::print_yellow(String::from("WA")),
        wrong_answer,
        accepted + wrong_answer
    );
    Ok(())
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

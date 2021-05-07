use anyhow::{bail, Result};
use clap::Clap;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

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
    /* テストディレクトリへのパスを取得 */
    let root_path = MyFileIO::get_root_path();

    /* ロゴを出力 */
    print_logo()?;

    /* プログラムの初期化 */
    init(root_path.clone())?;

    /* generatorを実行 */
    generator(root_path.clone())?;

    /* generatorで生成したファイルパスの取得 */
    let mut testcase_dir_path = root_path.clone();
    testcase_dir_path.push("testcase");
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
 * @param path テストディレクトリへの絶対パス
 * @return 正常終了の有無
 */
fn print_logo() -> Result<()> {
    println!(r"-----------------------------------------");
    println!(r"       __________  _________________     ");
    println!(r"      / ____/ __ \/ ___/_  __/_  __/     ");
    println!(r"     / /   / /_/ /\__ \ / /   / /        ");
    println!(r"    / /___/ ____/___/ // /   / /         ");
    println!(r"    \____/_/    /____//_/   /_/          ");
    println!(r"                                         ");
    println!(r"Competitive Programming Stress Test Tools");
    println!(r"-----------------------------------------");
    Ok(())
}

/**
 * プログラムの初期化
 * @param test_path テストディレクトリへの絶対パス
 * @return 正常終了の有無
 */
fn init(test_path: PathBuf) -> Result<()> {
    /* 不要なファイルを削除 */
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
 * @param generator_path テストディレクトリへの絶対パス
 * @return 正常終了の有無
 */
fn generator(mut generator_path: PathBuf) -> Result<()> {
    /* パスの作成 */
    generator_path.push("generator.cpp");
    let mut generator_root_path = generator_path.clone();
    generator_root_path.pop();

    /* generatorを実行 */
    let args = vec![String::from(generator_root_path.to_str().unwrap())];
    let exec_output = exec_generator(generator_path.clone(), &args, &generator_root_path)?;
    if SETTING.logging.dump_exe_result {
        println!(
            "{} is done.",
            PrintColorize::print_cyan(String::from("[ generator ]"))
        );
        println!("{}", exec_output);
    } else {
        println!(
            "{} is done.",
            PrintColorize::print_cyan(String::from("[ generator ]"))
        );
    }
    Ok(())
}

/**
 * smart/stupidを実行
 * @param program_path テストディレクトリへの絶対パス
 * @param testcase_paths テストケースのパス一覧
 * @param program_type smart or stupid
 * @return 正常終了の有無
 */
fn exec_user_program(
    mut program_path: PathBuf,
    testcase_paths: &Vec<PathBuf>,
    program_type: String,
) -> Result<()> {
    program_path.push(format!("{}.cpp", program_type));
    let mut program_root_path = program_path.clone();
    program_root_path.pop();

    /* C++プログラムを全て並列実行 */
    let mut handles = Vec::new();

    for (i, testcase) in testcase_paths.iter().enumerate() {
        
        /* サブスレッドへデータを渡す */
        /* TODO: ここどうにかする */
        let (sender_program_path, receiver_program_path) = mpsc::channel();
        sender_program_path.send(program_path.clone()).unwrap();
        let (sender_root_path, receiver_root_path) = mpsc::channel();
        sender_root_path.send(program_root_path.clone()).unwrap();
        let (sender_testcase_len, receiver_testcase_len) = mpsc::channel();
        sender_testcase_len.send(testcase_paths.len()).unwrap();
        let (sender_testcase, receiver_testcase) = mpsc::channel();
        sender_testcase.send(testcase.clone()).unwrap();
        let (sender_program_type, receiver_program_type) = mpsc::channel();
        sender_program_type.send(program_type.clone()).unwrap();

        handles.push(thread::spawn(move || {
            /* メインスレッドから受け取り */
            let program_path = receiver_program_path.recv().unwrap();
            let program_root_path = receiver_root_path.recv().unwrap();
            let testcase = receiver_testcase.recv().unwrap();
            let testcase_len = receiver_testcase_len.recv().unwrap();
            let program_type = receiver_program_type.recv().unwrap();

            let args = vec![String::from("<"), String::from(testcase.to_str().unwrap())];
            let (exec_output, exec_time, is_tle) =
                exec_cpp_program(program_path.clone(), &args, &program_root_path).unwrap();
            if SETTING.logging.dump_exe_result {
                println!(
                    "{} ({}/{}) is {}. ({}.{:03} sec)",
                    PrintColorize::print_cyan(format!("[ {} ]", program_type)),
                    i + 1,
                    testcase_len,
                    is_tle,
                    exec_time.as_secs(),
                    exec_time.subsec_nanos() / 1_000_000
                );
                let max_len = SETTING.execution.max_output_len as usize;
                if exec_output.len() < max_len {
                    /* 実行結果の文字列が短い場合 */
                    println!("{}", exec_output);
                } else {
                    /* 実行結果の文字列が長い場合 */
                    let exec_output_format = exec_output.replace("\n", "\x1b[33m\\n\x1b[m").replacen(
                        "\x1b[33m\\n\x1b[m",
                        "\n",
                        (SETTING.execution.max_output_line - 1) as usize,
                    );
                    println!(
                        "Output data is too large. (content-size: {})",
                        exec_output.len()
                    );
                    let end = exec_output_format.char_indices().nth(max_len).unwrap().0;
                    let sliced_output = &exec_output_format[0..end];
                    println!("{}\x1b[m\n......\n", &sliced_output);
                }
            } else {
                println!(
                    "{} ({}/{}) is {}. ({}.{:03} sec)",
                    PrintColorize::print_cyan(format!("[ {} ]", program_type)),
                    i + 1,
                    testcase_len,
                    is_tle,
                    exec_time.as_secs(),
                    exec_time.subsec_nanos() / 1_000_000
                );
            }
            /* 実行結果をファイル書き込み */
            let mut output_path = program_root_path.clone();
            output_path.push(format!("cpstt_out/{}", program_type));
            output_path.push(testcase.file_name().unwrap().to_str().unwrap());
            output_path.set_extension("out");
            MyFileIO::write_file(&output_path, &exec_output).unwrap();
        }));
    }
    for handle in handles {
        let _ = handle.join();
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
) -> Result<(String, Duration, String)> {
    compile(&cpp_path)?;
    let exec_cat = Command::new("cat")
        .args(&[exec_args[1].clone()])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to load testcase");

    /* メインスレッドとサブスレッドのデータ送受信チャンネル */
    let (sender_root_path, receiver_root_path) = mpsc::channel();
    sender_root_path.send(root_path.clone()).unwrap();
    let (sender_args, receiver_args) = mpsc::channel();
    sender_args.send(exec_args.clone()).unwrap();

    /* サブスレッドからのデータ書き込み先 */
    let output = Arc::new(Mutex::new(vec![String::from(""), String::from("")]));
    let output_ref = output.clone();
    let exec_time = Arc::new(Mutex::new(Duration::new(0, 0)));
    let exec_ref = exec_time.clone();

    /* 時間計測開始 */
    let start = Instant::now();

    /* C++プログラムの実行 */
    thread::spawn(move || {
        let root_path = receiver_root_path.recv().unwrap();
        let exec_args = receiver_args.recv().unwrap();
        let exec_cpp = Command::new(format!("{}/a.out", root_path.to_str().unwrap()))
            .args(exec_args)
            .stdin(unsafe { Stdio::from_raw_fd(exec_cat.stdout.as_ref().unwrap().as_raw_fd()) })
            .output()
            .expect("Failed to execution C++ program");
        /* 実行結果の書き込み */
        let mut output = output_ref.lock().unwrap();
        output[0] = String::from_utf8_lossy(&exec_cpp.stdout).into_owned();
        output[1] = String::from_utf8_lossy(&exec_cpp.stderr).into_owned();
        /* 実行時間の書き込み */
        let mut exec_time = exec_ref.lock().unwrap();
        *exec_time = start.elapsed();
    });

    /* スリープの実行 */
    let handle2 = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200)); // TODO: ここ書き換える
    });
    let _ = handle2.join();

    let end = *exec_time.lock().unwrap();
    let std_out_err = (*output.lock().unwrap()).clone();

    /* 実行時エラーの処理 */
    if std_out_err[1] != String::from("") {
        eprintln!("{}", std_out_err[1]);
        PrintError::print_error(format!(
            "It seems execution error [{}]",
            cpp_path.to_str().unwrap()
        ));
        bail!("Some Error is occurred!");
    }

    /* TLE判定 */
    let is_tle = if end != Duration::new(0, 0) {
        String::from("done")
    } else {
        PrintColorize::print_yellow(String::from("TLE"))
    };
    Ok((std_out_err[0].clone(), end, is_tle))
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
    smart_test_path.push("cpstt_out/smart");
    let mut stupid_test_path = root_path.clone();
    stupid_test_path.push("cpstt_out/stupid");
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
                "{} {}: {}",
                PrintColorize::print_cyan(String::from("[ test ]")),
                PrintColorize::print_green(String::from("AC")),
                smart.file_name().unwrap().to_str().unwrap()
            );
        } else {
            wrong_answer += 1;
            println!(
                "{} {}: {}",
                PrintColorize::print_cyan(String::from("[ test ]")),
                PrintColorize::print_yellow(String::from("WA")),
                smart.file_name().unwrap().to_str().unwrap()
            );
        }
    }
    /* 結果を出力 */
    println!(
        "{} {}: {}, {}: {} (testcase: {})",
        PrintColorize::print_cyan(String::from("[ result ]")),
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
        let result_ok = print_logo();
        assert!(result_ok.is_ok());
    }

    #[test]
    /**
     * generatorファイルの実行テスト
     */
    fn exec_generator_test() {
        /* 正常ファイル */
        let mut generator_path = MyFileIO::get_root_path();
        generator_path.push("generator.cpp");
        let mut generator_root_path = generator_path.clone();
        generator_root_path.pop();
        let args = vec![String::from(generator_root_path.to_str().unwrap())];
        let exec_output =
            exec_generator(generator_path.clone(), &args, &generator_root_path).unwrap();
        assert_eq!(exec_output, String::from(""));
    }
}

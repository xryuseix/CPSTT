use anyhow::{bail, Result};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub use crate::print_error::PrintError;

pub struct MyFileIO {}

#[derive(Debug, Deserialize)]
pub struct SettingsClass {
    pub execution: ExecutionSettings,
}

#[derive(Debug, Deserialize)]
pub struct ExecutionSettings {
    pub max_output_len: i32,
    pub max_output_line: i32,
}

lazy_static! {
    /**
     * 設定ファイルの読み込み(実際にデータを持つのはこれ
     * SETTING.execution.max_output_len のようにアクセスする
     */
    pub static ref SETTING: SettingsClass = {
        let mut settings_path = MyFileIO::get_root_path().clone();
        settings_path.push("settings.toml");
        let settings = MyFileIO::read_settings(settings_path.clone()).unwrap();
        settings
    };
}

impl MyFileIO {
    /**
     * プロジェクトのルートパスを取得
     * @return プロジェクトのルートパス
     */
    pub fn get_root_path() -> PathBuf {
        let mut exec_path = env::current_exe().unwrap();
        for _i in 0..3 {
            exec_path.pop();
        }
        exec_path
    }

    /**
     * 特定ディレクトリ内のファイルパス一覧を取得
     * @param dir_path 一覧を取得したいディレクトリへの絶対パス
     * @return 異常終了: エラー
     *         正常終了: パスが入った配列
     */
    pub fn get_path_list(dir_path: PathBuf) -> Result<Vec<PathBuf>> {
        let mut file_paths = Vec::new();
        let paths = fs::read_dir(dir_path)?;
        for path in paths.into_iter() {
            file_paths.push(path?.path());
        }
        file_paths.sort();
        Ok(file_paths.clone())
    }

    /**
     * 特定ディレクトリ内のファイルを全て削除
     * @param dir_path 削除したいディレクトリへの絶対パス
     * @return 異常終了: エラー
     *         正常終了: 実行結果の文字列
     */
    pub fn file_clean(dir_path: PathBuf) -> Result<(), anyhow::Error> {
        let paths = MyFileIO::get_path_list(dir_path)?;
        for path in paths.iter() {
            let extension = path.extension().unwrap().to_str().unwrap();
            if extension == "in" || extension == "out" {
                fs::remove_file(path)?;
            } else {
                PrintError::print_error(format!(
                    "{} could not be deleted because its extension is {}",
                    path.to_str().unwrap(),
                    extension
                ));
                bail!("Some Error is occurred!");
            }
        }
        Ok(())
    }

    /**
     * ファイルの書き込み
     * @param path 書き込み先
     * @param content 書き込みたい文字列
     * @return 異常終了: エラー
     *         正常終了: 実行結果の文字列
     */
    pub fn write_file(path: &PathBuf, content: &String) -> Result<()> {
        let mut file = File::create(path)?;
        write!(file, "{}", content)?;
        file.flush()?;
        Ok(())
    }

    /**
     * ファイルの読み込み
     * @param path ファイルへの絶対パス
     * @return 異常終了: エラー
     *         正常終了: 実行結果の文字列
     */
    pub fn read_file(path: String) -> Result<String, anyhow::Error> {
        let file_content = fs::read_to_string(path)?;
        Ok(file_content)
    }

    /**
     * 設定ファイルの読み込み
     * @param path settings.tomlへの絶対パス
     * @return 異常終了: エラー
     *         正常終了: 実行結果の文字列
     */
    pub fn read_settings(path: PathBuf) -> Result<SettingsClass, anyhow::Error> {
        let settings_content = MyFileIO::read_file(String::from(path.to_str().unwrap()))?;
        let settings: SettingsClass = toml::from_str(&settings_content).unwrap();
        Ok(settings)
    }
}

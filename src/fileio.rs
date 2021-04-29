use std::env;
use std::path::PathBuf;
use std::fs;
use anyhow::{Result};

pub struct MyFileIO {}

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
}

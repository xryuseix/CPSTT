use ansi_term::Colour::{Green, Red, Yellow};

pub struct PrintError {}

impl PrintError {
    /**
     * エラーを出力
     * @param msg エラー内容
     */
    pub fn print_error(msg: String) -> Self {
        eprint!("{}: ", Red.bold().paint("Error"));
        eprintln!("{}", msg);
        Self {}
    }

    /**
     * WARNINGを出力
     * @param msg WARNING内容
     */
    #[allow(dead_code)]
    pub fn print_warning(msg: String) -> Self {
        eprint!("{}: ", Yellow.bold().paint("Warning"));
        eprintln!("{}", msg);
        Self {}
    }
}

pub struct PrintColorize {}

impl PrintColorize {
    /**
     * 色を緑色に変換
     * @param msg 表示内容
     */
    pub fn print_green(msg: String) -> String {
        return Green.bold().paint(msg).to_string();
    }

    /**
     * 色を黄色に変換
     * @param msg 表示内容
     */
    pub fn print_yellow(msg: String) -> String {
        return Yellow.bold().paint(msg).to_string();
    }
}

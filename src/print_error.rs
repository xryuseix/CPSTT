use ansi_term::Colour::{Red, Yellow};

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

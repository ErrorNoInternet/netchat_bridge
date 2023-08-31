use chrono::Local;
use colored::*;

pub enum LogMessageType {
    Bot,
    Matrix,
    Warning,
    Error,
    MatrixWarning,
    MatrixError,
}

pub fn log_matrix_error<T, E: std::fmt::Display>(result: Result<T, E>) {
    match result {
        Ok(_) => (),
        Err(error) => log_message(LogMessageType::MatrixError, &error.to_string()),
    }
}

pub fn log_message(message_type: LogMessageType, message: &String) {
    match message_type {
        LogMessageType::Bot => {
            println!(
                "{} {} {}",
                current_time(),
                colored_brackets(&"BOT".bold().magenta()),
                message
            )
        }
        LogMessageType::Matrix => {
            println!(
                "{} {} {}",
                current_time(),
                colored_brackets(&"MATRIX".bold().green()),
                message
            )
        }
        LogMessageType::Warning => println!(
            "{} {} {}",
            current_time(),
            colored_brackets(&"WARNING".bold().red()),
            message.yellow()
        ),
        LogMessageType::Error => println!(
            "{} {} {}",
            current_time(),
            colored_brackets(&"ERROR".bold().red()),
            message.red()
        ),
        LogMessageType::MatrixWarning => println!(
            "{} {} {}",
            current_time(),
            colored_brackets(&"WARNING (Matrix)".bold().red()),
            message.yellow()
        ),
        LogMessageType::MatrixError => println!(
            "{} {} {}",
            current_time(),
            colored_brackets(&"ERROR (Matrix)".bold().red()),
            message.red()
        ),
    }
}

fn current_time() -> String {
    format!(
        "{}{}{}",
        "[".bold().white(),
        Local::now()
            .format("%Y/%m/%d %H:%M:%S")
            .to_string()
            .bold()
            .white(),
        "]".bold().white()
    )
}

fn colored_brackets(text: &ColoredString) -> String {
    format!("{}{}{}", "[".bold().yellow(), text, "]".bold().yellow())
}

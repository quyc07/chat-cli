mod datetime;
mod friend;
mod main_select;
mod user;
mod token;
mod recent_chat;
mod console;
mod add_friend;
mod style;
use clap::{Parser, Subcommand};
use futures::StreamExt;
use reqwest;
use serde::Deserialize;
use std::fmt::Display;
use tokio::io::AsyncBufReadExt;

// 分隔符
pub(crate) const DELIMITER: &str = "-----------------------------------------------";

pub(crate) fn delimiter() {
    println!("{DELIMITER}");
}

#[cfg(feature = "release")]
static HOST: &str = include_str!("../config/release");
#[cfg(not(feature = "release"))]
static HOST: &str = "http://localhost:3000";

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let _ = match cli.command {
        Commands::Register { name, password } => user::register(name, password).await,
        Commands::Login { name, password } => user::login(name, password).await,
    };

}

#[derive(Parser)]
#[command(version="0.1",about="A Chat Client", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 注册用户
    Register {
        /// 用户名
        #[arg(short='n', long, value_parser = check_name)]
        name: String,
        /// 密码
        #[arg(short='p', long, value_parser = check_password)]
        password: String,
    },
    /// 登陆
    Login {
        /// 用户名
        #[arg(short, long)]
        name: String,
        /// 密码
        #[arg(short, long)]
        password: String,
    },
}
/// 校验用户名
/// 用户名必须是纯英文
fn check_name(name: &str) -> Result<String, String> {
    // name 必须是纯英文或英文与数字的组合
    if !name.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("用户名必须为英文或英文与数字的组合".to_string());
    }
    Ok(name.to_string())
}

/// 检查密码是否有效
/// 有效的密码必须包含至少一个数字、一个大写字母和一个小写字母。
fn check_password(password: &str) -> Result<String, String> {
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());

    if has_digit && has_uppercase && has_lowercase {
        Ok(password.to_string())
    } else {
        Err("有效的密码必须包含至少一个数字、一个大写字母和一个小写字母。".to_string())
    }
}

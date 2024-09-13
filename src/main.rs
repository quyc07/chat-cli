mod chat;
mod datetime;
mod friend;
mod main_select;
mod user;

use clap::{Parser, Subcommand};
use futures::StreamExt;
use reqwest;
use reqwest::Client;
use serde::Deserialize;
use tokio::io;
use tokio::io::AsyncBufReadExt;

// 分隔符
pub(crate) const DELIMITER: &str = "-----------------------------------------------";

pub(crate) fn delimiter() {
    println!("{DELIMITER}");
}

pub const HOST: &str = "http://127.0.0.1:3000";
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Register { name, password } => user::register(name, password).await,
        Commands::Login { name, password } => user::login(name, password).await,
    }
}

async fn maina() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端以连接到SSE流
    let client = Client::new();
    // 异步请求SSE流
    let mut sse_stream = client.get(format!("{}/event/stream", HOST))
        .header("Authorization", "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpZCI6MTEsIm5hbWUiOiJhbmR5IiwiZW1haWwiOiJxYWFAMTYzLmNvbSIsInBob25lIjoiMTg5MTEyMjMzNDQiLCJkZ3JhcGhfdWlkIjoiMHg0ZTQyIiwicm9sZSI6IlVzZXIiLCJleHAiOjE3MjYxNTkzNzN9.NX4wcm0QsqF46PA3dJUMSHwF92DpAyMpyo8U0JyI0-8")
        .header("User-Agent", "Chat-Cli/1.0")
        .send()
        .await?
        .bytes_stream();

    // 异步监听用户输入
    let stdin = io::BufReader::new(io::stdin());
    let mut stdin_lines = stdin.lines();

    println!("SSE 监听中... 输入 'exit' 退出");

    loop {
        tokio::select! {
            // 处理从SSE流中接收到的消息
            Some(msg) = sse_stream.next() => {
                match msg {
                    Ok(bytes) => {
                        let sse_message = String::from_utf8(bytes.to_vec()).unwrap();
                        println!("收到SSE消息: {}", sse_message);
                    },
                    Err(e) => {
                        eprintln!("SSE错误: {}", e);
                        break;
                    }
                }
            }

            // 处理用户输入
            Ok(Some(input)) = stdin_lines.next_line() => {
                if input.trim() == "exit" {
                    println!("退出...");
                    break;
                } else {
                    println!("你输入了: {}", input);
                }
            }
        }
    }

    Ok(())
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
        #[arg(short, long, value_parser = check_name)]
        name: String,
        /// 密码
        #[arg(short, long, value_parser = check_password)]
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

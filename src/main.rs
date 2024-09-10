use clap::{Arg, Command, Parser, Subcommand};
use reqwest;
use reqwest::StatusCode;
use serde::Deserialize;

const HOST: &str = "http://localhost:3000";
fn main() {
    // cli()
    let cli = Cli::parse();
    match cli.command {
        Commands::Register { name, password } => register(name, password),
        Commands::Login { name, password } => {
            println!("Logging in user {} with password {}", name, password);
        }
    }
}

fn register(name: String, password: String) {
    // 使用reqwest 向 POST HOST/user/register 接口注册用户
    let url = format!("{HOST}/user/register");
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(&url)
        .json(&serde_json::json!({
            "name": name,
            "password": password
        }))
        .send();
    match res {
        Ok(res) => {
            match res.status() {
                StatusCode::OK => {
                    println!("恭喜{name}注册成功，请登陆场聊吧！")
                }
                StatusCode::CONFLICT => {
                    println!("用户名已存在，请重新注册")
                }
                _ => {
                    println!("注册失败: {}", res.json::<RegisterRes>().unwrap().msg)
                }
            }
        }
        Err(err) => {
            println!("注册失败: {}", err)
        }
    }
}

#[derive(Deserialize)]
struct RegisterRes {
    code: i8,
    msg: String,
    data: i8,
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

fn cli() {
    let matches = Command::new("chat-cli")
        .version("0.1.0")
        .author("Your Name <your_email@example.com>")
        .about("Handles command line arguments")
        .arg(
            Arg::new("host")
                .long("host")
                .value_parser(clap::value_parser!(String))
                .help("Sets the host address")
                .required(true),
        )
        .arg(
            Arg::new("name")
                .long("name")
                .value_parser(clap::value_parser!(String))
                .help("Sets the user's name")
                .required(true),
        )
        .arg(
            Arg::new("password")
                .long("password")
                .value_parser(clap::value_parser!(String))
                .help("Sets the user's password")
                .required(true),
        )
        .get_matches();

    let host = matches.get_one::<String>("host").unwrap();
    let name = matches.get_one::<String>("name").unwrap();
    let password = matches.get_one::<String>("password").unwrap();

    let login_url = format!("http://{}:3000/token/login", host);
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&login_url)
        .json(&serde_json::json!({
            "name": name,
            "password": password
        }))
        .send();

    let token = match response {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<Login>() {
                    Ok(Login { msg, data, code: _ }) => {
                        println!("Login successful. Message: {}", msg);
                        println!("Access token: {}", data.access_token);
                        Some(data.access_token)
                    }

                    Err(e) => {
                        println!("Failed to parse response: {}", e);
                        None
                    }
                }
            } else {
                println!("Login failed: HTTP {}", res.status());
                None
            }
        }
        Err(e) => {
            println!("Failed to send login request: {}", e);
            None
        }
    };

    if token.is_none() {
        println!("Login failed. Exiting the program.");
        std::process::exit(1);
    }

    let options = vec!["Friends", "Groups"];
    let selection = dialoguer::Select::new()
        .with_prompt("Please select an option")
        .items(&options)
        .interact()
        .unwrap();

    println!("You selected: {}", options[selection]);
    if options[selection] == "Friends" {
        let client = reqwest::blocking::Client::new();
        let friends_url = format!("http://{}:3000/friend", host);
        let response = client
            .get(&friends_url)
            .header("Authorization", format!("Bearer {}", token.unwrap()))
            .send();

        let friends = match response {
            Ok(res) => {
                if res.status().is_success() {
                    match res.json::<FriendsRes>() {
                        Ok(FriendsRes {
                               msg: _,
                               data,
                               code: _,
                           }) => Some(data),
                        Err(e) => {
                            println!("Failed to read response: {}", e);
                            None
                        }
                    }
                } else {
                    println!("Failed to get friends list: HTTP {}", res.status());
                    None
                }
            }
            Err(e) => {
                println!("Failed to send request: {}", e);
                None
            }
        };
        if friends.is_none() {
            println!("Get friends failed. Exiting the program.");
            std::process::exit(1);
        }
        let friends = friends.unwrap();
        let friend_names: Vec<&str> = friends.iter().map(|f| f.name.as_str()).collect();
        let selection = dialoguer::Select::new()
            .with_prompt("Select a friend")
            .items(&friend_names)
            .interact()
            .unwrap();

        let selected_friend = &friends[selection];
        println!(
            "Selected friend - ID: {}, Name: {}",
            selected_friend.id, selected_friend.name
        );
    }
}

#[derive(Deserialize)]
struct Login {
    code: i8,
    msg: String,
    data: LoginRes,
}
#[derive(Deserialize)]
struct FriendsRes {
    code: i8,
    msg: String,
    data: Vec<Friend>,
}

#[derive(Deserialize)]
struct LoginRes {
    access_token: String,
}

#[derive(Deserialize)]
struct Friend {
    id: i32,
    name: String,
}

use crate::{main_select, HOST};
use reqwest::StatusCode;
use serde::Deserialize;
use std::cell::RefCell;
use crate::main_select::MainSelect;

pub(crate) fn register(name: String, password: String) {
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

thread_local! {
        pub static TOKEN: RefCell<String> = RefCell::new(String::default());
    }

pub(crate) fn login(name: String, password: String) {
    let login_url = format!("{HOST}/token/login");
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
                    Ok(Login { msg: _, data, code: _ }) => {
                        println!("登陆成功");
                        Some(data.access_token)
                    }
                    Err(e) => {
                        println!("Failed to parse response: {}", e);
                        None
                    }
                }
            } else if res.status() == StatusCode::UNAUTHORIZED {
                println!("Login failed: 用户名或密码错误");
                None
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
    TOKEN.replace(token.unwrap());
    MainSelect::select();
}

#[derive(Deserialize)]
struct Login {
    code: i8,
    msg: String,
    data: LoginRes,
}
#[derive(Deserialize)]
struct LoginRes {
    access_token: String,
}

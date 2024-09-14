use crate::datetime::datetime_format;
use crate::main_select::MainSelect;
use crate::{delimiter, HOST};
use chrono::{DateTime, Local};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::time::Duration;

pub(crate) async fn register(name: String, password: String) {
    // 使用reqwest 向 POST HOST/user/register 接口注册用户
    let url = format!("{HOST}/user/register");
    let client = Client::new();
    let res = client
        .post(&url)
        .json(&serde_json::json!({
            "name": name,
            "password": password
        }))
        .send()
        .await;
    match res {
        Ok(res) => match res.status() {
            StatusCode::OK => {
                println!("恭喜{name}注册成功，请登陆场聊吧！")
            }
            StatusCode::CONFLICT => {
                println!("用户名已存在，请重新注册")
            }
            _ => {
                println!("注册失败: {}", res.text().await.unwrap())
            }
        },
        Err(err) => {
            println!("注册失败: {}", err)
        }
    }
}

thread_local! {
    pub static TOKEN: RefCell<String> = RefCell::new(String::default());
    pub static TOKEN_EXPIRE_TIME: RefCell<DateTime<Local>> = RefCell::new(Local::now());
}

pub(crate) async fn login(name: String, password: String) {
    let login_url = format!("{HOST}/token/login");
    let client = Client::new();
    let response = client
        .post(&login_url)
        .json(&serde_json::json!({
            "name": name,
            "password": password
        }))
        .send()
        .await;
    let token = match response {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<LoginRes>().await {
                    Ok(LoginRes { access_token }) => {
                        println!("登陆成功");
                        delimiter();
                        Some(access_token)
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
    let token = token.unwrap();
    TOKEN.replace(token.clone());
    // 启动异步线程，定时刷新token过期时间
    tokio::spawn(async move {
        loop {
            let renew_token_period = Duration::from_secs(30);
            println!("Refreshing token...");
            let renew_url = format!("{HOST}/token/renew");
            let client = Client::new();
            let response = client
                .patch(renew_url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await;
            match response {
                Ok(res) => {
                    if res.status().is_success() {
                        match res.text().await {
                            Ok(token) => {
                                TOKEN.replace(token.clone());
                            }
                            Err(e) => {
                                println!("Failed to parse response: {}", e);
                            }
                        }
                    } else {
                        println!("Token refresh failed: HTTP {}", res.status());
                    }
                }
                Err(err) => {
                    println!("Failed to send token refresh request: {}", err);
                }
            }
            tokio::time::sleep(renew_token_period).await;
        }
    });
    MainSelect::select().await;
}

#[derive(Deserialize)]
pub(crate) struct LoginRes {
    pub access_token: String,
}

use crate::main_select::MainSelect;
use crate::token::CURRENT_USER;
use crate::{delimiter, token, HOST};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use std::time::Duration;

pub(crate) async fn register(name: String, password: String) {
    let mail: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Your email")
        .validate_with({
            let mut force = None;
            move |input: &String| -> Result<(), &str> {
                if input.contains('@') || force.as_ref().map_or(false, |old| old == input) {
                    Ok(())
                } else {
                    force = Some(input.clone());
                    Err("This is not a mail address; type the same value again to force use")
                }
            }
        })
        .interact_text()
        .unwrap();

    println!("Email: {}", mail);

    let phone: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Your phone")
        .interact_text()
        .unwrap();

    println!("Phone: {}", phone);

    // 使用reqwest 向 POST HOST/user/register 接口注册用户
    let url = format!("{HOST}/user/register");
    let client = Client::new();
    let res = client
        .post(&url)
        .json(&serde_json::json!({
            "name": name,
            "password": password,
            "phone": phone,
            "mail": mail
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
    let user = token::parse_token(token.as_str()).await.unwrap().claims;
    {
        let mut guard = CURRENT_USER.lock().unwrap();
        guard.user = user; // Create a longer-lived binding
        guard.token = token; // Create a longer-lived binding
    }
    let token = format!("Bearer {}", CURRENT_USER.lock().unwrap().token);
    // 启动异步线程，定时刷新token过期时间
    tokio::spawn(async move {
        loop {
            let renew_token_period = Duration::from_secs(60);
            tokio::time::sleep(renew_token_period).await;
            let renew_url = format!("{HOST}/token/renew");
            let client = Client::new();
            let response = client
                .patch(renew_url)
                .header("Authorization", token.clone())
                .send()
                .await;
            match response {
                Ok(res) => {
                    if res.status().is_success() {
                        match res.text().await {
                            Ok(t) => {
                                let token_data = token::parse_token(t.as_str()).await.unwrap();
                                let mut guard = CURRENT_USER.lock().unwrap();
                                guard.user = token_data.claims;
                                guard.token = t;
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
        }
    });
    MainSelect::select().await;
}

#[derive(Deserialize)]
pub(crate) struct LoginRes {
    pub access_token: String,
}

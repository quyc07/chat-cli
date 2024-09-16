use crate::token::CURRENT_USER;
use crate::{style, HOST};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

pub(crate) async fn add_friend_select() {
    let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择：")
        .items(&vec!["添加好友", "好友申请"])
        .interact()
        .unwrap();
    match selection {
        0 => {
            add_friend().await;
        }
        1 => {
            friend_request().await;
        }
        _ => {
            println!("error selection");
            std::process::exit(1);
        }
    }
}

async fn friend_request() {
    todo!()
}

pub(crate) async fn add_friend() {
    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Friend name")
        .interact_text()
        .unwrap();
    style::loading(format!("搜索好友: {}", name));
    match find_friend(name).await {
        Ok(friends) => {
            let name_2_id = friends.into_iter().map(|friend| (friend.name, friend.id)).collect::<HashMap<String, i32>>();
            let names = name_2_id.keys().map(|name| name.as_str()).collect::<Vec<_>>();
            let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
                .with_prompt("请选择添加哪个好友")
                .items(&names)
                .interact()
                .unwrap();
            let url = format!("{HOST}/friend/req/{}", name_2_id.get(names[selection]).unwrap());
            let res = Client::new()
                .post(url)
                .header("Content-Type", "application/json")
                .header(
                    "Authorization",
                    format!("Bearer {}", CURRENT_USER.lock().unwrap().token),
                )
                .send()
                .await;
            match res {
                Ok(res) => {
                    if res.status().is_success() {
                        println!("添加成功");
                    } else {
                        println!("添加失败: {}", res.text().await.unwrap());
                    }
                }
                Err(err) => {
                    println!("Failed to send request: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Err(err) => {
            println!("Failed to find friend: {}", err);
            std::process::exit(1);
        }
    }
}

#[derive(Deserialize)]
struct FindFriendRes {
    id: i32,
    name: String,
}

async fn find_friend(name: String) -> Result<Vec<FindFriendRes>, String> {
    let url = format!("{HOST}/user/find/{name}");
    let res = Client::new()
        .get(url)
        .header(
            "Authorization",
            format!("Bearer {}", CURRENT_USER.lock().unwrap().token),
        )
        .send()
        .await;
    match res {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<Vec<FindFriendRes>>().await {
                    Ok(friends) => {
                        println!("Found {} friends", friends.len());
                        for friend in &friends {
                            println!("{}", friend.name);
                        }
                        Ok(friends)
                    }
                    Err(e) => {
                        println!("Failed to parse response: {}", e);
                        Err(e.to_string())
                    }
                }
            } else {
                println!("Find failed: HTTP {}", res.status());
                Err(res.text().await.unwrap())
            }
        }
        Err(err) => {
            println!("Failed to send find request: {}", err);
            Err(err.to_string())
        }
    }
}
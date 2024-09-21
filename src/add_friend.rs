use crate::datetime::datetime_format;
use crate::token::CURRENT_USER;
use crate::{style, HOST};
use chrono::{DateTime, Local};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input};
use indexmap::IndexMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

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

#[derive(Deserialize)]
struct FriendReqVo {
    id: i32,
    request_id: i32,
    request_name: String,
    #[serde(with = "datetime_format")]
    create_time: DateTime<Local>,
    reason: Option<String>,
    status: FriendRequestStatus,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FriendRequestStatus {
    WAIT,
    APPROVE,
    REJECT,
}

impl Display for FriendRequestStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FriendRequestStatus::WAIT => "待处理",
                FriendRequestStatus::APPROVE => "已同意",
                FriendRequestStatus::REJECT => "已拒绝",
            }
        )
    }
}

async fn friend_request() {
    let url = format!("{HOST}/friend/req");
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
                match res.json::<Vec<FriendReqVo>>().await {
                    Ok(friend_reqs) if !friend_reqs.is_empty() => {
                        let option_2_id = friend_reqs.into_iter().map(|req| {
                            (format!("姓名：{}\n  备注：{}\n  {}", req.request_name, req.reason.clone().unwrap_or("请求添加好友".to_string()), req.status), req)
                        }).collect::<IndexMap<String, FriendReqVo>>();
                        let options = option_2_id.keys().map(|x| x.clone()).collect::<Vec<_>>();
                        let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("好友申请列表")
                            .items(&options)
                            .interact()
                            .unwrap();
                        let req = option_2_id.get(&options[selection]).unwrap();
                        if req.status == FriendRequestStatus::WAIT {
                            if Confirm::with_theme(&ColorfulTheme::default())
                                .with_prompt("同意好友申请么？")
                                .interact()
                                .unwrap()
                            {
                                review(&req.id, FriendRequestStatus::APPROVE).await;
                            } else {
                                review(&req.id, FriendRequestStatus::REJECT).await;
                            }
                        }
                    }
                    Ok(_) => {
                        println!("暂无好友申请");
                        std::process::exit(1);
                    }
                    Err(e) => {
                        println!("Failed to parse response: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(err) => {
            println!("Failed to get friend request: {}", err);
            std::process::exit(1);
        }
    }
}

async fn review(id: &i32, status: FriendRequestStatus) {
    let url = format!("{HOST}/friend/req");
    let res = Client::new()
        .post(url)
        .header(
            "Authorization",
            format!("Bearer {}", CURRENT_USER.lock().unwrap().token),
        )
        .json(&serde_json::json!({
                                    "id": id,
                                    "status": status,
                                }))
        .send()
        .await;
    match res {
        Ok(res) => {
            if res.status().is_success() {
                println!("操作成功");
            } else {
                println!("操作失败: {}", res.text().await.unwrap());
            }
        }
        Err(err) => {
            println!("Failed to send request: {}", err);
            std::process::exit(1);
        }
    }
}

pub(crate) async fn add_friend() {
    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("好友名称")
        .interact_text()
        .unwrap();
    style::loading(format!("搜索好友: {}", name));
    match find_friend(name).await {
        Ok(friends) => {
            let name_2_id = friends.into_iter().map(|friend| (friend.name, friend.id)).collect::<IndexMap<String, i32>>();
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
                .json(&serde_json::json!({}))
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
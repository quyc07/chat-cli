use crate::datetime::datetime_format;
use crate::main_select::{Friend, MainSelect};
use crate::user::TOKEN;
use crate::{delimiter, HOST};
use chrono::{DateTime, Local};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

pub(crate) fn select(friends: Vec<Friend>) {
    let friend_names: Vec<&str> = friends.iter().map(|f| f.name.as_str()).collect();
    let selection = dialoguer::Select::new()
        .with_prompt(MainSelect::ChatWithFriends.to_str())
        .items(&friend_names)
        .interact()
        .unwrap();

    let selected_friend = &friends[selection];
    delimiter();
    // TODO 获取聊天记录
    chat_history_with_friend(selected_friend);
}

fn chat_history_with_friend(friend: &Friend) {
    let url = format!("{HOST}/user/{}/history", friend.id);
    let res = reqwest::blocking::Client::new()
        .get(url)
        .header("Authorization", format!("Bearer {}", TOKEN.with_borrow(|t| t.clone())))
        .send();

    match res {
        Ok(res) => {
            match res.status() {
                StatusCode::OK => {
                    let res = res.json::<Vec<UserHistoryMsg>>();
                    match res {
                        Ok(res) => {
                            for msg in res {
                                println!("{}", serde_json::to_string(&msg).unwrap());
                            }
                        }
                        Err(err) => {
                            println!("Failed to parse chat history:{}", err);
                            return;
                        }
                    }
                }
                _ => {
                    println!("Failed to get chat history:{}", res.status());
                    return;
                }
            }
        }
        Err(err) => {
            println!("Failed to get chat history:{}", err);
            return;
        }
    }
}

/// 历史聊天记录
#[derive(Deserialize, Serialize)]
struct UserHistoryMsg {
    /// 消息id
    mid: i64,
    /// 消息内容
    msg: String,
    /// 消息发送时间
    #[serde(with = "datetime_format")]
    time: DateTime<Local>,
    /// 消息发送者id
    from_uid: i32,
}
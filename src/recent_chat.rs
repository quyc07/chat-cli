use crate::datetime::datetime_format;
use crate::friend::Friend;
use crate::token::CURRENT_USER;
use crate::{console, delimiter, friend, HOST};
use chrono::{DateTime, Local};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use indexmap::IndexMap;

pub(crate) async fn recent_chat() {
    delimiter();
    let url = format!("{HOST}/user/history/100");
    let res = Client::new()
        .get(url)
        .header(
            "Authorization",
            format!("Bearer {}", CURRENT_USER.lock().unwrap().token),
        )
        .send()
        .await;
    if let Ok(res) = res {
        if res.status().is_success() {
            let res = res.json::<Vec<ChatVo>>().await;
            if let Ok(chatVos) = res {
                let select_to_id: IndexMap<String, (Option<i32>, Option<i32>, String)> = chatVos.into_iter().map(|chat_vo| {
                    match chat_vo {
                        ChatVo::User {
                            uid,
                            user_name,
                            mid: _mid,
                            msg,
                            msg_time,
                            unread,
                        } => {
                            delimiter();
                            match unread {
                                None => (format!("好友: {}\n  时间: {}\n  {}", user_name, msg_time, msg), (Some(uid), None, user_name)),
                                Some(unread) => (format!("好友: {}\n  时间: {}\n  {}\n  未读: {}", user_name, msg_time, msg, unread), (Some(uid), None, user_name)),
                            }
                        }
                        ChatVo::Group {
                            gid,
                            group_name,
                            uid: _uid,
                            user_name,
                            mid: _mid,
                            msg,
                            msg_time,
                            unread,
                        } => {
                            delimiter();
                            match unread {
                                None => (format!("群: {}\n  时间: {}\n  {}: {}", group_name, msg_time, user_name, msg), (None, Some(gid), group_name)),
                                Some(unread) => (format!("群: {}\n  时间: {}\n  {}: {}\n  未读: {}", user_name, msg_time, user_name, msg, unread), (None, Some(gid), group_name)),
                            }
                        }
                    }
                }).collect();
                let options = select_to_id.keys().map(|s| s.as_str()).collect::<Vec<_>>();
                let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("最近聊天列表")
                    .items(&options)
                    .interact()
                    .unwrap();
                match select_to_id.get(options[selection]).unwrap() {
                    (Some(uid), None, name) => {
                        console::clean_all();
                        friend::chat_with_friend(&Friend { id: *uid, name: name.to_string() }).await;
                    }
                    (None, Some(gid), name) => { todo!() }
                    _ => {
                        println!("error selection");
                        std::process::exit(1);
                    }
                };
            }
        }
    }
}

/// 聊天记录
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
enum ChatVo {
    /// UserChat
    User {
        /// id of friend
        uid: i32,
        /// name of friend
        user_name: String,
        /// message id
        mid: i64,
        /// message content
        msg: String,
        /// message time
        #[serde(with = "datetime_format")]
        msg_time: DateTime<Local>,
        /// unread message count
        unread: Option<String>,
    },
    /// GroupChat
    Group {
        /// id of group
        gid: i32,
        /// name of group
        group_name: String,
        /// id of friend
        uid: i32,
        /// name of friend
        user_name: String,
        /// message id
        mid: i64,
        /// message content
        msg: String,
        /// message time
        #[serde(with = "datetime_format")]
        msg_time: DateTime<Local>,
        /// unread message count
        unread: Option<String>,
    },
}
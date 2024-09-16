use crate::datetime::datetime_format;
use crate::token::CURRENT_USER;
use crate::{delimiter, HOST};
use chrono::{DateTime, Local};
use reqwest::Client;
use serde::{Deserialize, Serialize};

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
                for chatVo in chatVos {
                    match chatVo {
                        ChatVo::User {
                            uid,
                            user_name,
                            mid: _mid,
                            msg,
                            msg_time,
                            unread,
                        } => {
                            match unread {
                                None => println!("好友: {}\n时间: {}\n{}", user_name, msg_time, msg),
                                Some(unread) => println!("好友: {}\n时间: {}\n{}\n未读: {}", user_name, msg_time, msg, unread),
                            };
                            delimiter();
                        }
                        ChatVo::Group {
                            gid,
                            group_name,
                            uid,
                            user_name,
                            mid,
                            msg,
                            msg_time,
                            unread,
                        } => {
                            match unread {
                                None => println!("群: {}\n时间: {}\n{}: {}", group_name, msg_time, user_name, msg),
                                Some(unread) => println!("群: {}\n时间: {}\n{}: {}\n未读: {}", user_name, msg_time, user_name, msg, unread),
                            }
                            delimiter();
                        }
                    }
                }
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
use crate::main_select::MainSelect::{AddFriend, ChatHistory, ChatInGroups, ChatWithFriends};
use crate::user::TOKEN;
use crate::{friend, HOST};
use reqwest::Client;
use serde::Deserialize;

pub(crate) enum MainSelect {
    AddFriend,
    ChatHistory,
    ChatWithFriends,
    ChatInGroups,
}

fn main_selects() -> Vec<&'static str> {
    vec![
        AddFriend.to_str(),
        ChatHistory.to_str(),
        ChatWithFriends.to_str(),
        ChatInGroups.to_str(),
    ]
}
impl MainSelect {
    pub(crate) fn to_str(&self) -> &str {
        match self {
            AddFriend => "添加好友",
            ChatHistory => "聊天列表",
            ChatWithFriends => "好友列表",
            ChatInGroups => "群聊列表",
        }
    }

    // 字符串转枚举
    fn from_str(s: &str) -> Self {
        match s {
            "添加好友" => AddFriend,
            "聊天列表" => ChatHistory,
            "好友列表" => ChatWithFriends,
            "群聊列表" => ChatInGroups,
            _ => panic!("Invalid string"),
        }
    }
    pub(crate) async fn select() {
        let options = main_selects();
        let selection = dialoguer::Select::new()
            .with_prompt("请选择")
            .items(&options)
            .interact()
            .unwrap();
        let select = MainSelect::from_str(options[selection]);
        select.do_select().await;
    }

    async fn do_select(&self) {
        match self {
            AddFriend => add_friend(),
            ChatHistory => chat_history(),
            ChatWithFriends => chat_with_friends().await,
            ChatInGroups => chat_in_groups(),
        }
    }
}

fn chat_history() {
    todo!()
}

fn chat_in_groups() {
    todo!()
}

async fn chat_with_friends() {
    let client = Client::new();
    let friends_url = format!("{HOST}/friend");
    let response = client
        .get(&friends_url)
        .header(
            "Authorization",
            format!("Bearer {}", TOKEN.with_borrow(|t| t.clone())),
        )
        .send()
        .await;

    let friends = match response {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<Vec<Friend>>().await {
                    Ok(friends) => Some(friends),
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
    friend::select(friends.unwrap()).await;
}

fn add_friend() {
    todo!()
}

#[derive(Deserialize)]
pub(crate) struct Friend {
    pub(crate) id: i32,
    pub(crate) name: String,
}

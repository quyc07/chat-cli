use crate::main_select::MainSelect::{AddFriend, ChatInGroups, ChatWithFriends};
use crate::{friend, HOST};
use serde::Deserialize;
use crate::user::TOKEN;

pub(crate) enum MainSelect {
    AddFriend,
    ChatWithFriends,
    ChatInGroups,
}


fn main_selects() -> Vec<&'static str> {
    vec![AddFriend.to_str(), ChatWithFriends.to_str(), ChatInGroups.to_str()]
}
impl MainSelect {
    fn to_str(&self) -> &str {
        match self {
            AddFriend => "添加好友",
            ChatWithFriends => "好友列表",
            ChatInGroups => "群聊列表",
        }
    }

    // 字符串转枚举
    fn from_str(s: &str) -> Self {
        match s {
            "添加好友" => AddFriend,
            "好友列表" => ChatWithFriends,
            "群聊列表" => ChatInGroups,
            _ => panic!("Invalid string"),
        }
    }
    pub(crate) fn select() {
        let options = main_selects();
        let selection = dialoguer::Select::new()
            .with_prompt("请选择：")
            .items(&options)
            .interact()
            .unwrap();
        let select = MainSelect::from_str(options[selection]);
        select.do_select();
    }

    fn do_select(&self) {
        match self {
            AddFriend => add_friend(),
            ChatWithFriends => chat_with_friends(),
            ChatInGroups => chat_in_groups(),
        }
    }
}

fn chat_in_groups() {
    todo!()
}

fn chat_with_friends() {
    let client = reqwest::blocking::Client::new();
    let friends_url = format!("{HOST}/friend");
    let response = client
        .get(&friends_url)
        .header("Authorization", format!("Bearer {}", TOKEN.with_borrow(|t| t.clone())))
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
    friend::select(friends.unwrap());
}

fn add_friend() {
    todo!()
}

#[derive(Deserialize)]
struct FriendsRes {
    code: i8,
    msg: String,
    data: Vec<Friend>,
}

#[derive(Deserialize)]
pub(crate) struct Friend {
    pub(crate) id: i32,
    pub(crate) name: String,
}
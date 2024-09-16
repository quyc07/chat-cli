use crate::main_select::MainSelect::{AddFriend, ChatInGroups, ChatWithFriends, RecentChat};
use crate::{recent_chat, friend};

pub(crate) enum MainSelect {
    AddFriend,
    RecentChat,
    ChatWithFriends,
    ChatInGroups,
}

impl MainSelect {
    fn selects() -> Vec<&'static str> {
        vec![
            AddFriend.to_str(),
            RecentChat.to_str(),
            ChatWithFriends.to_str(),
            ChatInGroups.to_str(),
        ]
    }

    pub(crate) fn to_str(&self) -> &str {
        match self {
            AddFriend => "添加好友",
            RecentChat => "最近消息",
            ChatWithFriends => "好友列表",
            ChatInGroups => "群聊列表",
        }
    }

    // 字符串转枚举
    fn from_str(s: &str) -> Self {
        match s {
            "添加好友" => AddFriend,
            "最近消息" => RecentChat,
            "好友列表" => ChatWithFriends,
            "群聊列表" => ChatInGroups,
            _ => panic!("Invalid string"),
        }
    }
    pub(crate) async fn select() {
        let options = MainSelect::selects();
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
            RecentChat => recent_chat::recent_chat().await,
            ChatWithFriends => friend::chat_with_friends().await,
            ChatInGroups => chat_in_groups(),
        }
    }
}

fn add_friend() {
    todo!()
}

fn chat_in_groups() {
    todo!()
}

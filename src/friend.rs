use crate::datetime::datetime_format;
use crate::main_select::{Friend, MainSelect};
use crate::user::TOKEN;
use crate::{delimiter, HOST};
use chrono::{DateTime, Local};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::io::BufRead;

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
                            println!("Chat with {}:", friend.name);
                            println!("----------------------------------------");
                            if res.is_empty() {
                                println!("No chat history available.");
                            } else {
                                for msg in res {
                                    let sender = if msg.from_uid == friend.id {
                                        &friend.name
                                    } else {
                                        "You"
                                    };
                                    println!("[{}] {}: {}", msg.time.format("%Y-%m-%d %H:%M:%S"), sender, msg.msg);
                                }
                            }
                            println!("----------------------------------------");
                            println!("Listening for new messages...");
                            
                            let client = reqwest::blocking::Client::new();
                            let response = client.get(format!("{}/event/stream", HOST))
                                .header("Authorization", format!("Bearer {}", TOKEN.with_borrow(|t| t.clone())))
                                .header("User-Agent", "Chat-Cli/1.0")
                                .send()
                                .expect("Failed to connect to SSE stream");

                            let mut reader = std::io::BufReader::new(response);
                            let mut line = String::new();

                            loop {
                                line.clear();
                                if let Ok(bytes_read) = reader.read_line(&mut line) {
                                    if bytes_read == 0 {
                                        break;
                                    }
                                    if line.starts_with("data:") {
                                        let event_data = line.trim_start_matches("data:").trim();
                                        match serde_json::from_str::<Message>(event_data) {
                                            Ok(Message::ChatMessage(chat_message)) => {
                                                let sender = if chat_message.payload.from_uid == friend.id {
                                                    &friend.name
                                                } else {
                                                    "You"
                                                };
                                                println!("[{}] {}: {}", 
                                                    chat_message.payload.created_at.format("%Y-%m-%d %H:%M:%S"), 
                                                    sender, 
                                                    chat_message.payload.detail.get_content(),
                                                );
                                            }
                                            Ok(Message::Heartbeat(heartbeat_message)) => {
                                                println!("Heartbeat received: {:?}", heartbeat_message);
                                            }
                                            Err(err) => {
                                                eprintln!("Failed to parse event data: {}", err);
                                            }
                                        }
                                    }
                                }
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

#[derive(Debug, Clone, Serialize,Deserialize)]
pub enum Message {
    ChatMessage(ChatMessage),
    Heartbeat(HeartbeatMessage),
}

// 也可以使用strum库来实现
impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Message::ChatMessage(_) => "Chat",
                Message::Heartbeat(_) => "Heartbeat",
            }
        )
    }
}

#[derive(Debug, Clone, Serialize,Deserialize)]
pub struct HeartbeatMessage {
    #[serde(with = "datetime_format")]
    time: DateTime<Local>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ChatMessage {
    /// Message id
    pub mid: i64,
    pub payload: ChatMessagePayload,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessagePayload {
    /// Sender id
    pub from_uid: i32,

    #[serde(with = "datetime_format")]
    /// The create time of the message.
    pub created_at: DateTime<Local>,

    /// Message target
    pub target: MessageTarget,

    /// Message detail
    pub detail: MessageDetail,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum MessageTarget {
    User(MessageTargetUser),
    Group(MessageTargetGroup),
}

impl From<MessageTarget> for String {
    fn from(value: MessageTarget) -> Self {
        match value {
            MessageTarget::User(MessageTargetUser { uid }) => format!("MessageTargetUser:{uid}"),
            MessageTarget::Group(MessageTargetGroup { gid }) => {
                format!("MessageTargetGroup:{gid}")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct MessageTargetUser {
    pub uid: i32,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct MessageTargetGroup {
    pub gid: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MessageDetail {
    Normal(MessageNormal),
    Replay(MessageReplay),
}

impl MessageDetail {
    pub fn get_content(&self) -> String {
        match self {
            MessageDetail::Normal(msg) => msg.content.content.clone(),
            MessageDetail::Replay(msg) => msg.content.content.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageNormal {
    pub content: MessageContent,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageReplay {
    pub mid: i64,
    pub content: MessageContent,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageContent {
    /// Extended attributes
    // pub properties: Option<HashMap<String, Value>>,
    /// Content type
    // pub content_type: String,
    /// Content
    pub(crate) content: String,
}
use crate::datetime::datetime_format;
use crate::main_select::MainSelect;
use crate::token::CURRENT_USER;
use crate::{delimiter, HOST};
use chrono::{DateTime, Local};
use crossterm::terminal::ClearType::CurrentLine;
use crossterm::{cursor, execute, terminal};
use futures::StreamExt;
use regex::Regex;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::io::{stdout, Write};
use tokio::io::AsyncBufReadExt;


#[derive(Deserialize)]
pub(crate) struct Friend {
    pub(crate) id: i32,
    pub(crate) name: String,
}

pub(crate) async fn find_friends() {
    let client = Client::new();
    let friends_url = format!("{HOST}/friend");
    let response = client
        .get(&friends_url)
        .header(
            "Authorization",
            format!("Bearer {}", CURRENT_USER.lock().unwrap().token),
        )
        .send()
        .await;
    match response {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<Vec<Friend>>().await {
                    Ok(friends) if !friends.is_empty() => {
                        select_friend(friends).await;
                    }
                    Ok(_) => {
                        println!("暂无好友");
                        std::process::exit(1);
                    }
                    Err(e) => {
                        println!("Failed to read response: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("Failed to get friends list: HTTP {}", res.status());
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("Failed to send request: {}", e);
            std::process::exit(1);
        }
    };
}

pub(crate) async fn select_friend(friends: Vec<Friend>) {
    let friend_names: Vec<&str> = friends.iter().map(|f| f.name.as_str()).collect();
    match dialoguer::Select::new()
        .with_prompt(MainSelect::ChatWithFriends.to_str())
        .items(&friend_names)
        .interact() {
        Ok(selection) => {
            let selected_friend = &friends[selection];
            delimiter();
            chat_with_friend(selected_friend).await;
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };
}

pub(crate) async fn chat_with_friend(selected_friend: &Friend) {
    let latest_mid = fetch_history(selected_friend).await;
    if let Some(latest_mid) = latest_mid {
        set_read_index(UpdateReadIndex::User { target_uid: selected_friend.id, mid: latest_mid }).await;
    }
    chat(selected_friend).await;
}

#[derive(Serialize)]
enum UpdateReadIndex {
    User { target_uid: i32, mid: i64 },
    Group { target_gid: i32, mid: i64 },
}

async fn set_read_index(ri: UpdateReadIndex) {
    let client = Client::new();
    client
        .put(format!("{HOST}/ri"))
        .header(
            "Authorization",
            format!("Bearer {}", CURRENT_USER.lock().unwrap().token),
        )
        .body(serde_json::to_string(&ri).unwrap())
        .send()
        .await.expect("unable to set read index");
}

async fn chat(friend: &Friend) {
    let client = Client::new();
    let mut sse_stream = client
        .get(format!("{HOST}/event/stream"))
        .header(
            "Authorization",
            format!("Bearer {}", CURRENT_USER.lock().unwrap().token),
        )
        .header("User-Agent", "Chat-Cli/1.0")
        .send()
        .await
        .unwrap()
        .bytes_stream();

    // 异步监听用户输入，使用tokio::io::BufReader及时获取用户输入数据
    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);

    loop {
        let mut input = String::new();
        let input_future = reader.read_line(&mut input);
        tokio::select! {
            // 处理从SSE流中接收到的消息
            Some(msg) = sse_stream.next() => {
                match msg {
                    Ok(bytes) => {
                        let sse_message = String::from_utf8(bytes.to_vec()).unwrap();
                        // 获取data
                        let data = sse_message.lines()
                        .into_iter()
                        .find(|line| line.starts_with("data:"))
                        .map(|line| line.trim_start_matches("data:").trim())
                        .filter(|line| !line.is_empty());
                        if let Some(event_data) = data {
                            match serde_json::from_str::<Message>(event_data) {
                                Ok(Message::ChatMessage(chat_message)) => {
                                    let sender = if chat_message.payload.from_uid == friend.id {
                                        &friend.name
                                    } else {
                                        // 清空用户输入的那一行
                                        execute!(stdout(),cursor::MoveUp(1)).unwrap();
                                        execute!(stdout(),terminal::Clear(CurrentLine)).unwrap();
                                        "You"
                                    };
                                    println!("[{}] {}: {}",
                                             chat_message.payload.created_at.format("%Y-%m-%d %H:%M:%S"),
                                             sender,
                                             chat_message.payload.detail.get_content(),
                                    );
                                    // 刷出数据
                                    stdout().flush().unwrap();
                                }
                                Ok(Message::Heartbeat(_)) => {
                                    // println!("Heartbeat received: {:?}", heartbeat_message);
                                }
                                Err(err) => {
                                    eprintln!("Failed to parse event data: {}", err);
                                }
                            }

                        }
                    },
                    Err(e) => {
                        eprintln!("SSE错误: {}", e);
                        break;
                    }
                }
            }
            // 处理用户输入
            Ok(_) = input_future => {
                if input.trim() == "exit" {
                    println!("退出...");
                    break;
                } else if !input.trim().is_empty() {
                    let url = format!("{HOST}/user/{}/send", friend.id);
                    let res = client
                        .post(&url)
                        .header("Content-Type", "application/json")
                        .header(
                            "Authorization",
                            format!("Bearer {}", CURRENT_USER.lock().unwrap().token),
                        )
                        .body(serde_json::json!({
                            "msg": replace_whitespace(&input),
                        }).to_string())
                        .send()
                        .await;

                    match res {
                        Ok(res) => {
                            if res.status() != StatusCode::OK {
                                println!("Send message failed: {}, {}", res.status(), res.text().await.unwrap());
                            }
                        }
                        Err(err) => {
                            println!("Send message failed: {}", err);
                        }
                    }
                }
            }
        }
    }
}

// 替换掉换行符
fn replace_whitespace(text: &str) -> String {
    let re = Regex::new(r"[\s\r\n]+").unwrap();
    re.replace_all(&text, "").into_owned()
}

async fn fetch_history(friend: &Friend) -> Option<i64> {
    let url = format!("{HOST}/user/{}/history", friend.id);
    let res = Client::new()
        .get(url)
        .header(
            "Authorization",
            format!("Bearer {}", CURRENT_USER.lock().unwrap().token),
        )
        .send()
        .await;
    match res {
        Ok(res) => match res.status() {
            StatusCode::OK => {
                let res = res.json::<Vec<UserHistoryMsg>>().await;
                match res {
                    Ok(res) => {
                        println!("Chat with {}:", friend.name);
                        println!("----------------------------------------");
                        if res.is_empty() {
                            println!("No chat history available.");
                            None
                        } else {
                            for msg in &res {
                                let sender = if msg.from_uid == friend.id {
                                    &friend.name
                                } else {
                                    "You"
                                };
                                println!(
                                    "[{}] {}: {}",
                                    msg.time.format("%Y-%m-%d %H:%M:%S"),
                                    sender,
                                    msg.msg
                                );
                            }
                            Some(res.last()?.mid)
                        }
                    }
                    Err(err) => {
                        println!("Failed to parse chat history:{}", err);
                        None
                    }
                }
            }
            _ => {
                println!("Failed to get chat history:{}", res.status());
                None
            }
        },
        Err(err) => {
            println!("Failed to get chat history:{}", err);
            None
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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


#[cfg(test)]
mod test {
    use serde_json::json;

    #[test]
    fn test_get_friend_history() {
        let history = json!({"ChatMessage":{"mid":98,"payload":{"from_uid":10,"created_at":"2024-09-12T23:15:05.264972+08:00","target":{"User":{"uid":11}},"detail":{"Normal":{"content":{"content":"hello world!!!!!"}}}}}});
        let result = serde_json::from_slice::<super::Message>(history.to_string().as_bytes());
        match result {
            Ok(msg) => {
                println!("{:?}", msg)
            }
            Err(err) => {
                println!("{}", err)
            }
        }
    }
}

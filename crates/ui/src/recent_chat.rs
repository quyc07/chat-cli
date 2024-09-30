use crate::datetime::datetime_format;
use crate::token::CURRENT_USER;
use crate::ui::total_area;
use crate::HOST;
use chrono::{DateTime, Local};
use color_eyre::eyre::format_err;
use color_eyre::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Color, Style};
use ratatui::style::palette::material::{BLUE, GREEN};
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::{Modifier, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget};
use ratatui::{symbols, DefaultTerminal};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;

const TODO_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;

pub(crate) struct RecentChat {
    error_message: Option<String>,
    should_exit: bool,
    chat_list: ChatList,
    selected_chat_history: Option<SelectedChatHistory>,
}

enum SelectedChatHistory {
    User {
        uid: i32,
        user_name: String,
        history: Vec<UserHistoryMsg>,
    },
    Group {
        gid: i32,
        group_name: String,
        history: Vec<GroupHistoryMsg>,
    },
}

impl RecentChat {
    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, →/Enter to change status, g/G or Home/End to go top/bottom.")
            .centered()
            .render(area, buf);
    }
}

struct ChatList {
    items: Arc<Mutex<Vec<ChatVo>>>,
    state: ListState,
}

impl RecentChat {
    pub(crate) fn new() -> Result<Self> {
        let chat_list = ChatList {
            items: Arc::new(Mutex::new(recent_chat()?)),
            state: ListState::default(),
        };

        let chat = Self {
            error_message: None,
            should_exit: false,
            chat_list,
            selected_chat_history: None,
        };
        chat.start_update_thread();
        Ok(chat)
    }

    fn start_update_thread(&self) {
        let value_clone = self.chat_list.items.clone();
        thread::spawn(move || loop {
            // 尽快释放锁，方便数据呈现
            {
                let mut value = value_clone.lock().unwrap();
                match recent_chat() {
                    Ok(items) => {
                        *value = items;
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                    }
                }
            }
            thread::sleep(std::time::Duration::from_secs(5));
        });
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.clear()?;
            terminal.draw(|frame| {
                let rect = total_area(frame);
                frame.render_widget(&mut self, rect)
            })?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key)?
            };
        }
        Ok(())
    }

    fn handle_key(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(self.should_exit = true),
            KeyCode::Down => self.select_next(),
            KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            _ => Ok(())
        }
    }
    fn select_next(&mut self) -> Result<()> {
        self.chat_list.state.select_next();
        self.fetch_history_and_update_read_index()
    }

    fn select_previous(&mut self) -> Result<()> {
        self.chat_list.state.select_previous();
        self.fetch_history_and_update_read_index()
    }
    fn select_first(&mut self) -> Result<()> {
        self.chat_list.state.select_first();
        self.fetch_history_and_update_read_index()
    }
    fn select_last(&mut self) -> Result<()> {
        self.chat_list.state.select_last();
        self.fetch_history_and_update_read_index()
    }

    fn fetch_history_and_update_read_index(&mut self) -> Result<()> {
        let chat_list = self.chat_list.items.lock().unwrap();
        match self.chat_list.state.selected() {
            Some(i) if i < chat_list.len() => {
                let chat_vo = &chat_list[i];
                match chat_vo {
                    ChatVo::User { uid, user_name, .. } => {
                        match fetch_user_history(*uid) {
                            Ok(chat_history) => {
                                let last_mid = chat_history.last().unwrap().mid;
                                self.selected_chat_history = Some(SelectedChatHistory::User {
                                    uid: *uid,
                                    user_name: user_name.clone(),
                                    history: chat_history,
                                });
                                // 更新 已读索引
                                set_read_index(UpdateReadIndex::User { target_uid: *uid, mid: last_mid })
                            }
                            Err(err) => Err(format_err!("Failed to fetch chat history:{}",err)),
                        }
                    }
                    ChatVo::Group { gid, group_name, .. } => {
                        match fetch_group_history(*gid) {
                            Ok(chat_history) => {
                                let last_mid = chat_history.last().unwrap().mid;
                                self.selected_chat_history = Some(SelectedChatHistory::Group {
                                    gid: *gid,
                                    group_name: group_name.clone(),
                                    history: chat_history,
                                });
                                // 更新 已读索引
                                set_read_index(UpdateReadIndex::Group { target_gid: *gid, mid: last_mid })
                            }
                            Err(err) => Err(format_err!("Failed to fetch chat history:{}",err))
                        }
                    }
                }
            }
            _ => Ok(())
        }
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Recent Chat").centered())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .chat_list
            .items.lock().unwrap()
            .iter()
            .enumerate()
            .map(|(i, chat_vo)| {
                let color = alternate_colors(i);
                ListItem::new(Text::from(chat_vo)).bg(color)
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.chat_list.state);
    }

    fn render_chat(&self, area: Rect, buf: &mut Buffer) {
        let (items, title) = match &self.selected_chat_history {
            Some(SelectedChatHistory::User { uid, user_name, history }) => {
                let vec: Vec<ListItem> = history
                    .iter()
                    .enumerate()
                    .map(|(i, chat)| {
                        let color = alternate_colors(i);
                        let content = vec![
                            Line::from(vec![
                                Span::styled(if *uid == chat.from_uid { format!("{:width$}", "你", width = 49) } else { format!("{:width$}", user_name, width = 50) }, Style::default().fg(Color::LightBlue)),
                                Span::styled(format!("{}", chat.time), Style::default().fg(Color::Gray)),
                            ]),
                            Line::from(Span::styled(format!("{}", chat.msg), Style::default().fg(Color::White))),
                        ];
                        ListItem::new(Text::from(content)).bg(color)
                    })
                    .collect();
                (vec, format!("与{}的聊天记录", user_name))
            }
            Some(SelectedChatHistory::Group { gid, group_name, history }) => {
                let vec: Vec<ListItem> = history
                    .iter()
                    .enumerate()
                    .map(|(i, chat)| {
                        let color = alternate_colors(i);
                        let uid = CURRENT_USER.lock().unwrap().user.clone().unwrap().id;
                        let content = vec![
                            Line::from(vec![
                                Span::styled(if uid == chat.from_uid { format!("{:width$}", "你", width = 49) } else { format!("{:width$}", chat.name_of_from_uid, width = 50) }, Style::default().fg(Color::LightBlue)),
                                Span::styled(format!("{}", chat.time), Style::default().fg(Color::Gray)),
                            ]),
                            Line::from(Span::styled(format!("{}", chat.msg), Style::default().fg(Color::White))),
                        ];
                        ListItem::new(Text::from(content)).bg(color)
                    })
                    .collect();
                (vec, format!("{group_name}"))
            }
            None => {
                return;
            }
        };
        let block = Block::new()
            .title(Line::raw(title).centered())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        let list = List::new(items)
            .block(block);

        Widget::render(list, area, buf);
    }
}

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

impl Widget for &mut RecentChat {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [main_area, footer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
            .areas(area);

        RecentChat::render_footer(footer_area, buf);
        
        if self.chat_list.state.selected().is_some() {
            let [list_area, chat_area] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Fill(2),
            ])
                .areas(main_area);
            self.render_list(list_area, buf);
            self.render_chat(chat_area, buf);
        } else {
            self.render_list(main_area, buf);
        }
    }
}

fn recent_chat() -> Result<Vec<ChatVo>> {
    let url = format!("{HOST}/user/history/100");
    let token = CURRENT_USER.lock().unwrap().token.clone().unwrap();
    let res = Client::new()
        .get(url)
        .header(
            "Authorization",
            format!("Bearer {}", token),
        )
        .send();
    if let Ok(res) = res {
        if res.status().is_success() {
            res.json::<Vec<ChatVo>>().map_err(|err| { format_err!("Fail to Parse Recent Chat: {}", err) })
        } else {
            Err(format_err!("Fail to Get Recent Chat"))
        }
    } else {
        Err(format_err!("Fail to Get Recent Chat"))
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

impl ChatVo {
    fn get_name(&self) -> String {
        match self {
            ChatVo::User { user_name, .. } => user_name.clone(),
            ChatVo::Group { user_name, .. } => user_name.clone(),
        }
    }
}

impl From<&ChatVo> for Text<'_> {
    fn from(value: &ChatVo) -> Self {
        match value {
            ChatVo::User {
                uid: _uid,
                user_name,
                msg,
                msg_time,
                unread,
                ..
            } => {
                let mut content = vec![
                    Line::from(Span::styled(format!("好友: {}\n", user_name), Style::default().fg(Color::LightBlue))),
                    Line::from(Span::styled(format!("时间: {}\n", msg_time), Style::default().fg(Color::LightBlue))),
                    Line::from(Span::styled(format!("{}\n", msg), Style::default().fg(Color::White))),
                ];
                if let Some(unread) = unread {
                    content.push(Line::from(Span::styled(format!("未读: {}\n", unread), Style::default().fg(Color::LightBlue))))
                }
                Self::from(content)
            }
            ChatVo::Group {
                gid: _gid,
                group_name,
                user_name,
                msg,
                msg_time,
                unread,
                ..
            } => {
                let mut content = vec![
                    Line::from(Span::styled(format!("群: {}\n", group_name), Style::default().fg(Color::LightBlue))),
                    Line::from(Span::styled(format!("时间: {}\n", msg_time), Style::default().fg(Color::LightBlue))),
                    Line::from(Span::styled(format!("{}: {}\n", user_name, msg), Style::default().fg(Color::White))),
                ];
                if let Some(unread) = unread {
                    content.push(Line::from(Span::styled(format!("未读: {}\n", unread), Style::default().fg(Color::LightBlue))))
                }
                Self::from(content)
            }
        }
    }
}

fn fetch_user_history(target_uid: i32) -> Result<Vec<UserHistoryMsg>> {
    let url = format!("{HOST}/user/{target_uid}/history");
    let token = CURRENT_USER.lock().unwrap().token.clone().unwrap();
    let res = Client::new()
        .get(url)
        .header(
            "Authorization",
            format!("Bearer {token}"),
        )
        .send();
    match res {
        Ok(res) => match res.status() {
            StatusCode::OK => {
                let res = res.json::<Vec<UserHistoryMsg>>();
                res.or_else(|e| Err(format_err!("Failed to get chat history :{}",e)))
            }
            _ => {
                Err(format_err!("Failed to get chat history:{}", res.status()))
            }
        },
        Err(err) => {
            Err(format_err!("Failed to get chat history:{}", err))
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

#[derive(Serialize, Deserialize)]
struct GroupHistoryMsg {
    pub mid: i64,
    pub msg: String,
    #[serde(with = "datetime_format")]
    pub time: DateTime<Local>,
    pub from_uid: i32,
    pub name_of_from_uid: String,
}

fn fetch_group_history(gid: i32) -> Result<Vec<GroupHistoryMsg>> {
    let url = format!("{HOST}/group/{gid}/history");
    let token = CURRENT_USER.lock().unwrap().token.clone().unwrap();
    let res = Client::new()
        .get(url)
        .header(
            "Authorization",
            format!("Bearer {token}"),
        )
        .send();
    match res {
        Ok(res) => match res.status() {
            StatusCode::OK => {
                let res = res.json::<Vec<GroupHistoryMsg>>();
                res.or_else(|e| Err(format_err!("Failed to get chat history :{}",e)))
            }
            _ => {
                Err(format_err!("Failed to get chat history:{}", res.status()))
            }
        },
        Err(err) => {
            Err(format_err!("Failed to get chat history:{}", err))
        }
    }
}

#[derive(Serialize)]
enum UpdateReadIndex {
    User { target_uid: i32, mid: i64 },
    Group { target_gid: i32, mid: i64 },
}

fn set_read_index(ri: UpdateReadIndex) -> Result<()> {
    let token = CURRENT_USER.lock().unwrap().token.clone().unwrap();
    let res = Client::new()
        .put(format!("{HOST}/ri"))
        .header(
            "Authorization",
            format!("Bearer {token}"),
        )
        .json(&ri)
        .send();
    match res {
        Ok(res) => {
            println!("{}", res.text().unwrap());
            Ok(())
        }
        Err(err) => Err(format_err!("Failed to set read index:{}", err))
    }
}
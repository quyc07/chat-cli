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
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph, StatefulWidget, Widget, Wrap};
use ratatui::{symbols, DefaultTerminal};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

const TODO_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;

pub(crate) struct RecentChat {
    error_message: Option<String>, // 添加错误消息字段
    should_exit: bool,
    chat_list: ChatList,
}

impl RecentChat {
    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, →/Enter to change status, g/G or Home/End to go top/bottom.")
            .centered()
            .render(area, buf);
    }
}

struct ChatList {
    items: Vec<ChatVo>, // TODO 需要异步刷新来获取最新消息
    state: ListState,
}

impl RecentChat {
    pub(crate) fn new() -> Result<Self> {
        let chat_list = ChatList {
            items: recent_chat()?,
            state: ListState::default(),
        };
        Ok(Self {
            error_message: None,
            should_exit: false,
            chat_list,
        })
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| {
                let rect = total_area(frame);
                frame.render_widget(&mut self, rect)
            })?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    fn handle_key(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Down => self.select_next(),
            KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Right | KeyCode::Enter => {
                self.to_chat();
            }
            _ => {}
        }
    }
    fn select_next(&mut self) {
        self.chat_list.state.select_next()
    }
    fn select_previous(&mut self) {
        self.chat_list.state.select_previous()
    }
    fn select_first(&mut self) {
        self.chat_list.state.select_first()
    }
    fn select_last(&mut self) {
        self.chat_list.state.select_last()
    }
    fn to_chat(&self) {
        let index = self.chat_list.state.selected();
        match index {
            Some(index) => {
                let chat_vo = &self.chat_list.items[index];
                match chat_vo {
                    ChatVo::User { uid, .. } => {
                        // TODO: to chat with user
                    }
                    ChatVo::Group { gid, .. } => {
                        // TODO: to chat with group
                    }
                }
            }
            None => {}
        }
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .chat_list
            .items
            .iter()
            .enumerate()
            .map(|(i, chat_vo)| {
                let color = alternate_colors(i);
                ListItem::new(Line::from(chat_vo)).bg(color)
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
        // We get the info depending on the item's state.
        let info = if let Some(i) = self.chat_list.state.selected() {
            Line::from(&self.chat_list.items[i])
        } else {
            Line::from("Nothing selected...".to_string())
        };

        // We show the list item's info under the list in this paragraph
        let block = Block::new()
            .title(Line::raw("TODO Info").centered())
            .borders(Borders::LEFT | Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG)
            .padding(Padding::horizontal(1));

        // We can now render the item info
        Paragraph::new(info)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .render(area, buf);
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
            let [list_area, chat_area] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]).areas(main_area);
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

impl From<&ChatVo> for Line<'_> {
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
                    Span::styled(format!("好友: {}\n", user_name), Style::default().fg(Color::LightBlue)),
                    Span::styled(format!("时间: {}\n", msg_time), Style::default().fg(Color::LightBlue)),
                    Span::styled(format!("{}\n", msg), Style::default().fg(Color::White)),
                ];
                if let Some(unread) = unread {
                    content.push(Span::styled(format!("未读: {}\n", unread), Style::default().fg(Color::LightBlue)))
                }
                Line::from(content)
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
                    Span::styled(format!("群: {}\n", group_name), Style::default().fg(Color::LightBlue)),
                    Span::styled(format!("时间: {}\n", msg_time), Style::default().fg(Color::LightBlue)),
                    Span::styled(format!("{}: {}\n", user_name, msg), Style::default().fg(Color::White)),
                ];
                if let Some(unread) = unread {
                    content.push(Span::styled(format!("未读: {}\n", unread), Style::default().fg(Color::LightBlue)))
                }
                Line::from(content)
            }
        }
    }
}

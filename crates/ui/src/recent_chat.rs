use crate::datetime::datetime_format;
use crate::token::CURRENT_USER;
use crate::{centered_rect, ui, HOST};
use chrono::{DateTime, Local};
use color_eyre::eyre::format_err;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Clone)]
enum Menu {
    RecentChat,
    Contacts,
    Me,
}

pub struct RecentChat {
    selected_menu: Menu,
    error_message: Option<String>, // 添加错误消息字段
}

impl RecentChat {
    pub(crate) fn new() -> Self {
        Self { selected_menu: Menu::RecentChat, error_message: None }
    }
    // TODO 最近聊天页面
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let result = recent_chat()?;
        loop {
            terminal.draw(|f| self.draw(f, &result))?;
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match self.selected_menu {
                    Menu::RecentChat => match key.code {
                        KeyCode::Char('q') => {
                            let mut mutex_guard = CURRENT_USER.lock().unwrap();
                            mutex_guard.user = None;
                            mutex_guard.token = None;
                            return Ok(());
                        }
                        KeyCode::Right => {
                            self.selected_menu = Menu::Contacts;
                        }
                        _ => {}
                    },
                    Menu::Contacts => match key.code {
                        KeyCode::Char('q') => {
                            let mut mutex_guard = CURRENT_USER.lock().unwrap();
                            mutex_guard.user = None;
                            mutex_guard.token = None;
                            return Ok(());
                        }
                        KeyCode::Left => {
                            self.selected_menu = Menu::RecentChat;
                        }
                        KeyCode::Right => {
                            self.selected_menu = Menu::Me;
                        }
                        _ => {}
                    }
                    Menu::Me => match key.code {
                        KeyCode::Char('q') => {
                            let mut mutex_guard = CURRENT_USER.lock().unwrap();
                            mutex_guard.user = None;
                            mutex_guard.token = None;
                            return Ok(());
                        }
                        KeyCode::Left => {
                            self.selected_menu = Menu::Contacts;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame, chatVos: &Vec<ChatVo>) {
        let block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));
        let area = ui::total_area(frame);
        frame.render_widget(block, area);

        let [chat_list_area, manu_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(3),
            ])
            .areas(area);
        // TODO 聊天列表
        const SIZE: usize = chat_list_area.height / 4;
        let constraints = vec![Constraint::Length(4); SIZE as usize];
        let list_area: [Rect; SIZE] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .areas(area);
        list_area.iter()
            .enumerate()
            .for_each(|(i, area)| {
                list_render(frame, area, chatVos, i)
            });
        self.menu_render(frame, manu_area);
    }

    fn menu_render(&mut self, frame: &mut Frame, manu_area: Rect) {
        let manu_border = Block::default().borders(Borders::NONE).style(Style::default().bg(Color::Gray));
        frame.render_widget(manu_border, manu_area);

        let [recent_chat_area, contacts_area, me_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 3); 3])
            .areas(manu_area);

        let recent_chat_text = Paragraph::new("RecentChat")
            .style(self.chose_manu_paragraph(Menu::RecentChat))
            .block(self.chose_manu_block(Menu::RecentChat))
            .centered();
        let contacts_text = Paragraph::new("Contacts")
            .style(self.chose_manu_paragraph(Menu::Contacts))
            .block(self.chose_manu_block(Menu::Contacts))
            .centered();
        let me_text = Paragraph::new("Me")
            .style(self.chose_manu_paragraph(Menu::Me))
            .block(self.chose_manu_block(Menu::Me))
            .centered();

        frame.render_widget(recent_chat_text, centered_rect(80, 90, recent_chat_area));
        frame.render_widget(contacts_text, centered_rect(80, 90, contacts_area));
        frame.render_widget(me_text, centered_rect(80, 90, me_area));
    }

    fn chose_manu_paragraph(&self, current_menu: Menu) -> Style {
        if self.selected_menu == current_menu {
            Style::default().fg(Color::Black).add_modifier(Modifier::BOLD | Modifier::ITALIC)
        } else {
            Style::default().fg(Color::White)
        }
    }
    fn chose_manu_block(&self, current_menu: Menu) -> Block {
        if self.selected_menu == current_menu {
            Block::new().borders(Borders::ALL).style(Style::default().fg(Color::LightGreen).bg(Color::LightGreen))
        } else {
            Block::new().borders(Borders::ALL).style(Style::default().fg(Color::Gray).bg(Color::Gray))
        }
    }
}

fn list_render(frame: &mut Frame, rect: &Rect, chat_vos: &Vec<ChatVo>, i: usize) {
    let [name_area, msg_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 2); 2])
        .areas(*rect);
    let (name, msg, msg_time) = match &chat_vos[i] {
        ChatVo::User { user_name, msg, msg_time, .. } => (user_name.to_string(), msg.to_string(), msg_time),
        ChatVo::Group { group_name, msg, msg_time, .. } => (group_name.to_string(), msg.to_string(), msg_time),
    };
    let name = Paragraph::new(name).style(Style::default().fg(Color::White));
    frame.render_widget(name, name_area);

    let [msg_area, date_time_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Length(10)])
        .areas(msg_area);

    let msg = Paragraph::new(msg).style(Style::default().fg(Color::White));
    frame.render_widget(msg, msg_area);

    let date_time = Paragraph::new(msg_time.format("%Y-%m-%d %H:%M:%S").to_string()).style(Style::default().fg(Color::White));
    frame.render_widget(date_time, date_time_area)
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
use crate::recent_chat::RecentChat;
use crate::token::CURRENT_USER;
use crate::user_input::Input;
use crate::{centered_rect, token};
use crate::{ui, HOST};
use color_eyre::eyre::format_err;
use color_eyre::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::prelude::{Color, Line, Modifier, Style, Stylize, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub struct Login {
    username: Input,
    password: Input,
    current_mode: CurrentMode,
    currently_editing: Option<CurrentlyEditing>,
    error_message: Option<String>, // 添加错误消息字段
}

enum CurrentMode {
    Normal,
    Editing,
    Alerting,
}

impl Login {
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match self.current_mode {
                    CurrentMode::Normal => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Enter => {
                            match login(&self) {
                                Ok(_) => {
                                    // 登陆后进入最近聊天页面
                                    // TODO home页面修改为tab 支持切换manu
                                    let mut recent_chat = RecentChat::new()?;
                                    recent_chat.run(&mut terminal)?;
                                }
                                Err(err) => {
                                    self.error_message = Some(err.to_string());
                                    self.current_mode = CurrentMode::Alerting;
                                }
                            }
                        }
                        KeyCode::Char('e') => {
                            self.current_mode = CurrentMode::Editing;
                            self.currently_editing = Some(CurrentlyEditing::Username);
                        }
                        _ => {}
                    }
                    CurrentMode::Editing => {
                        match self.currently_editing {
                            None => {}
                            Some(CurrentlyEditing::Username) => {
                                match key.code {
                                    KeyCode::Char(to_insert) => self.username.enter_char(to_insert),
                                    KeyCode::Backspace => self.username.delete_char(),
                                    KeyCode::Left => self.username.move_cursor_left(),
                                    KeyCode::Right => self.username.move_cursor_right(),
                                    KeyCode::Esc => self.current_mode = CurrentMode::Normal,
                                    KeyCode::Tab => {
                                        self.toggle_editing();
                                    }
                                    _ => {}
                                }
                            }
                            Some(CurrentlyEditing::Password) => {
                                match key.code {
                                    KeyCode::Char(to_insert) => self.password.enter_char(to_insert),
                                    KeyCode::Backspace => self.password.delete_char(),
                                    KeyCode::Left => self.password.move_cursor_left(),
                                    KeyCode::Right => self.password.move_cursor_right(),
                                    KeyCode::Esc | KeyCode::Enter => self.current_mode = CurrentMode::Normal,
                                    _ => {}
                                }
                            }
                        }
                    }
                    CurrentMode::Alerting => {
                        match key.code {
                            KeyCode::Esc => {
                                self.error_message = None;
                                self.current_mode = CurrentMode::Normal;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn new() -> Self {
        Self {
            username: Input::new(),
            password: Input::new(),
            current_mode: CurrentMode::Normal,
            currently_editing: None,
            error_message: None, // 初始化错误消息
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let bg_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Green));

        let area = ui::total_area(frame);
        frame.render_widget(bg_block, area);

        let [cli_name_area, help_area, mut user_name_area, mut password_area, button_area, _] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Max(2),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .areas(area);

        let banner = r#"
 ░▒▓██████▓▒░░▒▓█▓▒░░▒▓█▓▒░░▒▓██████▓▒░▒▓████████▓▒░▒▓██████▓▒░░▒▓█▓▒░      ░▒▓█▓▒░
░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░  ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░
░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░  ░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░
░▒▓█▓▒░      ░▒▓████████▓▒░▒▓████████▓▒░ ░▒▓█▓▒░  ░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░
░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░  ░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░
░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░  ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░
 ░▒▓██████▓▒░░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░   ░▒▓██████▓▒░░▒▓████████▓▒░▒▓█▓▒░
        "#;

        let cli_name = Paragraph::new(banner)
            .block(Block::default().borders(Borders::NONE))
            .centered();

        frame.render_widget(cli_name, centered_rect(100, 50, cli_name_area));

        let (msg, style) = match self.current_mode {
            CurrentMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing, ".bold(),
                    "Enter".bold(),
                    " to Login.".bold(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            CurrentMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to move to next. ".into(),
                ],
                Style::default(),
            ),
            CurrentMode::Alerting => (
                vec!["Press ".into(),
                     "Esc".bold(),
                     " to stop editing, ".into()],
                Style::default()
            ),
        };

        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text)
            .wrap(ratatui::widgets::Wrap { trim: true }); // 添加自动换行
        frame.render_widget(help_message, centered_rect(70, 100, help_area));

        let user_name = Paragraph::new(self.username.input.as_str())
            .style(match self.current_mode {
                CurrentMode::Editing if self.currently_editing == Some(CurrentlyEditing::Username) => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .block(Block::bordered().title("Username"));
        user_name_area = centered_rect(70, 100, user_name_area);
        frame.render_widget(user_name, user_name_area);

        let password = Paragraph::new(self.password.input.as_str())
            .style(match self.current_mode {
                CurrentMode::Editing if self.currently_editing == Some(CurrentlyEditing::Password) => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .block(Block::bordered().title("Password"));
        password_area = centered_rect(70, 100, password_area);
        frame.render_widget(password, password_area);

        match self.current_mode {
            CurrentMode::Normal => {}
            CurrentMode::Editing => {
                match self.currently_editing {
                    None => {}
                    Some(CurrentlyEditing::Username) => {
                        set_cursor(frame, user_name_area, &self.username)
                    }
                    Some(CurrentlyEditing::Password) => {
                        set_cursor(frame, password_area, &self.password)
                    }
                }
            }
            CurrentMode::Alerting => {}
        }

        let login = Paragraph::new(Text::styled("Login", Style::default()))
            .block(Block::default().borders(Borders::ALL))
            .centered();
        frame.render_widget(login, centered_rect(50, 100, button_area));

        let error_area = Rect::new(area.width * 2 / 10, (area.height - 2) / 2, area.width * 6 / 10, 3); // 新增代码
        // let error_area = centered_rect(60, 25, area);
        // 绘制错误消息
        if let Some(message) = &self.error_message {
            let error_paragraph = Paragraph::new(message.as_str())
                .style(Style::default().fg(Color::Red))
                .block(Block::default().title("Error | Esc to close this msg").borders(Borders::ALL))
                .wrap(Wrap { trim: true });
            frame.render_widget(error_paragraph, error_area); // 选择合适的区域
            self.current_mode = CurrentMode::Alerting;
        }
    }

    pub fn toggle_editing(&mut self) {
        if let Some(edit_mode) = &self.currently_editing {
            match edit_mode {
                CurrentlyEditing::Username => self.currently_editing = Some(CurrentlyEditing::Password),
                CurrentlyEditing::Password => self.currently_editing = Some(CurrentlyEditing::Username),
            };
        } else {
            self.currently_editing = Some(CurrentlyEditing::Username);
        }
    }
}

fn login(login: &Login) -> Result<()> {
    match do_login(login) {
        Ok(token) => {
            let user = token::parse_token(token.as_str()).unwrap().claims;
            {
                let mut guard = CURRENT_USER.lock().unwrap();
                guard.user = Some(user); // Create a longer-lived binding
                guard.token = Option::from(token); // Create a longer-lived binding
            }
            renew();
            Ok(())
        }
        Err(err) => {
            Err(err)
        }
    }
}

fn do_login(login: &Login) -> Result<String> {
    let login_url = format!("{HOST}/token/login");
    let client = Client::new();
    let response = client
        .post(&login_url)
        .json(&serde_json::json!({
            "name": login.username.input,
            "password": login.password.input
        }))
        .send();
    match response {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<LoginRes>() {
                    Ok(LoginRes { access_token }) => {
                        Ok(access_token)
                    }
                    Err(e) => {
                        Err(format_err!("Failed to parse response: {}", e))
                    }
                }
            } else if res.status() == StatusCode::UNAUTHORIZED {
                Err(format_err!("Login failed: 用户名或密码错误"))
            } else {
                Err(format_err!("Login failed: HTTP {}", res.status()))
            }
        }
        Err(e) => {
            Err(format_err!("Failed to send login request: {}", e))
        }
    }
}

fn renew() {
    // 启动异步线程，定时刷新token过期时间
    thread::spawn(move || {
        loop {
            let token = match CURRENT_USER.lock().unwrap().token.clone() {
                None => { break; }
                Some(token) => token
            };
            let token = format!("Bearer {token}");
            let renew_token_period = Duration::from_secs(60);
            sleep(renew_token_period);
            let renew_url = format!("{HOST}/token/renew");
            let client = Client::new();
            let response = client
                .patch(renew_url)
                .header("Authorization", token.clone())
                .send();
            match response {
                Ok(res) => {
                    if res.status().is_success() {
                        match res.text() {
                            Ok(t) => {
                                let token_data = token::parse_token(t.as_str()).unwrap();
                                let mut guard = CURRENT_USER.lock().unwrap();
                                guard.user = Some(token_data.claims);
                                guard.token = Some(t);
                            }
                            Err(e) => {
                                eprintln!("Failed to parse response: {}", e);
                            }
                        }
                    } else {
                        eprintln!("Token refresh failed: HTTP {}", res.status());
                    }
                }
                Err(err) => {
                    eprintln!("Failed to send token refresh request: {}", err);
                }
            }
        }
    });
}

#[derive(Deserialize)]
pub(crate) struct LoginRes {
    pub access_token: String,
}

#[derive(Eq, PartialEq)]
enum CurrentlyEditing {
    Username,
    Password,
}

fn set_cursor(frame: &mut Frame, area: Rect, input: &Input) {
    frame.set_cursor_position(Position::new(
        area.x + input.character_index as u16 + 1,
        area.y + 1,
    ))
}

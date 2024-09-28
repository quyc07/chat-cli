use crate::contacts::Contacts;
use crate::me::Me;
use crate::recent_chat::RecentChat;
use crate::token::CURRENT_USER;
use crate::{centered_rect, ui};
use color_eyre::owo_colors::OwoColorize;
use color_eyre::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame};

#[derive(Eq, PartialEq, Clone)]
enum Menu {
    RecentChat,
    Contacts,
    Me,
}

pub struct Home {
    selected_menu: Menu,
    error_message: Option<String>,
    current_mode: CurrentMode
}

pub(crate) enum CurrentMode {
    Normal,
    Alerting,
}

impl Home {
    pub(crate) fn new() -> Self {
        Self { selected_menu: Menu::RecentChat, error_message: None, current_mode: CurrentMode::Normal }
    }
    // TODO 最近聊天页面
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;
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

    fn draw(&mut self, frame: &mut Frame) {
        let block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));
        let area = ui::total_area(frame);
        frame.render_widget(block, area);

        let [content_area, manu_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(3),
            ])
            .areas(area);
        match self.selected_menu {
            Menu::RecentChat => {
                let res = RecentChat::new();
                match res {
                    Ok(mut chat) => {
                        frame.render_widget(&mut chat, content_area)
                    }
                    Err(err) => { self.error_message = Some(err.to_string()); }
                }
            }
            Menu::Contacts => {
                let mut contacts = Contacts {};
                frame.render_widget(&mut contacts, content_area);
            }
            Menu::Me => {
                let mut me = Me {};
                frame.render_widget(&mut me, content_area);
            }
        }
        self.menu_render(frame, manu_area);
        
        let error_area = Rect::new(area.width * 2 / 10, (area.height - 2) / 2, area.width * 6 / 10, 3); // 新增代码
        // 绘制错误消息
        if let Some(message) = &self.error_message {
            let error_paragraph = Paragraph::new(message.as_str())
                .style(Style::default().fg(Color::Red))
                .block(Block::default().title("Error | Esc to close this msg").borders(Borders::ALL));
            frame.render_widget(error_paragraph, error_area); // 选择合适的区域
            self.current_mode = CurrentMode::Alerting;
        }
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


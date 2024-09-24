use crate::user_input::{Input, InputMode};
use crate::centered_rect;
use color_eyre::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Stylize, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame};

pub struct Login {
    username: Input,
    password: Input,
    input_mode: InputMode,
    currently_editing: Option<CurrentlyEditing>,
}


impl Login {
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Enter => {
                            return do_login(self);
                        }
                        KeyCode::Char('e') => {
                            self.input_mode = InputMode::Editing;
                            self.currently_editing = Some(CurrentlyEditing::Username);
                        }
                        _ => {}
                    }
                    InputMode::Editing => {
                        match self.currently_editing {
                            None => {}
                            Some(CurrentlyEditing::Username) => {
                                match key.code {
                                    KeyCode::Char(to_insert) => self.username.enter_char(to_insert),
                                    KeyCode::Backspace => self.username.delete_char(),
                                    KeyCode::Left => self.username.move_cursor_left(),
                                    KeyCode::Right => self.username.move_cursor_right(),
                                    KeyCode::Esc => self.input_mode = InputMode::Normal,
                                    KeyCode::Enter => {
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
                                    KeyCode::Esc | KeyCode::Enter => self.input_mode = InputMode::Normal,
                                    _ => {}
                                }
                            }
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
            input_mode: InputMode::Normal,
            currently_editing: None,
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let bg_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Green));

        let area = Rect::new(0, 0, frame.area().width * 6 / 10, frame.area().height);
        frame.render_widget(bg_block, area);

        let [cli_name_area, help_area, user_name_area, password_area, button_area, _] = Layout::default()
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

        let banner = vec![
            "   ____   _           _       ".to_string(),
            "  / ___| | |__   __ _| |__".to_string(),
            " | |     | '_ \\ / _` | __/".to_string(),
            " | |___  | | | | (_| | |_".to_string(),
            "  \\____| |_| |_|\\__,_|___\\".to_string(),
        ];

        let cli_name = Paragraph::new(banner.iter().map(|line| Line::from(Span::styled(line, Style::default().fg(Color::LightCyan)))).collect::<Vec<_>>())
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(cli_name, centered_rect(50, 50, cli_name_area));

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
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
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to move to next. ".into(),
                ],
                Style::default(),
            ),
        };

        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text)
            .wrap(ratatui::widgets::Wrap { trim: true }); // 添加自动换行
        frame.render_widget(help_message, centered_rect(70,100,help_area));

        let user_name = Paragraph::new(self.username.input.as_str())
            .style(match self.input_mode {
                InputMode::Editing if self.currently_editing == Some(CurrentlyEditing::Username) => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .block(Block::bordered().title("Username"));
        frame.render_widget(user_name, centered_rect(70, 100, user_name_area));

        let password = Paragraph::new(self.password.input.as_str())
            .style(match self.input_mode {
                InputMode::Editing if self.currently_editing == Some(CurrentlyEditing::Password) => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .block(Block::bordered().title("Password"));
        frame.render_widget(password, centered_rect(70, 100, password_area));

        match self.input_mode {
            InputMode::Normal => {}
            InputMode::Editing => {
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
        }

        let login = Paragraph::new(Text::styled("Login", Style::default()))
            .block(Block::default().borders(Borders::ALL))
            .centered();
        frame.render_widget(login, centered_rect(50, 100, button_area));
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

fn do_login(login: &Login) -> Result<()> {
    Ok(())
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

use crate::{centered_rect, sub_rect};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub(crate) fn login(frame: &mut Frame) {
    let bg_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));

    let area = Rect::new(0, 0, frame.area().width * 6 / 10, frame.area().height);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    let cli_name_block = Block::default().borders(Borders::NONE).style(Style::default());
    let cli_name = Paragraph::new(Text::styled(
        "Chat-Cli",
        Style::default().fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD | Modifier::ITALIC),
    ))
        .centered()
        .block(cli_name_block);
    frame.render_widget(bg_block, area);
    frame.render_widget(cli_name, centered_rect(50, 50, chunks[1]));

    let txt_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Gray));

    let user_name_rect = centered_rect(70, 100, chunks[2]);
    let password_rect = centered_rect(70, 100, chunks[3]);
    frame.render_widget(txt_block.clone(), user_name_rect);
    frame.render_widget(txt_block, password_rect);

    let user_name_prompt = Paragraph::new(Line::from(Span::styled("Username: ", Style::default().fg(Color::White))))
        .block(Block::default().borders(Borders::NONE))
        .centered();
    let password_prompt = Paragraph::new(Line::from(Span::styled("Password: ", Style::default().fg(Color::White))))
        .block(Block::default().borders(Borders::NONE))
        .centered();
    let user_name_prompt_rect = sub_rect(10, 30, 0, 100, user_name_rect);
    let password_prompt_rect = sub_rect(10, 30, 0, 100, password_rect);
    frame.render_widget(user_name_prompt, user_name_prompt_rect);
    frame.render_widget(password_prompt, password_prompt_rect);

    let login = Paragraph::new(Text::styled("Login", Style::default().fg(Color::White).bg(Color::Blue)))
        .block(Block::default().borders(Borders::ALL))
        .centered();
    let register = Paragraph::new(Text::styled("Register", Style::default().fg(Color::White).bg(Color::Blue)))
        .block(Block::default().borders(Borders::ALL))
        .centered();

    frame.render_widget(login, centered_rect(50, 100, chunks[4]));
}
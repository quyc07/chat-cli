use color_eyre::owo_colors::OwoColorize;
use color_eyre::{eyre::Context, Result};
use crossterm::event::KeyEventKind;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Span;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{
    crossterm::event::{self, Event, KeyCode}
    ,
    DefaultTerminal, Frame,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = run(terminal).context("app loop failed");
    ratatui::restore();
    app_result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(draw)?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                break Ok(());
            }
        }
    }
}

fn draw(frame: &mut Frame) {
    let bg_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let cli_name_block = Block::default().borders(Borders::NONE).style(Style::default());
    let cli_name = Paragraph::new(Text::styled(
        "Chat-Cli",
        Style::default().fg(Color::LightCyan),
    ))
        .centered()
        .block(cli_name_block);
    frame.render_widget(bg_block, frame.area());
    frame.render_widget(cli_name, chunks[0]);

    let txt_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Gray));

    let user_name_rect = centered_rect(70, 100, chunks[1]);
    let password_rect = centered_rect(70, 100, chunks[2]);
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

    let buttons_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[3]);
    frame.render_widget(login, sub_rect(16, 18, 10, 80, chunks[3]));
    frame.render_widget(register, sub_rect(66, 18, 10, 80, chunks[3]));
}

// ANCHOR: centered_rect
/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
// ANCHOR_END: centered_rect

fn sub_rect(x: u16, width: u16, y: u16, high: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(x),
            Constraint::Percentage(width),
            Constraint::Percentage(100 - x - width),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(y),
            Constraint::Percentage(high),
            Constraint::Percentage(100 - y - high),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
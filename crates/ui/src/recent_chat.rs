use color_eyre::owo_colors::OwoColorize;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame};

pub struct RecentChat {}

impl RecentChat {
    pub(crate) fn new() -> Self {
        Self {}
    }
    // TODO 最近聊天页面
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Gray));
        let paragraph = Paragraph::new("RecentChat")
            .style(Style::default())
            .block(block);
        frame.render_widget(paragraph, frame.area());
    }
}
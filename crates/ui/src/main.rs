mod login;
mod user_input;
mod token;

use crate::login::Login;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::{eyre::Context, Result};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[cfg(feature = "release")]
static HOST: &str = include_str!("../config/release");
#[cfg(not(feature = "release"))]
static HOST: &str = "http://localhost:3000";

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = Login::new().run(terminal).context("app loop failed");
    ratatui::restore();
    app_result
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
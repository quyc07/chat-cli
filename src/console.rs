use std::io::{stdout, Write};

use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};

pub(crate) fn clean_all() {
    execute!(stdout(), Clear(ClearType::All)).unwrap();
}
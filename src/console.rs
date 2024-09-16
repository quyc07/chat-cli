use std::io::{stdout, Write};

pub(crate) fn clean_all() {
    let mut stdout = stdout();
    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    ).unwrap();
    stdout.flush().unwrap();
}
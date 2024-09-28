use ratatui::layout::Rect;
use ratatui::Frame;

pub(crate) fn total_area(frame: &mut Frame) -> Rect {
    Rect::new(0, 0, frame.area().width * 6 / 10, frame.area().height)
}
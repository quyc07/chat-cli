use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

pub(crate) struct Contacts {}

impl Widget for &mut Contacts {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized
    {
        todo!()
    }
}
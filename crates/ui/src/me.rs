use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

pub(crate) struct Me {}

impl Widget for &mut Me{
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized
    {
        todo!()
    }
}
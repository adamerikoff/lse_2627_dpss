// Верхняя панель: роль сессии и подсказки клавиш. Видна всегда. Роль
// подсвечивается, чтобы эскалация до bank_admin читалась сразу.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;

pub fn draw(frame: &mut Frame, area: Rect, role: &str, hints: &str) {
    let shown = if role.is_empty() { "—" } else { role };
    let text = format!("neobank CBS    роль: {shown}    {hints}");

    let colour = if role == "bank_admin" {
        Color::Red
    } else {
        Color::Cyan
    };

    frame.render_widget(
        Paragraph::new(text).block(Block::bordered().border_style(Style::new().fg(colour))),
        area,
    );
}

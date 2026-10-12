// Панель журнала. Одна функция на обе панели (stdout и stderr) — различаются
// только заголовком, цветом рамки и источником строк.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;

pub fn draw(frame: &mut Frame, area: Rect, title: &str, colour: Color, lines: &[String]) {
    let block = Block::bordered()
        .title(format!(" {} ", title))
        .border_style(Style::new().fg(colour));

    let text = Paragraph::new(tail(lines, area.height))
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(text, area);
}

// Только последние строки, сколько влезает (минус две на рамку).
fn tail(lines: &[String], height: u16) -> String {
    let visible = height.saturating_sub(2) as usize;

    if lines.len() <= visible {
        return lines.join("\n");
    }

    lines[lines.len() - visible..].join("\n")
}

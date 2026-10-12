// Мастер «новый клиент» (6.3.2 + 6.4.2). Пошагово: ФИО → email → валюта →
// депозит. На последнем шаге выполняется онбординг одной транзакцией:
// клиент → счёт → карта + первая транзакция. Номера, даты и id — автоматически.

use crate::db::Db;
use crate::logging::Journal;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;

const STEPS: usize = 4;
const LABELS: [&str; STEPS] = ["ФИО", "email", "валюта (напр. USD)", "депозит"];

pub struct Form {
    values: [String; STEPS],
    step: usize,
}

impl Form {
    pub fn new() -> Form {
        Form {
            values: [String::new(), String::new(), String::new(), String::new()],
            step: 0,
        }
    }
}

pub fn handle(form: &mut Form, code: KeyCode, db: &mut Db, journal: &mut Journal) {
    match code {
        KeyCode::Char(character) => form.values[form.step].push(character),
        KeyCode::Backspace => {
            // Пустое поле + Backspace — шаг назад.
            if form.values[form.step].is_empty() && form.step > 0 {
                form.step -= 1;
            } else {
                form.values[form.step].pop();
            }
        }
        KeyCode::Enter => {
            if form.values[form.step].trim().is_empty() {
                journal.error(&format!("{} не заполнено", LABELS[form.step]));
                return;
            }
            if form.step + 1 < STEPS {
                form.step += 1;
            } else {
                submit(form, db, journal);
            }
        }
        _ => {}
    }
}

fn submit(form: &mut Form, db: &mut Db, journal: &mut Journal) {
    let result = db.onboard(
        form.values[0].trim(),
        form.values[1].trim(),
        form.values[2].trim(),
        form.values[3].trim(),
    );

    match result {
        Ok(account_number) => {
            journal.info(&format!(
                "новый клиент {}: счёт {} открыт",
                form.values[0].trim(),
                account_number
            ));
            *form = Form::new();
        }
        Err(error) => {
            journal.db_error("не удалось создать клиента, проверьте данные", &error);
        }
    }
}

pub fn draw(frame: &mut Frame, area: Rect, form: &Form) {
    let mut lines: Vec<Line> = Vec::with_capacity(STEPS);

    for (index, label) in LABELS.iter().enumerate() {
        let value = &form.values[index];
        let line = if index < form.step {
            // Уже заполнено.
            Line::styled(
                format!("✓ {}: {}", label, value),
                Style::new().fg(Color::DarkGray),
            )
        } else if index == form.step {
            // Текущий шаг.
            Line::styled(
                format!("▶ {}: {}▌", label, value),
                Style::new().fg(Color::White),
            )
        } else {
            // Ещё впереди.
            Line::styled(format!("  {}:", label), Style::new().fg(Color::DarkGray))
        };
        lines.push(line);
    }

    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" новый клиент ")),
        area,
    );
}

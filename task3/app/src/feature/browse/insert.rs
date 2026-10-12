// Режим вставки (6.3.1). Поле на колонку таблицы (кроме id); пустые поля
// опускаются — их значения БД подставляет по умолчанию. Инъекция — в
// значениях (имена колонок берутся из каталога).

use crate::db::Db;
use crate::db::commands::Table;
use crate::feature::browse::field_line;
use crate::logging::Journal;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;

pub struct Form {
    columns: Vec<String>,
    values: Vec<String>,
    focus: usize,
}

impl Form {
    pub fn new() -> Form {
        Form {
            columns: Vec::new(),
            values: Vec::new(),
            focus: 0,
        }
    }

    // Задать набор колонок (при входе в режим или смене таблицы).
    pub fn load(&mut self, columns: Vec<String>) {
        self.values = vec![String::new(); columns.len()];
        self.columns = columns;
        self.focus = 0;
    }

    fn active(&mut self) -> Option<&mut String> {
        self.values.get_mut(self.focus)
    }
}

pub fn handle(
    form: &mut Form,
    code: KeyCode,
    table: &str,
    db: &mut Db,
    journal: &mut Journal,
) -> Option<Table> {
    match code {
        KeyCode::Tab => {
            if !form.columns.is_empty() {
                form.focus = (form.focus + 1) % form.columns.len();
            }
            None
        }
        KeyCode::Backspace => {
            if let Some(value) = form.active() {
                value.pop();
            }
            None
        }
        KeyCode::Char(character) => {
            if let Some(value) = form.active() {
                value.push(character);
            }
            None
        }
        KeyCode::Enter => apply(form, table, db, journal),
        _ => None,
    }
}

fn apply(form: &Form, table: &str, db: &mut Db, journal: &mut Journal) -> Option<Table> {
    // Только заполненные колонки; пустые уходят на DEFAULT.
    let mut columns = Vec::new();
    let mut values = Vec::new();
    for index in 0..form.columns.len() {
        if !form.values[index].trim().is_empty() {
            columns.push(form.columns[index].clone());
            values.push(form.values[index].clone());
        }
    }

    if columns.is_empty() {
        journal.error("заполните хотя бы одно поле");
        return None;
    }

    match db.insert(table, &columns, &values) {
        Ok(rows) => {
            journal.info(&format!("{}: вставлено {} строк", table, rows));
            db.select_all(table).ok()
        }
        Err(error) => {
            journal.db_error("вставка не выполнена, проверьте данные", &error);
            None
        }
    }
}

pub fn draw(frame: &mut Frame, area: Rect, form: &Form) {
    if form.columns.is_empty() {
        frame.render_widget(
            Paragraph::new("нет колонок").block(Block::bordered().title(" вставка ")),
            area,
        );
        return;
    }

    let lines = form
        .columns
        .iter()
        .enumerate()
        .map(|(index, name)| {
            field_line(
                &format!("{}: ", name),
                &form.values[index],
                form.focus == index,
            )
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" вставка ")),
        area,
    );
}

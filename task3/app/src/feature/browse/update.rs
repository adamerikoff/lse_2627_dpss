// Режим обновления (6.2). Одно where-значение → одна строка (6.2.1);
// несколько через запятую → `IN` (6.2.2). После успеха таблица перечитывается.

use crate::db::Db;
use crate::db::commands::Table;
use crate::feature::browse::field_line;
use crate::logging::Journal;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;

#[derive(Clone, Copy, PartialEq)]
enum Field {
    SetColumn,
    SetValue,
    WhereColumn,
    WhereValue,
}

pub struct Form {
    set_column: String,
    set_value: String,
    where_column: String,
    where_value: String,
    field: Field,
}

impl Form {
    pub fn new() -> Form {
        Form {
            set_column: String::new(),
            set_value: String::new(),
            where_column: String::new(),
            where_value: String::new(),
            field: Field::SetColumn,
        }
    }

    fn active(&mut self) -> &mut String {
        match self.field {
            Field::SetColumn => &mut self.set_column,
            Field::SetValue => &mut self.set_value,
            Field::WhereColumn => &mut self.where_column,
            Field::WhereValue => &mut self.where_value,
        }
    }

    fn next_field(&mut self) {
        self.field = match self.field {
            Field::SetColumn => Field::SetValue,
            Field::SetValue => Field::WhereColumn,
            Field::WhereColumn => Field::WhereValue,
            Field::WhereValue => Field::SetColumn,
        };
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
            form.next_field();
            None
        }
        KeyCode::Backspace => {
            form.active().pop();
            None
        }
        KeyCode::Char(character) => {
            form.active().push(character);
            None
        }
        KeyCode::Enter => apply(form, table, db, journal),
        _ => None,
    }
}

fn apply(form: &Form, table: &str, db: &mut Db, journal: &mut Journal) -> Option<Table> {
    let set_column = form.set_column.trim();
    let where_column = form.where_column.trim();

    if set_column.is_empty() || where_column.is_empty() {
        journal.error("укажите set-колонку и where-колонку");
        return None;
    }

    let where_values = super::split_values(&form.where_value);

    let result = if where_values.len() > 1 {
        db.update_in(
            table,
            set_column,
            &form.set_value,
            where_column,
            &where_values,
        )
    } else {
        db.update(
            table,
            set_column,
            &form.set_value,
            where_column,
            &form.where_value,
        )
    };

    match result {
        Ok(rows) => {
            journal.info(&format!("{}: обновлено {} строк", table, rows));
            // Перечитываем, чтобы изменения были видны.
            db.select_all(table).ok()
        }
        Err(error) => {
            journal.db_error("обновление не выполнено, проверьте данные", &error);
            None
        }
    }
}

pub fn draw(frame: &mut Frame, area: Rect, form: &Form) {
    let lines = vec![
        field_line(
            "set колонка:   ",
            &form.set_column,
            form.field == Field::SetColumn,
        ),
        field_line(
            "set значение:  ",
            &form.set_value,
            form.field == Field::SetValue,
        ),
        field_line(
            "where колонка: ",
            &form.where_column,
            form.field == Field::WhereColumn,
        ),
        field_line(
            "where значение:",
            &form.where_value,
            form.field == Field::WhereValue,
        ),
    ];

    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" обновление ")),
        area,
    );
}

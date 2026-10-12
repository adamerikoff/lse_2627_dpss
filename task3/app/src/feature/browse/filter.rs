// Режим фильтра (6.1). Одно значение → `= $1` (6.1.2); несколько через
// запятую → `IN`/`ANY` (6.1.3); пустая колонка → все строки (6.1.1).

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
    Column,
    Value,
}

pub struct Form {
    column: String,
    value: String,
    field: Field,
}

impl Form {
    pub fn new() -> Form {
        Form {
            column: String::new(),
            value: String::new(),
            field: Field::Column,
        }
    }

    fn active(&mut self) -> &mut String {
        match self.field {
            Field::Column => &mut self.column,
            Field::Value => &mut self.value,
        }
    }
}

// Возвращает новый результат на Enter, иначе None (редактирование).
pub fn handle(
    form: &mut Form,
    code: KeyCode,
    table: &str,
    db: &mut Db,
    journal: &mut Journal,
) -> Option<Table> {
    match code {
        KeyCode::Tab => {
            form.field = match form.field {
                Field::Column => Field::Value,
                Field::Value => Field::Column,
            };
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
    let column = form.column.trim();

    if column.is_empty() {
        return match db.select_all(table) {
            Ok(result) => {
                journal.info(&format!("{}: {} строк", table, result.rows.len()));
                Some(result)
            }
            Err(error) => {
                journal.db_error("не удалось прочитать таблицу", &error);
                None
            }
        };
    }

    let values = super::split_values(&form.value);

    let result = if values.len() > 1 {
        db.filter_in(table, column, &values)
    } else {
        db.filter(table, column, &form.value)
    };

    match result {
        Ok(found) => {
            journal.info(&format!(
                "{} где {}: {} строк",
                table,
                column,
                found.rows.len()
            ));
            Some(found)
        }
        Err(error) => {
            journal.db_error("фильтр не выполнен, проверьте колонку и значение", &error);
            None
        }
    }
}

pub fn draw(frame: &mut Frame, area: Rect, form: &Form) {
    let lines = vec![
        field_line("колонка:  ", &form.column, form.field == Field::Column),
        field_line("значение: ", &form.value, form.field == Field::Value),
    ];

    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" фильтр ")),
        area,
    );
}

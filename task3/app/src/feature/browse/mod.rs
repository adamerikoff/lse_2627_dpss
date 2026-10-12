// Фича просмотра/правки. mod.rs держит общее: список таблиц, текущий результат,
// режим и маршрутизацию клавиш и отрисовки. Логика каждого режима — в своём
// файле (filter, update и далее), каждый само­достаточен.

mod filter;
mod insert;
mod onboard;
mod update;

use crate::db::Db;
use crate::db::commands::Table;
use crate::feature::Action;
use crate::logging::Journal;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::List;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table as TableWidget;

// Режим области ввода. Переключается ←/→.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Filter,
    Update,
    Insert,
    NewCustomer,
}

impl Mode {
    const ALL: [Mode; 4] = [Mode::Filter, Mode::Update, Mode::Insert, Mode::NewCustomer];

    fn label(self) -> &'static str {
        match self {
            Mode::Filter => "фильтр",
            Mode::Update => "обновление",
            Mode::Insert => "вставка",
            Mode::NewCustomer => "новый клиент",
        }
    }

    fn step(self, delta: isize) -> Mode {
        let index = Mode::ALL.iter().position(|m| *m == self).unwrap_or(0) as isize;
        let count = Mode::ALL.len() as isize;
        Mode::ALL[(index + delta).rem_euclid(count) as usize]
    }
}

pub struct State {
    tables: Vec<String>,
    selected: usize,
    table: Option<Table>,
    mode: Mode,
    filter: filter::Form,
    update: update::Form,
    insert: insert::Form,
    onboard: onboard::Form,
}

impl State {
    pub fn new(db: &mut Db, journal: &mut Journal) -> Result<State, String> {
        let tables = db.tables()?;
        let mut state = State {
            tables,
            selected: 0,
            table: None,
            mode: Mode::Filter,
            filter: filter::Form::new(),
            update: update::Form::new(),
            insert: insert::Form::new(),
            onboard: onboard::Form::new(),
        };
        state.show_all(db, journal);
        Ok(state)
    }

    fn selected_name(&self) -> String {
        match self.tables.get(self.selected) {
            Some(name) => name.clone(),
            None => String::new(),
        }
    }

    // Переключение таблицы сбрасывает формы и показывает все строки.
    fn choose(&mut self, selected: usize, db: &mut Db, journal: &mut Journal) {
        self.selected = selected;
        self.filter = filter::Form::new();
        self.update = update::Form::new();
        self.show_all(db, journal);
        if self.mode == Mode::Insert {
            self.load_insert(db, journal);
        }
    }

    // Загрузить в форму вставки только заполняемые колонки текущей таблицы.
    fn load_insert(&mut self, db: &mut Db, journal: &mut Journal) {
        let name = self.selected_name();
        match db.insertable_columns(&name) {
            Ok(columns) => self.insert.load(columns),
            Err(error) => journal.db_error("не удалось получить колонки таблицы", &error),
        }
    }

    fn show_all(&mut self, db: &mut Db, journal: &mut Journal) {
        let name = self.selected_name();
        if name.is_empty() {
            return;
        }

        match db.select_all(&name) {
            Ok(table) => {
                journal.info(&format!("{}: {} строк", name, table.rows.len()));
                self.table = Some(table);
            }
            Err(error) => {
                journal.db_error("не удалось прочитать таблицу", &error);
                self.table = None;
            }
        }
    }
}

// Подсказки с вкладками режимов; активный в скобках.
pub fn hints(state: &State) -> String {
    let tabs: Vec<String> = Mode::ALL
        .iter()
        .map(|mode| {
            if *mode == state.mode {
                format!("[{}]", mode.label())
            } else {
                mode.label().to_string()
            }
        })
        .collect();

    format!("←→ {}   ↑↓ таблица   Esc выход", tabs.join(" "))
}

pub fn handle_key(state: &mut State, key: KeyEvent, db: &mut Db, journal: &mut Journal) -> Action {
    // Общие клавиши: режим и выбор таблицы.
    match key.code {
        KeyCode::Left => {
            state.mode = state.mode.step(-1);
            if state.mode == Mode::Insert {
                state.load_insert(db, journal);
            }
            return Action::None;
        }
        KeyCode::Right => {
            state.mode = state.mode.step(1);
            if state.mode == Mode::Insert {
                state.load_insert(db, journal);
            }
            return Action::None;
        }
        KeyCode::Up => {
            if state.selected > 0 {
                state.choose(state.selected - 1, db, journal);
            }
            return Action::None;
        }
        KeyCode::Down => {
            if state.selected + 1 < state.tables.len() {
                state.choose(state.selected + 1, db, journal);
            }
            return Action::None;
        }
        _ => {}
    }

    // Остальное — активному режиму. Он возвращает новый результат либо None.
    let name = state.selected_name();
    let result = match state.mode {
        Mode::Filter => filter::handle(&mut state.filter, key.code, &name, db, journal),
        Mode::Update => update::handle(&mut state.update, key.code, &name, db, journal),
        Mode::Insert => insert::handle(&mut state.insert, key.code, &name, db, journal),
        Mode::NewCustomer => {
            // Мастер не меняет показанную таблицу, работает со своими.
            onboard::handle(&mut state.onboard, key.code, db, journal);
            None
        }
    };

    if let Some(table) = result {
        state.table = Some(table);
    }

    Action::None
}

pub fn view(frame: &mut Frame, main: Rect, input: Rect, state: &State) {
    let split = Layout::horizontal([Constraint::Length(20), Constraint::Min(20)]).split(main);
    draw_tables(frame, split[0], state);
    draw_rows(frame, split[1], state);

    match state.mode {
        Mode::Filter => filter::draw(frame, input, &state.filter),
        Mode::Update => update::draw(frame, input, &state.update),
        Mode::Insert => insert::draw(frame, input, &state.insert),
        Mode::NewCustomer => onboard::draw(frame, input, &state.onboard),
    }
}

fn draw_tables(frame: &mut Frame, area: Rect, state: &State) {
    let list = List::new(state.tables.clone())
        .block(Block::bordered().title(" таблицы "))
        .highlight_symbol("▶ ")
        .highlight_style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected));

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn draw_rows(frame: &mut Frame, area: Rect, state: &State) {
    let table = match &state.table {
        Some(table) => table,
        None => {
            frame.render_widget(Paragraph::new("нет данных").block(Block::bordered()), area);
            return;
        }
    };

    let title = format!(" {} — {} строк ", state.selected_name(), table.rows.len());

    let columns = table.headers.len().max(1);
    let widths: Vec<Constraint> = (0..columns)
        .map(|_| Constraint::Ratio(1, columns as u32))
        .collect();

    let header = Row::new(table.headers.clone()).style(Style::new().add_modifier(Modifier::BOLD));
    let rows: Vec<Row> = table
        .rows
        .iter()
        .map(|cells| Row::new(cells.clone()))
        .collect();

    let widget = TableWidget::new(rows, widths)
        .header(header)
        .block(Block::bordered().title(title));

    frame.render_widget(widget, area);
}

// Значения через запятую → список непустых. Одно значение → список из одного.
pub(super) fn split_values(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

// Общая отрисовка строки-поля: метка, значение, курсор у активного.
pub(super) fn field_line(label: &str, value: &str, active: bool) -> Line<'static> {
    let cursor = if active { "▌" } else { "" };
    let text = format!("{}{}{}", label, value, cursor);

    let style = if active {
        Style::new().fg(Color::White)
    } else {
        Style::new().fg(Color::DarkGray)
    };

    Line::styled(text, style)
}

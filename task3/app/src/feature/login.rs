// Фича входа: форма с двумя полями и подключение при submit. Значения отсюда
// идут в строку подключения — точка connection-string инъекции в уязвимой
// версии.

use crate::db::Db;
use crate::feature::Action;
use crate::feature::Screen;
use crate::feature::browse;
use crate::logging::Journal;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;

pub const HINTS: &str = "Tab — поле   Enter — вход   Esc — выход";

#[derive(Clone, Copy, PartialEq)]
pub enum Field {
    Login,
    Password,
}

pub struct State {
    pub login: String,
    pub password: String,
    pub field: Field,
}

impl State {
    pub fn new() -> State {
        State {
            login: String::new(),
            password: String::new(),
            field: Field::Login,
        }
    }

    fn active(&mut self) -> &mut String {
        match self.field {
            Field::Login => &mut self.login,
            Field::Password => &mut self.password,
        }
    }
}

pub fn handle_key(state: &mut State, key: KeyEvent, db: &mut Db, journal: &mut Journal) -> Action {
    match key.code {
        KeyCode::Tab => {
            state.field = match state.field {
                Field::Login => Field::Password,
                Field::Password => Field::Login,
            };
            Action::None
        }
        KeyCode::Backspace => {
            state.active().pop();
            Action::None
        }
        KeyCode::Char(character) => {
            state.active().push(character);
            Action::None
        }
        KeyCode::Enter => submit(state, db, journal),
        _ => Action::None,
    }
}

fn submit(state: &mut State, db: &mut Db, journal: &mut Journal) -> Action {
    if state.login.trim().is_empty() {
        journal.error("логин пуст");
        return Action::None;
    }

    journal.info(&format!("подключение, логин: {}", state.login));

    if let Err(error) = db.connect(&state.login, &state.password) {
        journal.db_error("вход не выполнен, проверьте логин и пароль", &error);
        return Action::None;
    }

    journal.info(&format!("роль сессии: {}", db.role));

    match browse::State::new(db, journal) {
        Ok(browse) => Action::Enter(Screen::Browse(Box::new(browse))),
        Err(error) => {
            journal.error(&error);
            Action::None
        }
    }
}

pub fn view(frame: &mut Frame, main: Rect, input: Rect, state: &State, secure: bool) {
    welcome(frame, main, secure);
    form(frame, input, state);
}

fn welcome(frame: &mut Frame, area: Rect, secure: bool) {
    let mode = if secure {
        "защищённый"
    } else {
        "уязвимый"
    };

    let text = format!(
        "neobank — клиент PostgreSQL (CBS)\n\
         режим: {mode}\n\n\
         Введите логин и пароль роли БД, Enter — вход.\n\n\
         Демо-роль сотрудника:  bank_employee / employee_pass"
    );

    frame.render_widget(
        Paragraph::new(text)
            .block(Block::bordered().title(" задание 3 "))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn form(frame: &mut Frame, area: Rect, state: &State) {
    let login = field_line("логин:  ", &state.login, state.field == Field::Login);

    let dots = "*".repeat(state.password.chars().count());
    let password = field_line("пароль: ", &dots, state.field == Field::Password);

    let border = Style::new().fg(Color::Cyan);
    frame.render_widget(
        Paragraph::new(vec![login, password])
            .block(Block::bordered().title(" вход ").border_style(border)),
        area,
    );
}

fn field_line(label: &str, value: &str, active: bool) -> Line<'static> {
    let cursor = if active { "▌" } else { "" };
    let text = format!("{}{}{}", label, value, cursor);

    let style = if active {
        Style::new().fg(Color::White)
    } else {
        Style::new().fg(Color::DarkGray)
    };

    Line::styled(text, style)
}

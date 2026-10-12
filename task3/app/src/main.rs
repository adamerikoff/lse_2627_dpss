mod config;
mod db;
mod feature;
mod logging;
mod ui;

use crate::db::Db;
use crate::feature::Action;
use crate::feature::Screen;
use crate::feature::browse;
use crate::feature::login;
use crate::logging::Journal;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::style::Color;

fn main() {
    // Единственный аргумент: --secure включает защищённую версию, иначе уязвимая.
    let secure = std::env::args().any(|arg| arg == "--secure");

    // init переводит терминал в raw mode и на альтернативный экран; restore
    // возвращает всё обратно и должен вызваться при любом исходе.
    let mut terminal = ratatui::init();
    let result = App::new(secure).run(&mut terminal);
    ratatui::restore();

    if let Err(error) = result {
        eprintln!("ошибка: {}", error);
    }
}

// Оркестрация минимальна: общая сессия плюс активная фича. Цикл раздаёт клавиши
// активной фиче и собирает кадр из обвязки и её view; логика экранов — в
// feature/*, не здесь.
struct App {
    db: Db,
    journal: Journal,
    screen: Screen,
}

impl App {
    fn new(secure: bool) -> App {
        let config = config::load();

        let mut journal = Journal::new(&config.log_file);
        journal.info(if secure {
            "режим: защищённый (--secure)"
        } else {
            "режим: уязвимый"
        });

        App {
            db: Db::new(secure, config.db_host, config.db_port, config.db_name),
            journal,
            screen: Screen::Login(login::State::new()),
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if key.code == KeyCode::Esc {
                    break;
                }
                self.dispatch(key);
            }
        }

        Ok(())
    }

    // Клавиша уходит активной фиче; её Action применяется здесь.
    fn dispatch(&mut self, key: KeyEvent) {
        let action = match &mut self.screen {
            Screen::Login(state) => login::handle_key(state, key, &mut self.db, &mut self.journal),
            Screen::Browse(state) => {
                browse::handle_key(state, key, &mut self.db, &mut self.journal)
            }
        };

        if let Action::Enter(screen) = action {
            self.screen = screen;
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let area = Layout::vertical([
            Constraint::Length(3), // верхняя панель
            Constraint::Min(5),    // главная область
            Constraint::Length(8), // область ввода (до 6 полей формы + рамка)
            Constraint::Length(6), // панели логов
        ])
        .split(frame.area());

        let hints = match &self.screen {
            Screen::Login(_) => login::HINTS.to_string(),
            Screen::Browse(state) => browse::hints(state),
        };
        ui::topbar::draw(frame, area[0], &self.db.role, &hints);

        match &self.screen {
            Screen::Login(state) => login::view(frame, area[1], area[2], state, self.db.secure()),
            Screen::Browse(state) => browse::view(frame, area[1], area[2], state),
        }

        let logs = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area[3]);
        ui::logs::draw(frame, logs[0], "stdout", Color::Green, &self.journal.out);
        ui::logs::draw(frame, logs[1], "stderr", Color::Red, &self.journal.err);
    }
}

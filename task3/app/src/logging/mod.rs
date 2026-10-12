// Журнал событий. Пишет в три места одновременно:
//   info()  -> панель stdout + файл
//   error() -> панель stderr + файл
// Файл logs/system.log получает все уровни подряд (п. 6.7).

use chrono::Local;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

const MAX_LINES: usize = 200;

pub struct Journal {
    pub out: Vec<String>,
    pub err: Vec<String>,
    file: Option<File>,
}

impl Journal {
    pub fn new(path: &str) -> Journal {
        let file = open_log(path);

        let mut journal = Journal {
            out: Vec::new(),
            err: Vec::new(),
            file,
        };

        if journal.file.is_none() {
            journal
                .err
                .push(line("ERROR", &format!("не открыть {}", path)));
        }

        journal
    }

    pub fn info(&mut self, message: &str) {
        let entry = line("INFO", message);
        push(&mut self.out, entry.clone());
        self.write(&entry);
    }

    // Простая ошибка (валидация ввода) — текст уже дружественный, одинаков
    // в панели и в файле.
    pub fn error(&mut self, message: &str) {
        let entry = line("ERROR", message);
        push(&mut self.err, entry.clone());
        self.write(&entry);
    }

    // Ошибка от БД/драйвера (6.5): пользователю — дружественный текст без
    // деталей реализации; в файл (6.7) — он же плюс сырое сообщение драйвера.
    pub fn db_error(&mut self, friendly: &str, detail: &str) {
        push(&mut self.err, line("ERROR", friendly));
        self.write(&line("ERROR", &format!("{} :: {}", friendly, detail)));
    }

    fn write(&mut self, entry: &str) {
        if let Some(file) = self.file.as_mut() {
            let _ = writeln!(file, "{}", entry);
        }
    }
}

fn line(level: &str, message: &str) -> String {
    let stamp = Local::now().format("%H:%M:%S");
    format!("{} {:5} {}", stamp, level, message)
}

fn push(lines: &mut Vec<String>, entry: String) {
    lines.push(entry);
    while lines.len() > MAX_LINES {
        lines.remove(0);
    }
}

fn open_log(path: &str) -> Option<File> {
    // Создать каталог лога, если он задан и ещё не существует.
    if let Some(parent) = Path::new(path).parent()
        && !parent.as_os_str().is_empty()
        && fs::create_dir_all(parent).is_err()
    {
        return None;
    }

    OpenOptions::new().create(true).append(true).open(path).ok()
}

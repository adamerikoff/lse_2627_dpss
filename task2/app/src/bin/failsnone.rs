// Все четыре защиты. Не зависает ни на одном сценарии.
//
// Отличие от fails4: проба выполняется в отдельном потоке, а цикл ждёт её
// результат с дедлайном. Это единственное, что спасает в сценарии 4, где
// ядро сервера отвечает на пробы, а сам сервер заморожен: сеть исправна,
// keepalive доволен, statement_timeout заморожен вместе с backend'ом, и
// решение может принять только клиент.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

fn main() {
    let settings = app::settings();
    app::setup_logging(&settings.log_file);

    let mut pg = postgres::Config::new();
    pg.host(&settings.db_host);
    pg.port(settings.db_port);
    pg.dbname(&settings.db_name);
    pg.user(&settings.db_user);
    pg.password(&settings.db_password);
    pg.application_name("task2-failsnone");

    pg.connect_timeout(Duration::from_secs(3));
    pg.options("-c statement_timeout=5000");

    let idle = Duration::from_secs(10);
    let probe_interval = Duration::from_secs(5);
    let retries = 3;

    pg.keepalives(true);
    pg.keepalives_idle(idle);
    pg.keepalives_interval(probe_interval);
    pg.keepalives_retries(retries);
    pg.tcp_user_timeout(idle + probe_interval * retries);

    // Дедлайн больше суммы внутренних защит (3 с на подключение + 5 с на
    // запрос). Иначе он выстреливал бы первым и глушил точный диагноз:
    // в логе была бы бесполезная строка «ответа нет» вместо
    // «connection timed out» или «query_canceled».
    let deadline = Duration::from_secs(10);
    let interval = Duration::from_secs(settings.ping_interval_seconds);

    log::info!("failsnone: все защиты, дедлайн пробы {} с, интервал {} с", deadline.as_secs(), settings.ping_interval_seconds);

    // Канал живёт между итерациями: если проба не уложилась в дедлайн, на
    // следующем цикле мы ждём её же, а не запускаем вторую.
    //
    // Без этого каждый пропущенный дедлайн оставлял бы висеть поток вместе с
    // его подключением к базе. При интервале 3 с пятиминутная пауза дала бы
    // 100 брошенных подключений — ровно max_connections по умолчанию.
    let mut pending: Option<mpsc::Receiver<()>> = None;
    let mut waiting_since = Instant::now();

    loop {
        let waiting_for = match pending {
            // Предыдущая проба не вернулась: ждём её, новую не запускаем.
            Some(receiver) => receiver,
            None => {
                let worker_pg = pg.clone();
                let worker_query = settings.query.clone();
                let (finished_tx, finished_rx) = mpsc::channel();

                thread::spawn(move || {
                    app::ping(&worker_pg, &worker_query);
                    // Получатель мог уже уйти по дедлайну — это нормально.
                    let _ = finished_tx.send(());
                });

                waiting_since = Instant::now();
                finished_rx
            }
        };

        match waiting_for.recv_timeout(deadline) {
            Ok(()) => {
                // Если ждали больше одного цикла, об этом стоит сказать:
                // сама задержка — диагноз.
                let waited = waiting_since.elapsed();
                if waited > deadline {
                    log::info!("проба вернулась через {} с", waited.as_secs());
                }
                pending = None;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Поток остаётся заблокированным: синхронный драйвер не умеет
                // прерывать запрос в полёте. Освободит его keepalive — или
                // снятие паузы в сценарии 4.
                log::error!(
                    "проба не отвечает {} с, цикл пропущен",
                    waiting_since.elapsed().as_secs()
                );
                pending = Some(waiting_for);
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                log::error!("поток пробы завершился аварийно");
                pending = None;
            }
        }

        thread::sleep(interval);
    }
}

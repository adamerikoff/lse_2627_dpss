// Фичи-экраны. Каждая держит своё состояние и логику (handle_key) и рисует
// себя (view). app.rs только маршрутизирует клавиши и применяет Action.

pub mod browse;
pub mod login;

// Активный экран несёт состояние своей фичи. browse::State большой, поэтому
// в боксе — чтобы enum не раздувался до размера самого крупного варианта.
pub enum Screen {
    Login(login::State),
    Browse(Box<browse::State>),
}

// Что фича просит сделать приложение после обработки клавиши.
pub enum Action {
    None,
    Enter(Screen),
}

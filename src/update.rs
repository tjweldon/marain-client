use crate::tui_framework::Event;
use crate::App;
use crossterm::event::KeyEvent;

pub fn update(app: &mut App, event: Event) {
    match event {
        Event::Tick => {}
        Event::Key(KeyEvent { code: key, .. }) => {
            let Some(cmd) = app.map_key(key) else {
                return;
            };
            app.handle(cmd);
        }
<<<<<<< Updated upstream
=======
        Event::Recv(msg) => {
            app.push_log(Log::always_from_string(msg));
        }
>>>>>>> Stashed changes
        _ => {}
    }
}

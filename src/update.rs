use crate::app::Log;
use crate::tui_framework::Event;
use crate::App;
use chrono::Utc;
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
        Event::Recv(msg) => {
            let mut parts: Vec<String> = msg.splitn(2, ":").map(String::from).collect();
            if parts.len() < 2 {
                parts = vec!["unknown".into(), parts.get(0).unwrap().to_owned()];
            }
            app.push_log(Log(Utc::now(), parts[0].clone(), parts[1].clone()));
        }
        _ => {}
    }
}

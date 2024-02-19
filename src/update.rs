use crate::app::Log;
use crate::tui_framework::Event;
use crate::App;
use chrono::{DateTime, Utc};
use crossterm::event::KeyEvent;
use marain_api::prelude::{ChatMsg, ServerMsg, ServerMsgBody};

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
            if let Ok(deserialized) = serde_json::from_str::<ServerMsg>(&msg) {
                let timestamp_dt = Into::<Option<DateTime<Utc>>>::into(deserialized.timestamp);

                match deserialized.body {
                    ServerMsgBody::LoginSuccess { token } => app.store_token(token),

                    ServerMsgBody::ChatRecv {
                        chat_msg:
                            ChatMsg {
                                sender, content, ..
                            },
                        ..
                    } => {
                        app.push_log(Log(timestamp_dt.unwrap_or(Utc::now()), sender, content));
                    }
                    _ => {}
                }
            } else {
                app.push_log(Log::always_from_string(msg));
            }
        }
        _ => {}
    }
}

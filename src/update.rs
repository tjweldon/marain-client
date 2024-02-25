use crate::app::Log;
use crate::tui_framework::Event;
use crate::App;
use chrono::{DateTime, Utc};
use crossterm::event::KeyEvent;
use marain_api::prelude::{ChatMsg, ServerMsg, ServerMsgBody, Status};

pub fn update(app: &mut App, event: Event) {
    match event {
        Event::Tick => {}

        // User input event handling
        Event::Key(KeyEvent { code: key, .. }) => {
            if let Some(cmd) = app.map_key(key) {
                app.handle(cmd);
            }
        }

        // Websocket event handling
        Event::Recv(msg) => {
            if let Ok(deserialized) = serde_json::from_str::<ServerMsg>(&msg) {
                let timestamp_dt = Into::<Option<DateTime<Utc>>>::into(deserialized.timestamp);

                // Handle any errors
                match deserialized.status {
                    Status::Yes => {}
                    Status::No(error_msg) => {
                        app.push_log(Log(Utc::now(), "SERVER".into(), error_msg.clone()));
                        log::error!("The computer said no: {error_msg}");
                        return;
                    }
                    Status::JustNo => {
                        app.push_log(Log(Utc::now(), "CLIENT".into(), "Failed to login".into()));
                        return;
                    }
                }

                // These are all success responses from the server
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
                    ServerMsgBody::Empty => {
                        let server_time = timestamp_dt.unwrap_or_else(|| {
                            log::error!("Server did not supply time");
                            Utc::now()
                        });
                        app.push_log(Log(
                            Utc::now(),
                            "SERVER".into(),
                            "The time is: ".to_string()
                                + &server_time.format("%Y-%m-%D %H:%M:%S").to_string(),
                        ))
                    }
                    ServerMsgBody::RoomData { logs, .. } => {
                        let chat_logs: Vec<Log> = logs
                            .iter()
                            .map(|cm| {
                                Log(
                                    Into::<Option<DateTime<Utc>>>::into(cm.timestamp.clone())
                                        .unwrap_or(Utc::now()),
                                    cm.sender.clone(),
                                    cm.content.clone(),
                                )
                            })
                            .collect();
                        app.replace_logs(chat_logs);
                    }
                }
            } else {
                app.push_log(Log::always_from_string(msg));
            }
        }
        _ => {}
    }
}

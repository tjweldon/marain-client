use crate::app::{App, Mode};
use crate::chat_log::Log;
use crate::tui_framework::Event;
use chrono::{DateTime, Utc};
use crossterm::event::KeyEvent;
use marain_api::prelude::{ChatMsg, ServerMsg, ServerMsgBody, Status, Timestamp};

fn translate_ts(ts: Timestamp) -> DateTime<Utc> {
    Into::<Option<DateTime<Utc>>>::into(ts).unwrap_or(Utc::now())
}

pub fn update(app: &mut App, event: Event) {
    match event {
        Event::Tick => {}

        // User input event handling
        Event::Key(KeyEvent { code: key, .. }) => {
            if let Some(cmd) = app.map_key(key) {
                app.handle(cmd);
            }
        }

        // Socket closed by server
        Event::ServerClose => {
            app.push_log(Log::new(
                "SERVER".into(),
                "Connection closed by server".into(),
            ));
            app.switch_mode(Mode::Disconnected);
        }

        // Websocket event handling
        Event::Recv(msg) => {
            match bincode::deserialize::<ServerMsg>(&msg[..]) {
                Ok(deserialized) => {
                    app.push_debug_log(deserialized.clone());

                    // Handle any errors
                    match deserialized.status {
                        Status::Yes => handle_server_msg(app, deserialized),
                        Status::No(error_msg) => {
                            app.push_log(Log::new("SERVER".into(), error_msg.clone()));
                            log::error!("The computer said no: {error_msg}");
                            return;
                        }
                        Status::JustNo => {
                            app.push_log(Log::new("CLIENT".into(), "Failed to login".into()));
                            return;
                        }
                    }
                }
                Err(deserialization_err) => {
                    app.push_log(Log::new(
                        "CLIENT".into(),
                        format!("Could not deserialize inbound message: {deserialization_err}"),
                    ));
                }
            }
        }
        _ => {}
    }
}

fn handle_server_msg(app: &mut App, deserialized: ServerMsg) {
    let dt = translate_ts(deserialized.timestamp.clone());
    // These are all success responses from the server
    match deserialized.body {
        ServerMsgBody::LoginSuccess { .. } => {
            panic!("Received a second LoginSuccess message from the server.")
        }
        ServerMsgBody::ChatRecv {
            chat_msg: ChatMsg {
                sender, content, ..
            },
            ..
        } => {
            app.push_log(Log::new(sender, content).at(dt));
        }
        ServerMsgBody::Empty => app.push_log(Log::new(
            "SERVER".into(),
            "The time is: ".to_string() + &dt.format("%Y-%m-%D %H:%M:%S").to_string(),
        )),
        ServerMsgBody::RoomData { logs, .. } => {
            let chat_logs: Vec<Log> = logs
                .iter()
                .map(|cm| {
                    Log::new(cm.sender.clone(), cm.content.clone())
                        .at(translate_ts(cm.timestamp.clone()))
                })
                .collect();
            app.replace_logs(chat_logs);
        }
    }
}

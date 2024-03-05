use crate::app::Log;
use crate::app::{App, Mode};
use crate::tui_framework::Event;
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
                    let timestamp_dt = Into::<Option<DateTime<Utc>>>::into(deserialized.timestamp);

                    // Handle any errors
                    match deserialized.status {
                        Status::Yes => {}
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

                    // These are all success responses from the server
                    match deserialized.body {
                        ServerMsgBody::LoginSuccess { token , public_key} => app.store_token(token),
                        ServerMsgBody::ChatRecv {
                            chat_msg:
                                ChatMsg {
                                    sender, content, ..
                                },
                            ..
                        } => {
                            app.push_log(
                                Log::new(sender, content).at(timestamp_dt.unwrap_or(Utc::now())),
                            );
                        }
                        ServerMsgBody::Empty => {
                            let server_time = timestamp_dt.unwrap_or_else(|| {
                                log::error!("Server did not supply time");
                                Utc::now()
                            });
                            app.push_log(Log::new(
                                "SERVER".into(),
                                "The time is: ".to_string()
                                    + &server_time.format("%Y-%m-%D %H:%M:%S").to_string(),
                            ))
                        }
                        ServerMsgBody::RoomData { logs, .. } => {
                            let chat_logs: Vec<Log> = logs
                                .iter()
                                .map(|cm| {
                                    Log::new(cm.sender.clone(), cm.content.clone()).at(Into::<
                                        Option<DateTime<Utc>>,
                                    >::into(
                                        cm.timestamp.clone(),
                                    )
                                    .unwrap_or(Utc::now()))
                                })
                                .collect();
                            app.replace_logs(chat_logs);
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

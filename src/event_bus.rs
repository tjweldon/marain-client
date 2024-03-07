use color_eyre::Result;
use marain_api::prelude::{ClientMsg, ClientMsgBody, Timestamp};

use crate::{
    app::App,
    tui_framework::{Event, Tui},
    update::update,
};

pub fn dispatch(app: &mut App, tui: &mut Tui, event: Event) -> Result<()> {
    match event {
        Event::Render => {
            tui.draw(app)?;
        }
        Event::Send {
            token,
            timestamp,
            ref contents,
            ..
        } => {
            let msg = ClientMsg {
                token: Some(token),
                body: ClientMsgBody::SendToRoom {
                    contents: contents.clone(),
                },
                timestamp: Timestamp::from(timestamp),
            };
            tui.push_binary_msg_to_server(msg);
        }
        Event::ServerCommand {
            token,
            timestamp,
            message_body,
            ..
        } => {
            let server_msg = ClientMsg {
                token: Some(token),
                timestamp: Timestamp::from(timestamp),
                body: message_body,
            };
            tui.push_binary_msg_to_server(server_msg);
        }
        e => {
            log::info!("No handling for {e:?}");
            update(app, tui, e);
        }
    }
    Ok(())
}

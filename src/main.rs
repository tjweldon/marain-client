mod app;
mod shared_secret;
mod socket_client;
mod tui_framework;
mod ui;
mod update;
mod user_config;

use color_eyre::Result;
use crossterm::{
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use marain_api::prelude::{ClientMsg, ClientMsgBody, Timestamp};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::stdout;

use crate::app::App;
use crate::update::update;
use crate::user_config::load_config;
use tui_framework::*;

async fn setup() -> Result<(App, Tui)> {
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut tui = Tui::from_conf(
        terminal,
        TuiConf {
            update_freq: 30.0,
            ..TuiConf::default()
        },
    )
    .default_client();

    let mut app = App::new(load_config().await);
    let client = shared_secret::handle_login_success(&mut tui, &mut app).await;

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    tui.enter(client).await?;
    app.set_send_chan(tui.get_sender());

    Ok((app, tui))
}

async fn run() -> Result<()> {
    let (mut app, mut tui) = setup().await?;

    while !app.should_quit {
        let event = tui.next().await?;
        if let Event::Render = event {
            tui.draw(&mut app)?;
        }
        update(&mut app, event.clone());

        match event {
            Event::Send {
                token,
                timestamp,
                contents,
                ..
            } => {
                let msg = ClientMsg {
                    token: Some(token),
                    body: ClientMsgBody::SendToRoom { contents },
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
            _ => {
                log::info!("No handling for {event:?}");
            }
        }
    }

    tui.exit()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    _ = log2::open("log.txt").module(true).start();

    log::error!("SANITY CHECK");

    let result = run().await;

    result?;

    Ok(())
}

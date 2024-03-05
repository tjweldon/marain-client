mod app;
mod socket_client;
mod tui_framework;
mod ui;
mod update;
mod user_config;

use chrono::Utc;
use color_eyre::Result;
use crossterm::{
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use marain_api::prelude::{ClientMsg, ClientMsgBody, Timestamp};
use rand_core::OsRng;
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::stdout;
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::app::App;
use crate::update::update;
use crate::user_config::load_config;
use tui_framework::*;

fn create_key_pair() -> (EphemeralSecret, PublicKey) {
    let client_secret = EphemeralSecret::random_from_rng(OsRng);
    let client_public = PublicKey::from(&client_secret);

    (client_secret, client_public)
}

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
    let (client_secret, client_public) = create_key_pair();
    let (client, token, server_public_key) = match tui
        .connect(ClientMsg {
            token: None,
            body: ClientMsgBody::Login(app.username.clone(), *client_public.as_bytes()),
            timestamp: Timestamp::from(Utc::now()),
        })
        .await
    {
        Some(x) => x,
        None => panic!("Could not retrieve token from server"),
    };

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    tui.enter(client).await?;
    let shared_secret = client_secret.diffie_hellman(&server_public_key);
    app.set_shared_secret(*shared_secret.as_bytes());
    app.token = Some(token);
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

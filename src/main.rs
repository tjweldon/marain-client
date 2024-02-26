mod app;
mod socket_client;
mod tui_framework;
mod ui;
mod update;

use chrono::Utc;
use color_eyre::Result;
use crossterm::{
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use env_logger::Env;
use marain_api::prelude::{ClientMsg, ClientMsgBody, Timestamp};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::stdout;

use crate::app::App;
use crate::update::update;
use tui_framework::*;

fn setup() -> Result<(App, Tui)> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut tui = Tui::from_conf(
        terminal,
        TuiConf {
            update_freq: 30.0,
            ..TuiConf::default()
        },
    )
    .default_client();
    let mut app = App::new();

    tui.enter(ClientMsg {
        token: None,
        body: ClientMsgBody::Login(app.username.clone()),
        timestamp: Timestamp::from(Utc::now()),
    })?;
    app.set_send_chan(tui.get_sender());

    Ok((app, tui))
}

async fn run() -> Result<()> {
    let (mut app, mut tui) = setup()?;

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
                    token,
                    body: ClientMsgBody::SendToRoom { contents },
                    timestamp: Timestamp::from(timestamp),
                };
                tui.push_msg_to_server(msg);
            }
            Event::ServerCommand {
                token,
                timestamp,
                message_body,
                ..
            } => {
                let server_msg = ClientMsg {
                    token,
                    timestamp: Timestamp::from(timestamp),
                    body: message_body,
                };
                tui.push_msg_to_server(server_msg);
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
    use env_logger::{Builder, Target};

    let mut builder = Builder::new();
    builder.parse_env(Env::default());
    builder.target(Target::Stderr);
    builder.build();

    let result = run().await;

    result?;

    Ok(())
}

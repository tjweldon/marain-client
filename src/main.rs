mod app;
mod socket_client;
mod tui_framework;
mod ui;
mod update;

use color_eyre::Result;
use crossterm::{
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
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
    tui.enter()?;
    let mut app = App::new();
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

        if let Event::Send(_) = event {
            tui.push_server_msg(event.clone());
        }

        update(&mut app, event);
    }

    tui.exit()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::try_init()?;
    let result = run().await;

    result?;

    Ok(())
}

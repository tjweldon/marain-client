mod app;
mod chat_log;
mod default_keybinds;
mod event_bus;
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
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::stdout;

use crate::app::App;
use crate::event_bus::dispatch;
use crate::user_config::load_config;
use tui_framework::*;

async fn setup() -> Result<(App, Tui)> {
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut tui = Tui::from_conf(terminal, TuiConf::default()).default_client();

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
        dispatch(&mut app, &mut tui, event)?;
    }

    tui.exit()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    _ = log2::open("log.txt").module(true).start();

    let result = run().await;

    result?;

    Ok(())
}

mod app;
mod tui_framework;
mod ui;
mod update;

use color_eyre::Result;
use crossterm::{
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use log::error;
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
    );
    tui.enter()?;
    let app = App::new();

    Ok((app, tui))
}

async fn run() -> Result<()> {
    let (mut app, mut tui) = setup()?;

    while !app.should_quit {
        let event = tui.next().await?;
        if let Event::Render = event {
            tui.draw(&mut app)?;
        }

        update(&mut app, event);
    }

    tui.exit()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::try_init()?;
    error!("RED ALERT");
    let result = run().await;

    result?;

    Ok(())
}

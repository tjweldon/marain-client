mod app;
mod tui_framework;
mod ui;
mod update;

use chrono::{DateTime, Utc};
use color_eyre::Result;
use crossterm::{
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
<<<<<<< Updated upstream
use log::error;
=======
use env_logger::Env;
use marain_api::prelude::{ClientMsg, ClientMsgBody, Timestamp};
>>>>>>> Stashed changes
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
<<<<<<< Updated upstream
    );
    tui.enter()?;
    let app = App::new();
=======
    )
    .default_client();
    let mut app = App::new();

    tui.enter(ClientMsg {
        token: None, 
        body: ClientMsgBody::Login(app.username.clone()), 
        timestamp: Timestamp::from(Utc::now())}
    )?;
    app.set_send_chan(tui.get_sender());
>>>>>>> Stashed changes

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
<<<<<<< Updated upstream
    env_logger::try_init()?;
    error!("RED ALERT");
=======
    use env_logger::{Builder, Target};
    
    let mut builder = Builder::new();
    builder.parse_env(Env::default());
    builder.target(Target::Stderr);
    builder.build();

>>>>>>> Stashed changes
    let result = run().await;

    result?;

    Ok(())
}

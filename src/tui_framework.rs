use std::{io, panic};

use chrono::{DateTime, Utc};
use color_eyre::Result;
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyCode, KeyEvent,
        KeyEventKind, MouseEvent,
    },
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{stream::StreamExt, FutureExt};
use log::info;
use marain_api::prelude::ClientMsg;
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;

use crate::{
    app::App,
    socket_client::{SocketClient, SocketConf},
    ui,
};

/// Terminal events.
#[allow(dead_code)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Event {
    /// App Initialization
    Init,
    /// Quit Event
    Quit,
    /// Unsure about this one
    Closed,
    /// Render Event
    Render,
    /// Indicates window focus lost
    FocusGained,
    /// Indicates window focus gained
    FocusLost,
    /// Error Event
    Error,
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
    /// Inbound Message.
    Recv(String),
    /// Outbound Message.
    Send {
        token: String,
        username: String,
        timestamp: DateTime<Utc>,
        contents: String,
    },
}

impl From<char> for Event {
    fn from(value: char) -> Self {
        Self::Key(KeyEvent::from(KeyCode::Char(value)))
    }
}

impl From<KeyCode> for Event {
    fn from(value: KeyCode) -> Self {
        Self::Key(KeyEvent::from(value))
    }
}

pub struct TuiConf {
    pub update_freq: f64,
    pub render_freq: f64,
}

impl Default for TuiConf {
    fn default() -> Self {
        Self {
            update_freq: 60.0,
            render_freq: 60.0,
        }
    }
}

/// Representation of a terminal user interface.
///
///
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
pub struct Tui {
    /// Interface to the Terminal.
    pub terminal: CrosstermTerminal,
    pub task: Option<JoinHandle<()>>,
    pub socket_conf: SocketConf,

    pub receiver: UnboundedReceiver<Event>,

    pub sender: UnboundedSender<Event>,
    pub socket_sender: Option<futures::channel::mpsc::UnboundedSender<String>>,

    pub frame_rate: f64,

    pub update_rate: f64,
}

impl Tui {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: CrosstermTerminal) -> Self {
        let (sender, receiver) = unbounded_channel::<Event>();
        Self {
            terminal,
            task: None,
            socket_conf: SocketConf::default(),
            socket_sender: None,
            sender,
            receiver,
            frame_rate: 60.0,
            update_rate: 60.0,
        }
    }

    pub fn from_conf(terminal: CrosstermTerminal, config: TuiConf) -> Self {
        Self::new(terminal)
            .set_render_freq(config.render_freq)
            .set_update_freq(config.update_freq)
    }

    /// Fluent setter for the render frequency.
    /// If not set this value defaults to 60 fps.
    pub fn set_render_freq(mut self, fps: f64) -> Self {
        self.frame_rate = fps;

        self
    }

    /// Fluent setter for the update frequency.
    /// If not set this value defaults to 60 ups.
    pub fn set_update_freq(mut self, ups: f64) -> Self {
        self.update_rate = ups;

        self
    }

    pub fn configure_client(mut self, conf: SocketConf) -> Self {
        self.socket_conf = conf;

        self
    }

    pub fn default_client(self) -> Self {
        self.configure_client(SocketConf::default())
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn enter(&mut self, login_msg: ClientMsg) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

        // Define a custom panic hook to reset the terminal properties.
        // This way, you won't have your terminal messed up if an unexpected error happens.
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        self.start(login_msg);
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: crate::ui:render
    pub fn draw(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|frame| ui::render(app, frame))?;
        Ok(())
    }

    /// Resets the terminal interface.
    ///
    /// This function is also used for the panic hook to revert
    /// the terminal properties if unexpected errors occur.
    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn get_sender(&self) -> UnboundedSender<Event> {
        self.sender.clone()
    }

    /// Starts the async event loop
    pub fn start(&mut self, on_connect: ClientMsg) {
        let update_delay = std::time::Duration::from_secs_f64(1.0 / self.update_rate);
        let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);
        let client: SocketClient = self.socket_conf.spawn_client();
        let socket_sender = client.out_sink.clone();
        socket_sender
            .unbounded_send(serde_json::to_string(&on_connect).expect("The api code is broken"))
            .expect("Could not connect to the marain server.");
        self.socket_sender = Some(socket_sender.clone());
        info!("Sent login message to {:?}", self.socket_conf.url());

        let update_sender = self.sender.clone();

        // worker code -----
        let task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut update_interval = tokio::time::interval(update_delay);
            let mut render_interval = tokio::time::interval(render_delay);
            let mut client = client;

            loop {
                let update_delay = update_interval.tick();
                let render_delay = render_interval.tick();
                let input_event = reader.next().fuse();
                let server_event = client.next();

                tokio::select! {
                    maybe_recv = server_event => {
                        match maybe_recv {
                            Ok(message) => {
                                update_sender.send(Event::Recv(message)).unwrap();
                            },
                            _ => {},
                        }
                    }
                    maybe_input = input_event => {
                        // user events
                        match maybe_input {
                            Some(Ok(evt)) => match evt {
                                CrosstermEvent::Key(key) => {
                                    if key.kind == KeyEventKind::Press {
                                        update_sender.send(Event::Key(key)).unwrap();
                                    }
                                }
                                CrosstermEvent::Mouse(e) => {
                                    update_sender.send(Event::Mouse(e)).unwrap();
                                }
                                CrosstermEvent::Resize(w, h) => {
                                    update_sender.send(Event::Resize(w, h)).unwrap();
                                }
                                _ => info!("Handler not implemented for: {:?}", evt),
                            },
                            Some(Err(_)) => {
                                update_sender.send(Event::Error).unwrap();
                            },
                            None => {},
                        }
                    },
                    // backend/app update trigger
                    _update_tick = update_delay => {
                        update_sender.send(Event::Tick).unwrap();
                    },
                    // render trigger
                    _frame_tick = render_delay => {
                        update_sender.send(Event::Render).unwrap();
                    }
                }
            }
        });
        // end worker code --

        self.task = Some(task);
    }

    pub fn push_msg_to_server(&self, send_event: ClientMsg) {
        if let (Some(ref sender), Event::Send(msg)) = (self.socket_sender.clone(), send_event) {
            sender.unbounded_send(msg).unwrap();
        }
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or(color_eyre::eyre::eyre!("Unable to get event"))
    }
}

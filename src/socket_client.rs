use color_eyre::Result;
use futures::channel::mpsc::unbounded;
use futures_util::{
    future, pin_mut,
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use log2 as log;
use tokio::{
    net::TcpStream,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{handshake::client::Response, Message},
    MaybeTlsStream, WebSocketStream,
};
use url::Url;

#[derive(Clone, Debug)]
pub struct SocketConf {
    host: String,
    port: String,
}

impl SocketConf {
    pub fn url(&self) -> Url {
        if self.host.contains("/") {
            panic!("Just supply the hostname e.g. 'localhost'");
        }
        let url = Url::parse(&format!("ws://{}:{}", self.host, self.port))
            .expect("Failed to parse the socket url");
        log::info!("Parsed socket url: {}", url);

        url
    }

    pub async fn spawn_client(&self) -> SocketClient {
        SocketClient::init(self.clone()).await
    }
}

impl Default for SocketConf {
    fn default() -> Self {
        Self {
            host: std::env::args()
                .nth(1)
                .expect("Provide a host as the first position arg"),
            port: std::env::args().nth(2).unwrap_or("1337".into()),
        }
    }
}

pub struct SocketClient {
    _task: JoinHandle<()>,
    pub out_sink: futures::channel::mpsc::UnboundedSender<Message>,
    pub in_source: UnboundedReceiver<Message>,
}

impl SocketClient {
    /// This is the async process that handles forwarding of inbound and outbound messages to/from
    /// the socket stream.
    async fn work(
        outbound_source: futures::channel::mpsc::UnboundedReceiver<Message>,
        inbound_sink: UnboundedSender<Message>,
        ws_sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        ws_source: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) {
        let ws_to_inbound = ws_source.for_each(|message| async {
            match message {
                Ok(msg) => match msg {
                    Message::Text(_) => {
                        log::error!("Incorrect protocol detected");
                    }
                    Message::Binary(_) | Message::Close(_) => {
                        inbound_sink
                            .send(msg)
                            .expect("Could not forward inbound message from SocketClient");
                    }
                    _ => {
                        panic!("UNEXPECTED {msg:?}");
                    }
                },
                Err(e) => {
                    log::error!("SocketClient got error trying to read msg: {e}");
                }
            };
        });
        let outbound_to_ws = outbound_source.map(|s| Ok(s)).forward(ws_sink);

        pin_mut!(ws_to_inbound, outbound_to_ws);
        future::select(ws_to_inbound, outbound_to_ws).await;
    }

    pub async fn init(conf: SocketConf) -> Self {
        let (out_sink, out_source) = unbounded::<Message>();
        let (in_sink, in_source) = unbounded_channel::<Message>();
        let url = conf.url();
        let (ws_stream, _smth): (WebSocketStream<MaybeTlsStream<TcpStream>>, Response) =
            connect_async(url.clone())
                .await
                .expect(&format!("Failed to connect to {}", url));

        let (ws_sink, ws_source): (
            SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
            SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        ) = ws_stream.split();

        let _task = tokio::spawn(Self::work(out_source, in_sink, ws_sink, ws_source));
        Self {
            _task,
            out_sink,
            in_source,
        }
    }

    pub async fn next(&mut self) -> Result<Message> {
        self.in_source
            .recv()
            .await
            .ok_or(color_eyre::eyre::eyre!("Could not get socket message"))
    }
}

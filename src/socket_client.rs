use color_eyre::Result;
use futures::channel::mpsc::unbounded;
use futures_util::{
    future, pin_mut,
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use log::{error, info};
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
    fn url(&self) -> Url {
        let url = Url::parse(&format!("ws://{}:{}", self.host, self.port))
            .expect("Failed to parse the socket url");
        info!("Parsed socket url: {}", url);

        url
    }

    pub fn spawn_client(&self) -> SocketClient {
        SocketClient::init(self.clone())
    }
}

impl Default for SocketConf {
    fn default() -> Self {
        Self {
            host: "194.164.21.207".into(),
            port: "1337".into(),
        }
    }
}

pub struct SocketClient {
    _task: JoinHandle<()>,
    pub out_sink: futures::channel::mpsc::UnboundedSender<String>,
    pub in_source: UnboundedReceiver<String>,
}

impl Default for SocketClient {
    fn default() -> Self {
        Self::init(SocketConf::default())
    }
}

impl SocketClient {
    /// This is the async process that handles forwarding of inbound and outbound messages to/from
    /// the socket stream.
    async fn work(
        outbound_source: futures::channel::mpsc::UnboundedReceiver<String>,
        inbound_sink: UnboundedSender<String>,
        conf: SocketConf,
    ) {
        let url = conf.url();
        let (ws_stream, _smth): (WebSocketStream<MaybeTlsStream<TcpStream>>, Response) =
            connect_async(url.clone()).await.expect(&format!("Failed to connect to {}", url));

        let (ws_sink, ws_source): (
            SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
            SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        ) = ws_stream.split();

        let ws_to_inbound = ws_source.for_each(|message| async {
            match message {
                Ok(msg) => {
                    let txt_msg = msg.into_text().unwrap_or("unreadable".into());
                    inbound_sink
                        .send(txt_msg)
                        .expect("Could not forward inbount message from SocketClient");
                }
                Err(e) => {
                    error!("SocketClient got error trying to read msg: {e}");
                }
            };
        });
        let outbound_to_ws = outbound_source
            .map(|s| Ok(Message::text(s)))
            .forward(ws_sink);

        pin_mut!(ws_to_inbound, outbound_to_ws);
        future::select(ws_to_inbound, outbound_to_ws).await;
    }

    pub fn init(conf: SocketConf) -> Self {
        let (out_sink, out_source) = unbounded::<String>();
        let (in_sink, in_source) = unbounded_channel::<String>();
        let _task = tokio::spawn(Self::work(out_source, in_sink, conf));
        Self {
            _task,
            out_sink,
            in_source,
        }
    }

    pub async fn next(&mut self) -> Result<String> {
        self.in_source
            .recv()
            .await
            .ok_or(color_eyre::eyre::eyre!("Could not get socket message"))
    }
}

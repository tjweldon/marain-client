use chrono::Utc;
use marain_api::prelude::{ClientMsg, ClientMsgBody, Timestamp};
use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::{app::App, socket_client::SocketClient, Tui};

pub fn create_key_pair() -> (EphemeralSecret, PublicKey) {
    let client_secret = EphemeralSecret::random_from_rng(OsRng);
    let client_public = PublicKey::from(&client_secret);

    (client_secret, client_public)
}

fn login_msg(app: &App, client_public: PublicKey) -> ClientMsg {
    ClientMsg {
        token: None,
        body: ClientMsgBody::Login(app.username.clone(), *client_public.as_bytes()),
        timestamp: Timestamp::from(Utc::now()),
    }
}

pub async fn handle_login_success(tui: &mut Tui, app: &mut App) -> SocketClient {
    let (client_secret, client_public) = create_key_pair();
    let (client, token, server_public_key) = match tui.connect(login_msg(app, client_public)).await
    {
        Some(x) => x,
        None => panic!("Could not retrieve token from server"),
    };
    let shared_secret = client_secret.diffie_hellman(&server_public_key);
    app.set_shared_secret(*shared_secret.as_bytes());
    app.store_token(token);

    client
}

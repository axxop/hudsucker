use hudsucker::{
    async_trait::async_trait,
    hyper::{Body, Request, Response},
    rustls::internal::pemfile,
    *,
};
use log::*;
use std::net::SocketAddr;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

#[derive(Clone)]
struct NoopHandler {}

#[async_trait]
impl RequestResponseHandler for NoopHandler {
    async fn handle_request(&mut self, req: Request<Body>) -> RequestOrResponse {
        RequestOrResponse::Request(req)
    }

    async fn handle_response(&mut self, res: Response<Body>) -> Response<Body> {
        res
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut private_key_bytes: &[u8] = include_bytes!("ca/hudsucker.key");
    let mut ca_cert_bytes: &[u8] = include_bytes!("ca/hudsucker.pem");
    let private_key = pemfile::pkcs8_private_keys(&mut private_key_bytes)
        .expect("Failed to parse private key")
        .remove(0);
    let ca_cert = pemfile::certs(&mut ca_cert_bytes)
        .expect("Failed to parse CA certificate")
        .remove(0);

    let ca = CertificateAuthority::new(private_key, ca_cert, 1_000)
        .expect("Failed to create Certificate Authority");

    let proxy_config = ProxyConfig {
        listen_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
        shutdown_signal: shutdown_signal(),
        request_response_handler: NoopHandler {},
        incoming_message_handler: |msg| Some(msg),
        outgoing_message_handler: |msg| Some(msg),
        upstream_proxy: None,
        ca,
    };

    if let Err(e) = start_proxy(proxy_config).await {
        error!("{}", e);
    }
}

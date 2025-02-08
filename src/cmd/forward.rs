use crate::proxy::chain::ForwardChain;
use crate::proxy::handlers::log::LogHandler;
use crate::proxy::HttpHandler;
use anyhow::Result;
use clap::Parser;
use hudsucker::rcgen::{CertificateParams, KeyPair};
use hudsucker::rustls::crypto::aws_lc_rs;
use hudsucker::{certificate_authority::RcgenAuthority, Body, Proxy};
use log::error;
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use rustls::crypto::ring;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[derive(Parser, Debug)]
pub struct Args {
    /// path of the ca key
    #[arg(long, default_value = "ca.key")]
    ca_key: String,

    /// path of the ca cert
    #[arg(long, default_value = "ca.cert")]
    ca_cert: String,

    #[arg(long, default_value = "3000")]
    port: u16,
}

pub async fn run(args: &Args) -> Result<()> {
    ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let key_pair = fs::read_to_string(args.ca_key.clone()).expect("Failed to read CA key file");
    let ca_cert = fs::read_to_string(args.ca_cert.clone()).expect("Failed to read CA cert file");
    let key_pair = KeyPair::from_pem(key_pair.as_str()).expect("Failed to parse private key");
    let ca_cert = CertificateParams::from_ca_cert_pem(ca_cert.as_str())
        .expect("Failed to parse CA certificate")
        .self_signed(&key_pair)
        .expect("Failed to sign CA certificate");

    let ca = RcgenAuthority::new(key_pair, ca_cert, 1_000, aws_lc_rs::default_provider());

    let handlers: Vec<Arc<dyn HttpHandler<Body>>> = vec![Arc::new(LogHandler::<Body>::new())];

    let chain = ForwardChain::new(handlers);
    let proxy = Proxy::builder()
        .with_addr(SocketAddr::from(([0, 0, 0, 0], args.port)))
        .with_ca(ca)
        .with_rustls_client(ring::default_provider())
        .with_http_handler(chain)
        .with_graceful_shutdown(shutdown_signal())
        .build()
        .expect("Failed to create proxy");

    if let Err(e) = proxy.start().await {
        error!("Failed to start proxy {}", e);
    }

    Ok(())
}

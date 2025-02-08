use crate::proxy::chain::ReverseChain;
use crate::proxy::{ HttpHandler};
use axum::body::Body;
use axum::extract::State;
use axum::routing::any;
use axum::Router;
use clap::Parser;
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};
use std::net::SocketAddr;
use std::sync::Arc;
use crate::proxy::handlers::log::LogHandler;

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long, default_value = "4000")]
    port: u16,
}

pub async fn run(args: &Args) -> anyhow::Result<()> {
    let client: Client =
        hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
            .build(HttpConnector::new());

    let handlers: Vec<Arc<dyn HttpHandler<Body>>> = vec![
        Arc::new(LogHandler::<Body>::new())
    ];
    let chain = ReverseChain::new(handlers);

    let app = Router::new()
        .route("/", any::<ReverseChain, (), State<Client>>(chain))
        .with_state(State(client));

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", args.port)).await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}



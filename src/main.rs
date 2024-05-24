mod banks;
mod cli;
mod error;
pub mod fx;
mod handler;
mod import;
pub mod processing;
pub mod realms;
mod state;

use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::Method;
use axum::routing::post;
use axum::{routing::get, Router};
use clap::Parser;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    match cli::Args::parse().command {
        cli::Command::Import(_args) => {}
        cli::Command::Serve(_args) => serve().await?,
    }

    Ok(())
}

async fn serve() -> anyhow::Result<()> {
    // build our application with a route
    let app = Router::new().route("/", get(handler::index::handler)).nest(
        "/",
        Router::<AppState>::new()
            .route("/data", get(handler::portfolio::get::handler))
            .route("/ledgers", get(handler::ledger::list::handler))
            .route("/ledgers/summary", get(handler::ledger::summary::handler))
            .route(
                "/ledger/:id",
                get(handler::ledger::get::handler).post(handler::ledger::update::handler),
            )
            .route("/ledger", post(handler::ledger::create::handler))
            .with_state(AppState::new()?)
            .layer(
                CorsLayer::new()
                    .allow_origin(AllowOrigin::predicate(move |origin, _parts| {
                        if let Ok(origin) = origin.to_str() {
                            origin.contains("127.0.0.1") || origin.contains("localhost")
                        } else {
                            false
                        }
                    }))
                    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                    .allow_headers(vec![AUTHORIZATION, ACCEPT, CONTENT_TYPE])
                    .max_age(std::time::Duration::from_secs(3600)),
            ),
    );

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

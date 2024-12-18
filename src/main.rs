mod banks;
mod cli;
mod error;
pub mod fx;
mod handler;
mod import;
pub mod processing;
pub mod realms;
mod state;

use anyhow::Result;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::Method;
use axum::routing::{get, post, put};
use axum::Router;
use clap::Parser;
use reqwest::header::ACCESS_CONTROL_ALLOW_CREDENTIALS;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    init()?;

    match cli::Args::parse().command {
        cli::Command::Serve(_args) => serve().await?,
    }

    Ok(())
}

async fn serve() -> anyhow::Result<()> {
    let base_path = std::env::var("BASE_PATH")?;
    // build our application with a route
    let app = Router::new()
        .nest(
            &base_path,
            Router::<AppState>::new()
                .route("/data", get(handler::portfolio::get::handler))
                .route("/ledgers", get(handler::ledger::list::handler))
                .route("/ledgers/summary", get(handler::ledger::summary::handler))
                .nest(
                    "/ledger/:id",
                    Router::<AppState>::new()
                        .route(
                            "/",
                            get(handler::ledger::get::handler)
                                .post(handler::ledger::create::handler)
                                .put(handler::ledger::update::handler)
                                .delete(handler::ledger::delete::handler),
                        )
                        .nest(
                            "/files",
                            Router::<AppState>::new()
                                .route(
                                    "/",
                                    get(handler::ledger::files::get::handler)
                                        .post(handler::ledger::files::post::handler),
                                )
                                .nest(
                                    "/:fileName",
                                    Router::<AppState>::new().route(
                                        "/",
                                        put(handler::ledger::files::put::handler)
                                            .delete(handler::ledger::files::delete::handler),
                                    ),
                                ),
                        ),
                )
                .route(
                    "/ledger",
                    post(handler::ledger::create::handler).put(handler::ledger::update::handler),
                )
                .route("/auth/oidc", get(handler::auth::oidc::oidc_auth))
                .route(
                    "/auth/authorized",
                    get(handler::auth::login::login_authorized),
                )
                .route("/logout", get(handler::auth::logout::logout)),
        )
        .with_state(AppState::new()?)
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::predicate(move |_origin, _parts| {
                    // if let Ok(origin) = origin.to_str() {
                    //     origin.contains("127.0.0.1")
                    //         || origin.contains("localhost")
                    //         || origin.contains("zitadel.huesser.dev")
                    // } else {
                    //     false
                    // }
                    true
                }))
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers(vec![
                    AUTHORIZATION,
                    ACCEPT,
                    CONTENT_TYPE,
                    ACCESS_CONTROL_ALLOW_CREDENTIALS,
                ])
                .allow_credentials(true)
                .max_age(std::time::Duration::from_secs(3600)),
        );

    // run it
    let server_address = std::env::var("SERVER_ADDRESS")?;
    let listener = tokio::net::TcpListener::bind(server_address).await.unwrap();
    println!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn init() -> Result<()> {
    std::fs::create_dir_all("portfolio/ledgers")?;
    std::fs::create_dir_all("portfolio/fx")?;

    Ok(())
}

mod models;
mod parser;
mod graph;
mod handler;

use actix_web::{App, HttpServer, middleware};
use actix_cors::Cors;
use handler::{bfhl_handler, bfhl_get_handler, root_get_handler};
use tracing::info;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ── Logging setup ─────────────────────────────────────────────────────────

    // Daily rolling log file: logs/bfhl.YYYY-MM-DD
    let file_appender = rolling::daily("logs", "bfhl.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Filter: INFO and above by default; override with RUST_LOG env var
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Layer 1: coloured output to stdout
    let stdout_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_ansi(true);

    // Layer 2: plain text to the log file (no ANSI colour codes)
    let file_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_ansi(false)
        .with_writer(non_blocking);

    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    // ── Server start ──────────────────────────────────────────────────────────
    info!("Starting server on http://0.0.0.0:9000");
    info!("Logs are written to logs/bfhl.log.<date>");

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("https://bajaj-assessment-self.vercel.app")
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            // Actix access log: METHOD /path  StatusCode  latency
            .wrap(middleware::Logger::new(
                "%a \"%r\" %s %b bytes %Dms",
            ))
            .service(root_get_handler)
            .service(bfhl_get_handler)
            .service(bfhl_handler)
    })
    .bind("0.0.0.0:9000")?
    .run()
    .await
}

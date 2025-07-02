use crate::{backend::Backend, logging::CustomFormatter};
use anyhow::Result;
use rmcp::transport::sse_server::{SseServer, SseServerConfig};
use tokio_util::sync::CancellationToken;
use tracing::Level;
use tracing_subscriber::{self, EnvFilter};

pub mod backend;
pub mod config;
pub mod data_store;
pub mod doc_loader;
pub mod docs_builder;
pub mod documentation;
pub mod error;
pub mod features;
pub mod json_types;
pub mod logging;
pub mod my_types;
pub mod query;
pub mod utils;

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv_override().ok();

	tracing_subscriber::fmt()
		.event_format(CustomFormatter)
		.with_env_filter(EnvFilter::from_default_env().add_directive(Level::DEBUG.into()))
		.with_writer(std::io::stderr)
		.init();

	tracing::info!("Starting MCP SSE server");

	let port = std::env::var("PORT").unwrap_or("8080".to_string());
	let bind_addr = format!("0.0.0.0:{port}");

	let config = SseServerConfig {
		bind: bind_addr.parse()?,
		sse_path: "/sse".to_string(),
		post_path: "/message".to_string(),
		ct: CancellationToken::new(),
		sse_keep_alive: None,
	};

	let (sse_server, router) = SseServer::new(config);

	let listener = tokio::net::TcpListener::bind(sse_server.config.bind).await?;
	let server_address = sse_server.config.bind;

	let ct = sse_server.config.ct.child_token();

	let server = axum::serve(listener, router).with_graceful_shutdown(async move {
		ct.cancelled().await;
		tracing::info!("sse server cancelled");
	});

	tokio::spawn(async move {
		if let Err(e) = server.await {
			tracing::error!(error = %e, "sse server shutdown with error");
		}
	});

	let server_ct = sse_server.config.ct.clone();
	let ct = sse_server.with_service(move || Backend::new(server_ct.clone()));

	tracing::info!("Server running at http://{server_address}");

	tokio::signal::ctrl_c().await?;
	ct.cancel();

	Ok(())
}

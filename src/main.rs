use anyhow::Result;
use embed_anything_rs::services;
use rmcp::{
	Error as McpError, RoleServer, ServerHandler,
	model::*,
	schemars,
	service::RequestContext,
	tool,
	transport::sse_server::{SseServer, SseServerConfig},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{self, EnvFilter};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StructRequest {
	pub a: i32,
	pub b: i32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GenDocsRequest {
	#[schemars(description = "Crate name to generate docs for")]
	pub crate_name: String,
	#[serde(default = "default_version")]
	#[schemars(description = "Crate version requirement (defaults to *)")]
	pub version: String,
	#[serde(default)]
	#[schemars(description = "Optional features to enable")]
	pub features: Vec<String>,
}

fn default_version() -> String {
	"*".to_string()
}

#[derive(Clone)]
pub struct Counter {
	counter: Arc<Mutex<i32>>,
}

#[tool(tool_box)]
impl Counter {
	pub fn new() -> Self {
		Self {
			counter: Arc::new(Mutex::new(0)),
		}
	}

	#[tool(description = "Generate documentation for the given crate")]
	async fn generate_docs(
		&self,
		#[tool(aggr)] req: GenDocsRequest,
	) -> Result<CallToolResult, McpError> {
		let version = if req.version.is_empty() {
			"*".to_string()
		} else {
			req.version
		};

		match services::DocumentationService::generate_docs(
			&req.crate_name,
			&version,
			&req.features,
		) {
			Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!(
				"Successfully generated documentation for {} v{}",
				req.crate_name, version
			))])),
			Err(_) => Err(McpError::internal_error("internal error. try again", None)),
		}
	}

	#[tool(description = "Increment the counter by 1")]
	async fn increment(&self) -> Result<CallToolResult, McpError> {
		let mut counter = self.counter.lock().await;
		*counter += 1;
		Ok(CallToolResult::success(vec![Content::text(
			counter.to_string(),
		)]))
	}

	#[tool(description = "Get the current counter value")]
	async fn get_value(&self) -> Result<CallToolResult, McpError> {
		let counter = self.counter.lock().await;
		Ok(CallToolResult::success(vec![Content::text(
			counter.to_string(),
		)]))
	}

	#[tool(description = "Repeat what you say")]
	fn echo(
		&self,
		#[tool(param)]
		#[schemars(description = "Repeat what you say")]
		saying: String,
	) -> Result<CallToolResult, McpError> {
		Ok(CallToolResult::success(vec![Content::text(saying)]))
	}

	#[tool(description = "Calculate the sum of two numbers")]
	fn sum(
		&self,
		#[tool(aggr)] StructRequest { a, b }: StructRequest,
	) -> Result<CallToolResult, McpError> {
		Ok(CallToolResult::success(vec![Content::text(
			(a + b).to_string(),
		)]))
	}
}

#[tool(tool_box)]
impl ServerHandler for Counter {
	fn get_info(&self) -> ServerInfo {
		ServerInfo {
			protocol_version: ProtocolVersion::V_2024_11_05,
			capabilities: ServerCapabilities::builder().enable_tools().build(),
			server_info: Implementation::from_build_env(),
			instructions: Some("server details??".to_string()),
		}
	}

	async fn initialize(
		&self,
		_request: InitializeRequestParam,
		context: RequestContext<RoleServer>,
	) -> Result<InitializeResult, McpError> {
		if let Some(http_request_part) =
			context.extensions.get::<axum::http::request::Parts>()
		{
			let initialize_headers = &http_request_part.headers;
			let initialize_uri = &http_request_part.uri;
			tracing::info!(?initialize_headers, %initialize_uri, "initialize from http server");
		}
		Ok(self.get_info())
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.with_env_filter(
			EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()),
		)
		.with_writer(std::io::stderr)
		.with_ansi(false)
		.init();

	tracing::info!("Starting MCP SSE server");

	let config = SseServerConfig {
		bind: "127.0.0.1:8000".parse()?,
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

	let ct = sse_server.with_service(Counter::new);

	tracing::info!("Server running at http://{}/sse", server_address);

	tokio::signal::ctrl_c().await?;
	ct.cancel();

	Ok(())
}

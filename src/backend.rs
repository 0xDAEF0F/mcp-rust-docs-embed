use crate::{
	query_embedder::QueryEmbedder,
	services::{DocumentationService, query::QueryService},
	utils::{gen_table_name, resolve_latest_crate_version},
};
use anyhow::Result;
use rmcp::{
	Error as McpError, RoleServer, ServerHandler,
	model::{Content, *},
	schemars::{self, JsonSchema},
	service::RequestContext,
	tool,
};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenDocsRequest {
	#[schemars(description = "Crate name to generate docs for")]
	pub crate_name: String,
	#[serde(default = "default_version")]
	#[schemars(description = "Crate version requirement (defaults to *, i.e., latest)")]
	pub version: String,
	#[serde(default)]
	#[schemars(description = "Optional features to enable")]
	pub features: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EmbedRequest {
	#[schemars(description = "Crate name to embed docs for")]
	pub crate_name: String,
	#[serde(default = "default_version")]
	#[schemars(description = "Crate version (defaults to *, i.e., latest)")]
	pub version: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryRequest {
	#[schemars(description = "Query to search for in the embedded docs")]
	pub query: String,
	#[schemars(description = "Crate name to search in")]
	pub crate_name: String,
	#[serde(default = "default_version")]
	#[schemars(description = "Crate version (defaults to *, i.e., latest)")]
	pub version: String,
	#[serde(default = "default_limit")]
	#[schemars(description = "Number of results to return (defaults to 5)")]
	pub limit: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StatusRequest {
	#[schemars(description = "Operation ID to check status for")]
	pub operation_id: String,
}

fn default_version() -> String {
	"*".to_string()
}

fn default_limit() -> u64 {
	5
}

#[derive(Debug, Clone)]
pub struct EmbedOperation {
	pub status: EmbedStatus,
	pub crate_name: String,
	pub version: String,
	pub message: String,
}

#[derive(Debug, Clone)]
pub enum EmbedStatus {
	InProgress,
	Completed,
	Failed,
}

#[derive(Clone)]
pub struct Backend {
	embed_operations: Arc<RwLock<HashMap<String, EmbedOperation>>>,
	cancellation_token: CancellationToken,
}

impl Default for Backend {
	fn default() -> Self {
		Self {
			embed_operations: Arc::new(RwLock::new(HashMap::new())),
			cancellation_token: CancellationToken::new(),
		}
	}
}

#[tool(tool_box)]
impl Backend {
	pub fn new(cancellation_token: CancellationToken) -> Self {
		Self {
			embed_operations: Arc::new(RwLock::new(HashMap::new())),
			cancellation_token,
		}
	}

	#[tool(description = "Generate and embed documentation for the given crate")]
	async fn embed_docs(
		&self,
		#[tool(aggr)] req: EmbedRequest,
	) -> Result<CallToolResult, McpError> {
		let operation_id = format!("embed_{}_{}", req.crate_name, uuid::Uuid::new_v4());
		let ops = self.embed_operations.clone();
		let ct = self.cancellation_token.child_token();

		let version = if req.version.is_empty() || req.version == "*" {
			// resolve the latest version from crates.io
			match resolve_latest_crate_version(&req.crate_name).await {
				Ok(v) => v,
				Err(e) => {
					return Err(McpError::invalid_request(
						format!("Failed to resolve latest version: {}", e),
						None,
					));
				}
			}
		} else {
			req.version.clone()
		};

		// check if this version is already embedded
		let table_name = gen_table_name(&req.crate_name, &version);
		if let Ok(config) = envy::from_env::<crate::config::AppConfig>()
			&& let Ok(qdrant_client) =
				qdrant_client::Qdrant::from_url(&config.qdrant_url).build()
			&& let Ok(exists) = qdrant_client.collection_exists(&table_name).await
			&& exists
		{
			return Ok(CallToolResult::success(vec![Content::text(format!(
				"Documentation for {} v{} is already embedded",
				req.crate_name, version
			))]));
		}

		{
			let mut ops_lock = ops.write().await;
			ops_lock.insert(
				operation_id.clone(),
				EmbedOperation {
					status: EmbedStatus::InProgress,
					crate_name: req.crate_name.clone(),
					version: version.clone(),
					message: "Starting documentation generation and embedding process"
						.to_string(),
				},
			);
		}

		let op_id_clone = operation_id.clone();
		let crate_name = req.crate_name.clone();
		let version_clone = version.clone();

		tokio::spawn(async move {
			let result = tokio::select! {
				_ = ct.cancelled() => {
					Err(anyhow::anyhow!("Operation cancelled"))
				}
				res = async {
					// generate documentation first
					DocumentationService::generate_docs(
						&crate_name,
						&version_clone,
						&[],  // todo: support features in EmbedRequest
					)?;

					let embedder = QueryEmbedder::new()?;


					embedder.embed_crate(&crate_name, &version_clone).await
				} => res
			};

			let mut ops_lock = ops.write().await;
			if let Some(op) = ops_lock.get_mut(&op_id_clone) {
				match result {
					Ok(_) => {
						op.status = EmbedStatus::Completed;
						op.message = format!(
							"Successfully generated and embedded documentation for {} \
							 v{}",
							op.crate_name, op.version
						);
					}
					Err(e) => {
						op.status = EmbedStatus::Failed;
						op.message = format!("Failed to embed docs: {}", e);
					}
				}
			}
		});

		Ok(CallToolResult::success(vec![Content::text(format!(
			"Started documentation generation and embedding process with ID: {}. Use \
			 check_embed_status to monitor progress.",
			operation_id
		))]))
	}

	#[tool(description = "Query embedded documentation")]
	async fn query_docs(
		&self,
		#[tool(aggr)] req: QueryRequest,
	) -> Result<CallToolResult, McpError> {
		let version = if req.version.is_empty() || req.version == "*" {
			// resolve the latest version from crates.io
			match resolve_latest_crate_version(&req.crate_name).await {
				Ok(v) => v,
				Err(e) => {
					return Err(McpError::invalid_request(
						format!("Failed to resolve latest version: {}", e),
						None,
					));
				}
			}
		} else {
			req.version
		};

		// check if embeddings exist for this crate/version
		let table_name = gen_table_name(&req.crate_name, &version);
		if let Ok(config) = envy::from_env::<crate::config::AppConfig>()
			&& let Ok(qdrant_client) =
				qdrant_client::Qdrant::from_url(&config.qdrant_url).build()
		{
			match qdrant_client.collection_exists(&table_name).await {
				Ok(exists) => {
					if !exists {
						return Ok(CallToolResult::success(vec![Content::text(
							format!(
								"No embedded documentation found for {} v{}. Please run \
								 embed_docs first to generate and embed the \
								 documentation.",
								req.crate_name, version
							),
						)]));
					}
				}
				Err(_) => {
					// if we can't check, proceed with query anyway
				}
			}
		}

		match QueryService::query_embeddings(
			&req.query,
			&req.crate_name,
			&version,
			req.limit,
		)
		.await
		{
			Ok(results) => {
				if results.is_empty() {
					Ok(CallToolResult::success(vec![Content::text(format!(
						"No results found for query: {}",
						req.query
					))]))
				} else {
					let mut contents = vec![Content::text(format!(
						"Found {} results for query: {}",
						results.len(),
						req.query
					))];

					for (i, (score, content)) in results.iter().enumerate() {
						contents.push(Content::text(format!(
							"\n--- Result {} (score: {:.4}) ---\n{}",
							i + 1,
							score,
							content
						)));
					}

					Ok(CallToolResult::success(contents))
				}
			}
			Err(e) => Err(McpError::internal_error(
				format!("Query failed: {}", e),
				None,
			)),
		}
	}

	#[tool(description = "Check the status of an embedding operation")]
	async fn check_embed_status(
		&self,
		#[tool(aggr)] req: StatusRequest,
	) -> Result<CallToolResult, McpError> {
		// Clone the operation data to avoid holding the lock
		let op_data = {
			let ops_lock = self.embed_operations.read().await;
			ops_lock.get(&req.operation_id).cloned()
		};

		if let Some(op) = op_data {
			let status_text = match &op.status {
				EmbedStatus::InProgress => "in_progress",
				EmbedStatus::Completed => "completed",
				EmbedStatus::Failed => "failed",
			};

			Ok(CallToolResult::success(vec![Content::text(format!(
				"Embed operation {} for {} v{}: {} - {}",
				req.operation_id, op.crate_name, op.version, status_text, op.message
			))]))
		} else {
			Ok(CallToolResult::success(vec![Content::text(format!(
				"No embedding operation found with ID: {}",
				req.operation_id
			))]))
		}
	}

	#[tool(description = "List all embedding operations and their status")]
	async fn list_embed_operations(&self) -> Result<CallToolResult, McpError> {
		// Clone all operations to avoid holding the lock
		let operations = {
			let ops_lock = self.embed_operations.read().await;
			ops_lock.clone()
		};

		if operations.is_empty() {
			Ok(CallToolResult::success(vec![Content::text(
				"No embedding operations found".to_string(),
			)]))
		} else {
			let mut contents = vec![Content::text(format!(
				"Found {} embedding operations:",
				operations.len()
			))];

			for (id, op) in operations.iter() {
				let status_text = match &op.status {
					EmbedStatus::InProgress => "in_progress",
					EmbedStatus::Completed => "completed",
					EmbedStatus::Failed => "failed",
				};

				contents.push(Content::text(format!(
					"\n- {}: {} v{} [{}] - {}",
					id, op.crate_name, op.version, status_text, op.message
				)));
			}

			Ok(CallToolResult::success(contents))
		}
	}
}

#[tool(tool_box)]
impl ServerHandler for Backend {
	fn get_info(&self) -> ServerInfo {
		ServerInfo {
			protocol_version: ProtocolVersion::V_2024_11_05,
			capabilities: ServerCapabilities::builder().enable_tools().build(),
			server_info: Implementation {
				name: "embed-anything-rs".to_string(),
				version: "0.1.0".to_string(),
			},
			instructions: Some(
				"MCP server for Rust documentation embedding and search".to_string(),
			),
		}
	}

	async fn initialize(
		&self,
		_request: InitializeRequestParam,
		_context: RequestContext<RoleServer>,
	) -> Result<InitializeResult, McpError> {
		Ok(self.get_info())
	}
}

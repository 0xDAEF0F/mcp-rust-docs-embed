use crate::{
	query_embedder::QueryEmbedder,
	services::{DocumentationService, query::QueryService},
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

	/// Resolves "*" version to the actual version by looking at the docs directory
	fn resolve_version(crate_name: &str, version: &str) -> Result<String> {
		if version != "*" {
			return Ok(version.to_string());
		}

		let docs_path = format!("docs/{}", crate_name);
		let entries = std::fs::read_dir(&docs_path).map_err(|e| {
			anyhow::anyhow!(
				"Failed to read docs directory for {}: {}. Please run generate_docs \
				 first.",
				crate_name,
				e
			)
		})?;

		let mut versions: Vec<String> = entries
			.filter_map(|entry| entry.ok())
			.filter_map(|entry| {
				let path = entry.path();
				if path.is_dir() {
					path.file_name()
						.and_then(|n| n.to_str())
						.map(|s| s.to_string())
				} else {
					None
				}
			})
			.collect();

		if versions.is_empty() {
			return Err(anyhow::anyhow!(
				"No version directories found in docs/{}. Please run generate_docs \
				 first.",
				crate_name
			));
		}

		// Sort versions and take the latest (simple string sort for now)
		versions.sort();
		Ok(versions.pop().unwrap())
	}

	#[tool(description = "Generate documentation for the given crate")]
	async fn generate_docs(
		&self,
		#[tool(aggr)] req: GenDocsRequest,
	) -> Result<CallToolResult, McpError> {
		let version = if req.version.is_empty() {
			"*".to_string()
		} else {
			req.version.clone()
		};

		match DocumentationService::generate_docs(
			&req.crate_name,
			&version,
			&req.features,
		) {
			Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!(
				"Successfully generated documentation for {} v{}",
				req.crate_name, version
			))])),
			Err(e) => Err(McpError::internal_error(
				format!("Failed to generate docs: {}", e),
				None,
			)),
		}
	}

	#[tool(description = "Embed documentation for the given crate")]
	async fn embed_docs(
		&self,
		#[tool(aggr)] req: EmbedRequest,
	) -> Result<CallToolResult, McpError> {
		let operation_id = format!("embed_{}_{}", req.crate_name, uuid::Uuid::new_v4());
		let ops = self.embed_operations.clone();
		let ct = self.cancellation_token.child_token();

		let version = if req.version.is_empty() {
			"*".to_string()
		} else {
			req.version.clone()
		};

		{
			let mut ops_lock = ops.write().await;
			ops_lock.insert(
				operation_id.clone(),
				EmbedOperation {
					status: EmbedStatus::InProgress,
					crate_name: req.crate_name.clone(),
					version: version.clone(),
					message: "Starting embedding process".to_string(),
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
					let actual_version = Self::resolve_version(&crate_name, &version_clone)?;

					let embedder = QueryEmbedder::new()?;
					let result = embedder.embed_crate(&crate_name, &actual_version).await;

					// Update the operation with the actual version if it was resolved
					if version_clone == "*" {
						let mut ops_lock = ops.write().await;
						if let Some(op) = ops_lock.get_mut(&op_id_clone) {
							op.version = actual_version.clone();
						}
					}

					result
				} => res
			};

			let mut ops_lock = ops.write().await;
			if let Some(op) = ops_lock.get_mut(&op_id_clone) {
				match result {
					Ok(_) => {
						op.status = EmbedStatus::Completed;
						op.message = format!(
							"Successfully embedded documentation for {} v{}",
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
			"Started embedding process with ID: {}. Use check_embed_status to monitor \
			 progress.",
			operation_id
		))]))
	}

	#[tool(description = "Query embedded documentation")]
	async fn query_docs(
		&self,
		#[tool(aggr)] req: QueryRequest,
	) -> Result<CallToolResult, McpError> {
		let version = if req.version.is_empty() {
			"*".to_string()
		} else {
			req.version
		};

		let actual_version = match Self::resolve_version(&req.crate_name, &version) {
			Ok(v) => v,
			Err(e) => {
				return Err(McpError::invalid_request(format!("{}", e), None));
			}
		};

		match QueryService::query_embeddings(
			&req.query,
			&req.crate_name,
			&actual_version,
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
		let ops_lock = self.embed_operations.read().await;

		if let Some(op) = ops_lock.get(&req.operation_id) {
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
		let ops_lock = self.embed_operations.read().await;

		if ops_lock.is_empty() {
			Ok(CallToolResult::success(vec![Content::text(
				"No embedding operations found".to_string(),
			)]))
		} else {
			let mut contents = vec![Content::text(format!(
				"Found {} embedding operations:",
				ops_lock.len()
			))];

			for (id, op) in ops_lock.iter() {
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

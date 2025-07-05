use crate::{
	error::BackendError,
	github_processor::process_and_embed_github_repo,
	query::QueryService,
	utils::{
		extract_repo_name_from_url, gen_table_name_for_repo, parse_repository_input,
	},
};
use anyhow::{Context, Result};
use rmcp::{
	Error as McpError, RoleServer, ServerHandler,
	model::{Content, *},
	schemars::{self, JsonSchema},
	service::RequestContext,
	tool,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Custom deserializer for repository input that accepts either full URLs or owner/repo
/// format
fn deserialize_repository<'de, D>(deserializer: D) -> Result<String, D::Error>
where
	D: Deserializer<'de>,
{
	let input = String::deserialize(deserializer)?;
	parse_repository_input(&input).map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenDocsRequest {
	#[schemars(description = "Crate name to generate docs for")]
	pub crate_name: String,
	#[serde(default)]
	#[schemars(description = "Optional features to enable")]
	pub features: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EmbedRequest {
	#[serde(deserialize_with = "deserialize_repository")]
	#[schemars(
		description = "Repository to embed. Can be either a full GitHub URL (e.g., 'https://github.com/owner/repo') or shorthand format (e.g., 'owner/repo')"
	)]
	pub repo_url: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryRequest {
	#[schemars(description = "Query to search for in the embedded docs")]
	pub query: String,
	#[serde(deserialize_with = "deserialize_repository")]
	#[schemars(
		description = "Repository to search in. Can be either a full GitHub URL (e.g., 'https://github.com/owner/repo') or shorthand format (e.g., 'owner/repo')"
	)]
	pub repo_url: String,
	#[serde(default = "default_limit")]
	#[schemars(description = "Number of results to return (defaults to 10)")]
	pub limit: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StatusRequest {
	#[schemars(description = "Operation ID to check status for")]
	pub operation_id: String,
}

fn default_limit() -> u64 {
	10
}

#[derive(Debug, Clone)]
pub struct EmbedOperation {
	pub status: EmbedStatus,
	pub repo_url: String,
	pub message: String,
}

#[derive(Debug, Clone)]
pub enum EmbedStatus {
	InProgress,
	Completed,
	Failed,
}

#[derive(Clone, Default)]
pub struct Backend {
	embed_operations: Arc<RwLock<HashMap<String, EmbedOperation>>>,
	cancellation_token: CancellationToken,
}

#[tool(tool_box)]
impl Backend {
	/// Provides graceful shutdown capability by allowing background operations
	/// to be cancelled when the server needs to terminate
	pub fn new(cancellation_token: CancellationToken) -> Self {
		Self {
			cancellation_token,
			..Default::default()
		}
	}

	#[tool(description = "Generate and embed documentation from a Git repository")]
	async fn embed_repo(
		&self,
		#[tool(aggr)] req: EmbedRequest,
	) -> Result<CallToolResult, McpError> {
		tracing::info!("Starting embed_repo for repository: {}", req.repo_url);
		// Extract a safe name from the URL for the operation ID
		let repo_name = extract_repo_name_from_url(&req.repo_url)
			.unwrap_or_else(|_| "unknown".to_string());
		let operation_id = format!("embed_{}_{}", repo_name, Uuid::new_v4());
		tracing::debug!("Generated operation ID: {}", operation_id);
		let ops = self.embed_operations.clone();
		let cancellation_token = self.cancellation_token.child_token();

		// Check if this repo is already embedded
		let table_name = gen_table_name_for_repo(&req.repo_url).map_err(|e| {
			McpError::invalid_request(format!("Failed to generate table name: {e}"), None)
		})?;
		tracing::debug!("Generated table name: {}", table_name);

		tracing::info!("Checking if {} is already embedded", req.repo_url);

		if let Ok(qdrant_url) = dotenvy::var("QDRANT_URL")
			&& let Ok(qdrant_client) = qdrant_client::Qdrant::from_url(&qdrant_url)
				.api_key(dotenvy::var("QDRANT_API_KEY").ok())
				.build() && let Ok(exists) = qdrant_client.collection_exists(&table_name).await
			&& exists
		{
			tracing::info!("Repository {} is already embedded, skipping", req.repo_url);
			return Ok(CallToolResult::success(vec![Content::text(format!(
				"Repository {} is already embedded",
				req.repo_url
			))]));
		}
		tracing::info!(
			"Repository {} not found in embeddings, proceeding with embedding",
			req.repo_url
		);

		{
			tracing::debug!("Acquiring write lock for operations tracking");
			let mut ops_lock = ops.write().await;
			tracing::info!(
				"Registering operation {} for repository {}",
				operation_id,
				req.repo_url
			);
			ops_lock.insert(
				operation_id.clone(),
				EmbedOperation {
					status: EmbedStatus::InProgress,
					repo_url: req.repo_url.clone(),
					message: "Starting repository processing and embedding".to_string(),
				},
			);
		}

		let background_operation_id = operation_id.clone();
		let repo_url = req.repo_url.clone();

		tokio::spawn(async move {
			tracing::info!(
				"Spawning background task for embedding {} (operation: {})",
				repo_url,
				background_operation_id
			);
			let result = tokio::select! {
				_ = cancellation_token.cancelled() => {
					tracing::warn!("Operation {} cancelled for repository {}", background_operation_id, repo_url);
					Err(anyhow::anyhow!("Operation cancelled"))
				}
				res = async {
					// Process GitHub repository and embed it
					tracing::info!("Starting GitHub repository processing for {}", repo_url);
					let embed_result = process_and_embed_github_repo(&repo_url).await;
					match &embed_result {
						Ok(_) => tracing::info!("Successfully processed repository for {}", repo_url),
						Err(e) => tracing::error!("Failed to process repository for {}: {}", repo_url, e),
					}
					embed_result
				} => res
			};

			tracing::debug!("Updating operation status for {}", background_operation_id);
			let mut ops_lock = ops.write().await;
			if let Some(op) = ops_lock.get_mut(&background_operation_id) {
				match result {
					Ok(_) => {
						op.status = EmbedStatus::Completed;
						op.message = format!(
							"Successfully processed and embedded repository {}",
							op.repo_url
						);
						tracing::info!(
							"Operation {} completed successfully for {}",
							background_operation_id,
							op.repo_url
						);
					}
					Err(e) => {
						op.status = EmbedStatus::Failed;
						op.message = format!("Failed to embed repository: {e}");
						tracing::error!(
							"Operation {} failed for {}: {}",
							background_operation_id,
							op.repo_url,
							e
						);
					}
				}
			} else {
				tracing::warn!(
					"Operation {} not found in tracking map",
					background_operation_id
				);
			}
		});

		tracing::info!(
			"Embed operation {} started for repository {}",
			operation_id,
			req.repo_url
		);
		Ok(CallToolResult::success(vec![Content::text(format!(
			"Started repository processing and embedding with ID: {operation_id}. Sleep \
			 for about 6 seconds and then Use \"check_embed_status\" to monitor \
			 progress --- do this until it either succeeds or fails."
		))]))
	}

	#[tool(description = "Perform semantic search on repository documentation embeddings")]
	async fn query_embeddings(
		&self,
		#[tool(aggr)] req: QueryRequest,
	) -> Result<CallToolResult, McpError> {
		// Check if embeddings exist for this repository
		let table_name = gen_table_name_for_repo(&req.repo_url).map_err(|e| {
			McpError::invalid_request(format!("Failed to generate table name: {e}"), None)
		})?;
		if let Ok(qdrant_url) = dotenvy::var("QDRANT_URL")
			&& let Ok(qdrant_client) = qdrant_client::Qdrant::from_url(&qdrant_url)
				.api_key(dotenvy::var("QDRANT_API_KEY").ok())
				.build()
		{
			match qdrant_client.collection_exists(&table_name).await {
				Ok(exists) => {
					if !exists {
						return Err(McpError::invalid_request(
							format!(
								"No embeddings found for repository: {}",
								req.repo_url
							),
							None,
						));
					}
				}
				Err(_) => {
					// if we can't check, proceed with query anyway
				}
			}
		}

		let query_service = QueryService::new()
			.context("failed to initialize query service")
			.map_err(BackendError::Internal)?;

		let results = query_service
			.query_embeddings(&req.query, &req.repo_url, req.limit)
			.await
			.context("failed to query embeddings")
			.map_err(BackendError::Internal)?;

		if results.is_empty() {
			return Err(BackendError::NoQueryResults(req.query.clone()).into());
		}

		let header = format!(
			"Found {} results for query: {} (from repository: {})",
			results.len(),
			req.query,
			req.repo_url
		);

		let mut contents = vec![Content::text(header)];

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

	#[tool(description = "Check the status of an embedding operation")]
	async fn query_embed_status(
		&self,
		#[tool(aggr)] req: StatusRequest,
	) -> Result<CallToolResult, McpError> {
		// Clone the operation data to avoid holding the lock
		let op_data = {
			let ops_lock = self.embed_operations.read().await;
			ops_lock.get(&req.operation_id).cloned()
		};

		match op_data {
			Some(op) => {
				let status_text = match &op.status {
					EmbedStatus::InProgress => "in_progress",
					EmbedStatus::Completed => "completed",
					EmbedStatus::Failed => "failed",
				};

				Ok(CallToolResult::success(vec![Content::text(format!(
					"Embed operation {} for {}: {} - {}",
					req.operation_id, op.repo_url, status_text, op.message
				))]))
			}
			None => Err(BackendError::OperationNotFound(req.operation_id.clone()).into()),
		}
	}

	#[tool(
		description = "List the repositories that are already embedded in the mcp server"
	)]
	async fn list_embedded_repos(&self) -> Result<CallToolResult, McpError> {
		#[derive(Serialize)]
		struct RepoInfo {
			repo_name: String,
			embedded_at: Option<String>,
			doc_count: Option<usize>,
		}

		let mut repo_info: Vec<RepoInfo> = Vec::new();

		let qdrant_url = dotenvy::var("QDRANT_URL")
			.context("QDRANT_URL environment variable not set")
			.map_err(BackendError::Internal)?;
		let qdrant_api_key = dotenvy::var("QDRANT_API_KEY").ok();

		let qdrant_client = qdrant_client::Qdrant::from_url(&qdrant_url)
			.api_key(qdrant_api_key)
			.build()
			.context("failed to create Qdrant client")
			.map_err(BackendError::Internal)?;

		// list all collections from qdrant
		let collections = qdrant_client
			.list_collections()
			.await
			.context("failed to list collections from Qdrant")
			.map_err(BackendError::Internal)?;

		for collection in collections.collections {
			let name = collection.name;

			// parse collection name to extract repo name
			// format is: repo_{owner}_{repo}
			if let Some(repo_name) = name.strip_prefix("repo_") {
				// convert back underscores to original characters
				let repo_name = repo_name.replace('_', "/").replacen("/", "_", 1);

				// Try to get metadata for this collection
				let metadata = crate::data_store::DataStore::get_metadata(
					&qdrant_client,
					&format!("https://github.com/{repo_name}"),
				)
				.await
				.ok()
				.flatten();

				let info = if let Some(meta) = metadata {
					RepoInfo {
						repo_name,
						embedded_at: Some(meta.embedded_at.to_rfc3339()),
						doc_count: Some(meta.doc_count),
					}
				} else {
					RepoInfo {
						repo_name,
						embedded_at: None,
						doc_count: None,
					}
				};

				repo_info.push(info);
			}
		}

		// sort repositories by name
		repo_info.sort_by(|a, b| a.repo_name.cmp(&b.repo_name));

		let json_output = serde_json::to_string_pretty(&repo_info)
			.context("failed to serialize repo info")
			.map_err(BackendError::Internal)?;

		Ok(CallToolResult::success(vec![Content::text(json_output)]))
	}
}

#[tool(tool_box)]
impl ServerHandler for Backend {
	fn get_info(&self) -> ServerInfo {
		ServerInfo {
			protocol_version: ProtocolVersion::V_2024_11_05,
			capabilities: ServerCapabilities::builder().enable_tools().build(),
			server_info: Implementation {
				name: "mcp-rust-docs-embed".to_string(),
				version: "0.1.0".to_string(),
			},
			instructions: Some(
				"MCP server for Git repository documentation embedding and search"
					.to_string(),
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

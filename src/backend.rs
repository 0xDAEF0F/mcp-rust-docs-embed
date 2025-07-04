use crate::{
	error::BackendError,
	github_processor::process_and_embed_github_repo,
	query::QueryService,
	utils::{gen_table_name_without_version, resolve_crate_github_repo},
};
use anyhow::{Context, Result};
use rmcp::{
	Error as McpError, RoleServer, ServerHandler,
	model::{Content, *},
	schemars::{self, JsonSchema},
	service::RequestContext,
	tool,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

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
	#[schemars(description = "Crate name to embed docs for")]
	pub crate_name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryRequest {
	#[schemars(description = "Query to search for in the embedded docs")]
	pub query: String,
	#[schemars(description = "Crate name to search in")]
	pub crate_name: String,
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
	pub crate_name: String,
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
	pub fn new(cancellation_token: CancellationToken) -> Self {
		Self {
			cancellation_token,
			..Default::default()
		}
	}

	#[tool(
		description = "Generate and embed documentation for a given crate from its \
		               GitHub repository"
	)]
	async fn embed_crate(
		&self,
		#[tool(aggr)] req: EmbedRequest,
	) -> Result<CallToolResult, McpError> {
		tracing::info!("Starting embed_crate for crate: {}", req.crate_name);
		let operation_id = format!("embed_{}_{}", req.crate_name, Uuid::new_v4());
		tracing::debug!("Generated operation ID: {}", operation_id);
		let ops = self.embed_operations.clone();
		let ct = self.cancellation_token.child_token();

		// Resolve crate to GitHub repo URL
		tracing::info!("Resolving crate {} to GitHub repository", req.crate_name);
		let repo_url = resolve_crate_github_repo(&req.crate_name)
			.await
			.map_err(|e| {
				tracing::error!(
					"Failed to resolve crate {} to GitHub repo: {}",
					req.crate_name,
					e
				);
				McpError::invalid_request(
					format!("Failed to resolve crate to GitHub repository: {}", e),
					None,
				)
			})?;
		tracing::info!("Resolved {} to repository: {}", req.crate_name, repo_url);

		// Check if this repo is already embedded
		let table_name = gen_table_name_without_version(&req.crate_name);
		tracing::debug!("Generated table name: {}", table_name);

		tracing::info!("Checking if {} is already embedded", req.crate_name);
		if let Ok(qdrant_url) = dotenvy::var("QDRANT_URL")
			&& let Ok(qdrant_client) = qdrant_client::Qdrant::from_url(&qdrant_url)
				.api_key(dotenvy::var("QDRANT_API_KEY").ok())
				.build() && let Ok(exists) = qdrant_client.collection_exists(&table_name).await
			&& exists
		{
			tracing::info!("Crate {} is already embedded, skipping", req.crate_name);
			return Ok(CallToolResult::success(vec![Content::text(format!(
				"Documentation for {} is already embedded from repository: {}",
				req.crate_name, repo_url
			))]));
		}
		tracing::info!(
			"Crate {} not found in embeddings, proceeding with embedding",
			req.crate_name
		);

		{
			tracing::debug!("Acquiring write lock for operations tracking");
			let mut ops_lock = ops.write().await;
			tracing::info!(
				"Registering operation {} for crate {}",
				operation_id,
				req.crate_name
			);
			ops_lock.insert(
				operation_id.clone(),
				EmbedOperation {
					status: EmbedStatus::InProgress,
					crate_name: req.crate_name.clone(),
					repo_url: repo_url.to_string(),
					message: "Starting repository processing and embedding".to_string(),
				},
			);
		}

		let op_id_clone = operation_id.clone();
		let crate_name = req.crate_name.clone();

		tokio::spawn(async move {
			tracing::info!(
				"Spawning background task for embedding {} (operation: {})",
				crate_name,
				op_id_clone
			);
			let result = tokio::select! {
				_ = ct.cancelled() => {
					tracing::warn!("Operation {} cancelled for crate {}", op_id_clone, crate_name);
					Err(anyhow::anyhow!("Operation cancelled"))
				}
				res = async {
					// Process GitHub repository and embed it
					tracing::info!("Starting GitHub repository processing for {}", crate_name);
					let embed_result = process_and_embed_github_repo(&crate_name).await;
					match &embed_result {
						Ok(_) => tracing::info!("Successfully processed repository for {}", crate_name),
						Err(e) => tracing::error!("Failed to process repository for {}: {}", crate_name, e),
					}
					embed_result
				} => res
			};

			tracing::debug!("Updating operation status for {}", op_id_clone);
			let mut ops_lock = ops.write().await;
			if let Some(op) = ops_lock.get_mut(&op_id_clone) {
				match result {
					Ok(_) => {
						op.status = EmbedStatus::Completed;
						op.message = format!(
							"Successfully processed and embedded repository for {}",
							op.crate_name
						);
						tracing::info!(
							"Operation {} completed successfully for {}",
							op_id_clone,
							op.crate_name
						);
					}
					Err(e) => {
						op.status = EmbedStatus::Failed;
						op.message = format!("Failed to embed repository: {}", e);
						tracing::error!(
							"Operation {} failed for {}: {}",
							op_id_clone,
							op.crate_name,
							e
						);
					}
				}
			} else {
				tracing::warn!("Operation {} not found in tracking map", op_id_clone);
			}
		});

		tracing::info!(
			"Embed operation {} started for crate {}",
			operation_id,
			req.crate_name
		);
		Ok(CallToolResult::success(vec![Content::text(format!(
			"Started repository processing and embedding with ID: {}. Sleep for about 6 \
			 seconds and then Use \"check_embed_status\" to monitor progress --- do \
			 this until it either succeeds or fails.",
			operation_id
		))]))
	}

	#[tool(
		description = "Perform semantic search on a crates' documentation vector \
		               embeddings"
	)]
	async fn query_embeddings(
		&self,
		#[tool(aggr)] req: QueryRequest,
	) -> Result<CallToolResult, McpError> {
		// Check if embeddings exist for this crate
		let table_name = gen_table_name_without_version(&req.crate_name);
		if let Ok(qdrant_url) = dotenvy::var("QDRANT_URL")
			&& let Ok(qdrant_client) = qdrant_client::Qdrant::from_url(&qdrant_url)
				.api_key(dotenvy::var("QDRANT_API_KEY").ok())
				.build()
		{
			match qdrant_client.collection_exists(&table_name).await {
				Ok(exists) => {
					if !exists {
						return Err(BackendError::NoEmbeddedDocs {
							crate_name: req.crate_name.clone(),
							version: "repo".to_string(),
						}
						.into());
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
			.query_embeddings_without_version(&req.query, &req.crate_name, req.limit)
			.await
			.context("failed to query embeddings")
			.map_err(BackendError::Internal)?;

		if results.is_empty() {
			return Err(BackendError::NoQueryResults(req.query.clone()).into());
		}

		let header = format!(
			"Found {} results for query: {} (from {} repository)",
			results.len(),
			req.query,
			req.crate_name
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
					req.operation_id, op.crate_name, status_text, op.message
				))]))
			}
			None => Err(BackendError::OperationNotFound(req.operation_id.clone()).into()),
		}
	}

	#[tool(
		description = "List the crates and its versions/features that are already \
		               embedded in the mcp server"
	)]
	async fn list_embedded_crates(&self) -> Result<CallToolResult, McpError> {
		#[derive(Serialize)]
		struct CrateVersionInfo {
			version: String,
			features: Option<Vec<String>>,
			embedded_at: Option<String>,
			doc_count: Option<usize>,
		}

		let mut crate_info: HashMap<String, Vec<CrateVersionInfo>> = HashMap::new();

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

			// parse collection name to extract crate name
			// format is: repo_{crate_name}
			if name.starts_with("repo_") {
				let crate_name = &name[5..];

				// convert back underscores to original characters
				let crate_name = crate_name.replace('_', "-");

				// Try to get metadata for this collection
				let metadata =
					crate::data_store::DataStore::get_metadata_without_version(
						&qdrant_client,
						&crate_name,
					)
					.await
					.ok()
					.flatten();

				let info = if let Some(meta) = metadata {
					CrateVersionInfo {
						version: "repo".to_string(),
						features: None,
						embedded_at: Some(meta.embedded_at.to_rfc3339()),
						doc_count: Some(meta.doc_count),
					}
				} else {
					CrateVersionInfo {
						version: "repo".to_string(),
						features: None,
						embedded_at: None,
						doc_count: None,
					}
				};

				crate_info.entry(crate_name).or_default().push(info);
			}
		}

		// sort versions for each crate
		for versions in crate_info.values_mut() {
			versions.sort_by(|a, b| a.version.cmp(&b.version));
		}

		let json_output = serde_json::to_string_pretty(&crate_info)
			.context("failed to serialize crate info")
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

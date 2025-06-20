use crate::{
	error::BackendError,
	features::get_crate_features,
	services::{generate_and_embed_docs, query::QueryService},
	utils::{gen_table_name, resolve_latest_crate_version},
};
use anyhow::{Context, Result};
use rmcp::{
	Error as McpError, RoleServer, ServerHandler,
	model::{Content, *},
	schemars::{self, JsonSchema},
	service::RequestContext,
	tool,
};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use tap::TapFallible;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::error;

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
	10
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

	#[tool(description = "Generate and embed documentation for a given crate")]
	async fn embed_docs(
		&self,
		#[tool(aggr)] req: EmbedRequest,
	) -> Result<CallToolResult, McpError> {
		let operation_id = format!("embed_{}_{}", req.crate_name, uuid::Uuid::new_v4());
		let ops = self.embed_operations.clone();
		let ct = self.cancellation_token.child_token();

		let version = if req.version.is_empty() || req.version == "*" {
			resolve_latest_crate_version(&req.crate_name)
				.await
				.map_err(|_| {
					BackendError::VersionResolutionFailed(req.crate_name.clone())
				})?
		} else {
			req.version.clone()
		};

		// check if this version is already embedded
		let table_name = gen_table_name(&req.crate_name, &version);

		if let Ok(qdrant_url) = dotenvy::var("QDRANT_URL")
			&& let Ok(qdrant_client) =
				qdrant_client::Qdrant::from_url(&qdrant_url).build()
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
					// fetch all available features for the crate
					let features = get_crate_features(&crate_name, Some(&version_clone))
						.await
						.tap_err(|e| error!("Warning: Could not fetch features for {} v{}: {}", crate_name, version_clone, e))
						.unwrap_or_default();

					// generate documentation and embed it directly
					generate_and_embed_docs(
						&crate_name,
						&version_clone,
						&features,
					).await
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
			"Started documentation generation and embedding process with ID: {}. Sleep \
			 for about 6 seconds and then Use \"check_embed_status\" to monitor \
			 progress --- do this until it either suceeds or fails.",
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
		let version = if req.version.is_empty() || req.version == "*" {
			match resolve_latest_crate_version(&req.crate_name).await {
				Ok(v) => v,
				Err(_) => {
					return Err(BackendError::VersionResolutionFailed(
						req.crate_name.clone(),
					)
					.into());
				}
			}
		} else {
			req.version
		};

		// check if embeddings exist for this crate/version
		let table_name = gen_table_name(&req.crate_name, &version);
		if let Ok(qdrant_url) = dotenvy::var("QDRANT_URL")
			&& let Ok(qdrant_client) =
				qdrant_client::Qdrant::from_url(&qdrant_url).build()
		{
			match qdrant_client.collection_exists(&table_name).await {
				Ok(exists) => {
					if !exists {
						return Err(BackendError::NoEmbeddedDocs {
							crate_name: req.crate_name.clone(),
							version: version.clone(),
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
			.query_embeddings(&req.query, &req.crate_name, &version, req.limit)
			.await
			.context("failed to query embeddings")
			.map_err(BackendError::Internal)?;

		if results.is_empty() {
			return Err(BackendError::NoQueryResults(req.query.clone()).into());
		}

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

		match op_data {
			Some(op) => {
				let status_text = match &op.status {
					EmbedStatus::InProgress => "in_progress",
					EmbedStatus::Completed => "completed",
					EmbedStatus::Failed => "failed",
				};

				Ok(CallToolResult::success(vec![Content::text(format!(
					"Embed operation {} for {} v{}: {} - {}",
					req.operation_id, op.crate_name, op.version, status_text, op.message
				))]))
			}
			None => Err(BackendError::OperationNotFound(req.operation_id.clone()).into()),
		}
	}

	#[tool(
		description = "List all the crates and versions that are already embedded in \
		               the mcp server"
	)]
	async fn list_embedded_crates(&self) -> Result<CallToolResult, McpError> {
		let mut crate_versions: HashMap<String, Vec<String>> = HashMap::new();

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

			// parse collection name to extract crate name and version
			// format is: {crate_name}_v{version}
			if let Some(v_pos) = name.rfind("_v") {
				let crate_name = &name[..v_pos];
				let version = &name[v_pos + 2..];

				// convert back underscores to original characters
				let crate_name = crate_name.replace('_', "-");
				let version = version.replace('_', ".");

				crate_versions.entry(crate_name).or_default().push(version);
			}
		}

		// sort versions for each crate
		for versions in crate_versions.values_mut() {
			versions.sort();
		}

		let json_output = serde_json::to_string_pretty(&crate_versions)
			.context("failed to serialize crate versions")
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

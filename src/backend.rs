use crate::{
	documentation::generate_and_embed_docs,
	error::BackendError,
	features::get_crate_features,
	query::QueryService,
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
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

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
	#[serde(default)]
	#[schemars(description = "Features to enable for documentation generation")]
	pub features: Vec<String>,
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
	#[schemars(description = "Number of results to return (defaults to 10)")]
	pub limit: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StatusRequest {
	#[schemars(description = "Operation ID to check status for")]
	pub operation_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FeaturesRequest {
	#[schemars(description = "Crate name to query features for")]
	pub crate_name: String,
	#[serde(default = "default_version")]
	#[schemars(description = "Crate version (defaults to *, i.e., latest)")]
	pub version: String,
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
	async fn embed_crate(
		&self,
		#[tool(aggr)] req: EmbedRequest,
	) -> Result<CallToolResult, McpError> {
		let operation_id = format!("embed_{}_{}", req.crate_name, Uuid::new_v4());
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

		// validate features against available features for the crate
		let available_features = get_crate_features(&req.crate_name, Some(&version))
			.await
			.map_err(|e| {
				McpError::invalid_request(
					format!("Failed to fetch available features: {}", e),
					None,
				)
			})?;

		// check if all requested features are valid
		let invalid_features: Vec<_> = req
			.features
			.iter()
			.filter(|f| !available_features.contains(f))
			.cloned()
			.collect();

		if !invalid_features.is_empty() {
			return Err(McpError::invalid_request(
				format!(
					"Invalid features for {} v{}: {:?}. Available features: {:?}",
					req.crate_name, version, invalid_features, available_features
				),
				None,
			));
		}

		// check if this version is already embedded with the same features
		let table_name = gen_table_name(&req.crate_name, &version);

		if let Ok(qdrant_url) = dotenvy::var("QDRANT_URL")
			&& let Ok(qdrant_client) = qdrant_client::Qdrant::from_url(&qdrant_url)
				.api_key(dotenvy::var("QDRANT_API_KEY").ok())
				.build() && let Ok(exists) = qdrant_client.collection_exists(&table_name).await
			&& exists
		{
			// Check if the existing embedding has the same features
			if let Ok(Some(metadata)) = crate::data_store::DataStore::get_metadata(
				&qdrant_client,
				&req.crate_name,
				&version,
			)
			.await
			{
				let mut existing_features = metadata.features.clone();
				let mut requested_features = req.features.clone();
				existing_features.sort();
				requested_features.sort();

				if existing_features == requested_features {
					return Ok(CallToolResult::success(vec![Content::text(format!(
						"Documentation for {} v{} is already embedded with features: \
						 {:?}",
						req.crate_name, version, existing_features
					))]));
				} else {
					return Err(McpError::invalid_request(
						format!(
							"Documentation for {} v{} already exists with different \
							 features. Existing: {:?}, Requested: {:?}. Delete the \
							 existing collection first if you want to re-embed with \
							 different features.",
							req.crate_name,
							version,
							existing_features,
							requested_features
						),
						None,
					));
				}
			} else {
				// No metadata found, this is likely an old embedding without feature
				// tracking
				return Ok(CallToolResult::success(vec![Content::text(format!(
					"Documentation for {} v{} is already embedded (legacy embedding \
					 without feature tracking)",
					req.crate_name, version
				))]));
			}
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
		let features = req.features.clone();

		tokio::spawn(async move {
			let result = tokio::select! {
				_ = ct.cancelled() => {
					Err(anyhow::anyhow!("Operation cancelled"))
				}
				res = async {
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

		// check if embeddings exist for this crate/version and get metadata
		let table_name = gen_table_name(&req.crate_name, &version);
		let mut features_info = None;
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
							version: version.clone(),
						}
						.into());
					}
					// get metadata to show features
					if let Ok(Some(metadata)) =
						crate::data_store::DataStore::get_metadata(
							&qdrant_client,
							&req.crate_name,
							&version,
						)
						.await
					{
						features_info = Some(metadata.features);
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

		let header = if let Some(features) = features_info {
			format!(
				"Found {} results for query: {} (from {} v{} with features: [{}])",
				results.len(),
				req.query,
				req.crate_name,
				version,
				features.join(", ")
			)
		} else {
			format!(
				"Found {} results for query: {} (from {} v{})",
				results.len(),
				req.query,
				req.crate_name,
				version
			)
		};

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
					"Embed operation {} for {} v{}: {} - {}",
					req.operation_id, op.crate_name, op.version, status_text, op.message
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

			// parse collection name to extract crate name and version
			// format is: {crate_name}_v{version}
			if let Some(v_pos) = name.rfind("_v") {
				let crate_name = &name[..v_pos];
				let version = &name[v_pos + 2..];

				// convert back underscores to original characters
				let crate_name = crate_name.replace('_', "-");
				let version = version.replace('_', ".");

				// Try to get metadata for this collection
				let metadata = crate::data_store::DataStore::get_metadata(
					&qdrant_client,
					&crate_name,
					&version,
				)
				.await
				.ok()
				.flatten();

				let info = if let Some(meta) = metadata {
					CrateVersionInfo {
						version: version.clone(),
						features: Some(meta.features),
						embedded_at: Some(meta.embedded_at.to_rfc3339()),
						doc_count: Some(meta.doc_count),
					}
				} else {
					CrateVersionInfo {
						version: version.clone(),
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

	#[tool(description = "Query available features for a specific crate and version")]
	async fn query_crate_features(
		&self,
		#[tool(aggr)] req: FeaturesRequest,
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
			req.version.clone()
		};

		let features = get_crate_features(&req.crate_name, Some(&version))
			.await
			.map_err(|e| {
				McpError::invalid_request(
					format!("Failed to fetch features: {}", e),
					None,
				)
			})?;

		let feature_count = features.len();
		let features_json = serde_json::to_string_pretty(&features)
			.context("failed to serialize features")
			.map_err(BackendError::Internal)?;

		Ok(CallToolResult::success(vec![Content::text(format!(
			"Features for {} v{} ({} total):\n{}",
			req.crate_name, version, feature_count, features_json
		))]))
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

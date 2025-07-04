use crate::{config::EmbeddingConfig, utils::gen_table_name_for_repo};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use qdrant_client::{
	Payload, Qdrant,
	qdrant::{
		CreateCollectionBuilder, Distance, GetPointsBuilder, PointStruct,
		SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
	},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, trace};

/// Metadata stored with each embedding collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
	pub repo_url: String,
	pub embedded_at: DateTime<Utc>,
	pub embedding_model: String,
	pub doc_count: usize,
}

pub struct DataStore {
	pub qdrant_client: Qdrant,
	repo_url: String,
	collection_name: String,
}

impl DataStore {
	/// Initialize a new data store for repository-based embedding
	pub async fn new(repo_url: &str) -> Result<Self> {
		let qdrant_url = dotenvy::var("QDRANT_URL").context("QDRANT_URL not set")?;
		let qdrant_api_key = dotenvy::var("QDRANT_API_KEY").ok();

		let qdrant_client = Qdrant::from_url(&qdrant_url)
			.api_key(qdrant_api_key)
			.build()?;

		// Generate deterministic names
		let collection_name = gen_table_name_for_repo(repo_url)?;

		// setup qdrant collection - only create if it doesn't exist
		let collection_exists = qdrant_client.collection_exists(&collection_name).await?;
		if !collection_exists {
			let embedding_config = EmbeddingConfig::default();
			let collection = CreateCollectionBuilder::new(&collection_name)
				.vectors_config(VectorParamsBuilder::new(
					embedding_config.vector_size,
					Distance::Cosine,
				));

			let res = qdrant_client.create_collection(collection).await?;
			assert!(res.result, "collection could not be created");
		}

		Ok(Self {
			qdrant_client,
			repo_url: repo_url.to_string(),
			collection_name,
		})
	}

	/// Reset the Qdrant collection
	pub async fn reset(&self) -> Result<()> {
		self.qdrant_client
			.delete_collection(&self.collection_name)
			.await?;

		let embedding_config = EmbeddingConfig::default();
		let collection =
			CreateCollectionBuilder::new(&self.collection_name).vectors_config(
				VectorParamsBuilder::new(embedding_config.vector_size, Distance::Cosine),
			);

		_ = self.qdrant_client.create_collection(collection).await?;

		Ok(())
	}

	/// Add embedding data with content to Qdrant
	pub async fn add_embedding_with_content(
		&self,
		content: &str,
		vector: Vec<f32>,
	) -> Result<u64> {
		// generate a unique id based on timestamp and random value
		let id = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)?
			.as_nanos() as u64;

		// create payload with the content
		let payload = Payload::try_from(json!({
			"content": content
		}))?;

		// add vector and content to qdrant
		let points = vec![PointStruct::new(id, vector, payload)];
		let req = UpsertPointsBuilder::new(&self.collection_name, points);
		self.qdrant_client.upsert_points(req).await?;

		Ok(id)
	}

	/// Query embeddings and return the corresponding text content
	pub async fn query_with_content(
		&self,
		query_vector: Vec<f32>,
		max_results: u64,
	) -> Result<Vec<(f32, String)>> {
		let search_req =
			SearchPointsBuilder::new(&self.collection_name, query_vector, max_results)
				.with_payload(true);
		let search_res = self.qdrant_client.search_points(search_req).await?;

		let mut results = Vec::new();

		for result in search_res.result {
			let score = result.score;

			let Some(content) = result.payload.get("content") else {
				trace!(
					"skipping result that does not have a content field (probably \
					 metadata)"
				);
				continue;
			};
			let content = content
				.as_str()
				.context("could not convert the content `Value` into a `String`")?
				.to_owned();

			results.push((score, content));
		}

		Ok(results)
	}

	/// Store metadata for the collection
	pub async fn store_metadata(&self, doc_count: usize) -> Result<()> {
		let metadata = EmbeddingMetadata {
			repo_url: self.repo_url.clone(),
			embedded_at: Utc::now(),
			embedding_model: "text-embedding-3-small".to_string(),
			doc_count,
		};

		debug!("Storing metadata: {:?}", metadata);

		// Store metadata as a special point with ID 0
		let payload = Payload::try_from(json!({
			"metadata": serde_json::to_value(&metadata)?,
			"is_metadata": true
		}))?;

		debug!("Storing metadata in collection: {}", self.collection_name);

		let points = vec![PointStruct::new(0, vec![0.0; 1536], payload)];
		let req = UpsertPointsBuilder::new(&self.collection_name, points);
		self.qdrant_client.upsert_points(req).await?;

		Ok(())
	}

	/// Retrieve metadata for a repository
	pub async fn get_metadata(
		qdrant_client: &Qdrant,
		repo_url: &str,
	) -> Result<Option<EmbeddingMetadata>> {
		let collection_name = gen_table_name_for_repo(repo_url)?;

		// Try to get the metadata point (ID 0)
		let get_points = GetPointsBuilder::new(collection_name, vec![0.into()])
			.with_payload(true)
			.build();

		match qdrant_client.get_points(get_points).await {
			Ok(response) => {
				if let Some(point) = response.result.first()
					&& let Some(metadata_value) = point.payload.get("metadata")
				{
					let metadata: EmbeddingMetadata =
						serde_json::from_value(metadata_value.clone().into())?;
					return Ok(Some(metadata));
				}
				Ok(None)
			}
			Err(_) => Ok(None),
		}
	}
}

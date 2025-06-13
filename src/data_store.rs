use crate::{config::EmbeddingConfig, utils::gen_table_name};
use anyhow::{Context, Result};
use qdrant_client::{
	Payload, Qdrant,
	qdrant::{
		CreateCollectionBuilder, Distance, PointStruct, SearchPointsBuilder,
		UpsertPointsBuilder, VectorParamsBuilder,
	},
};
use serde_json::json;

pub struct DataStore {
	pub qdrant_client: Qdrant,
	crate_name: String,
	version: String,
}

impl DataStore {
	/// Initialize a new data store with Qdrant
	pub async fn try_new(crate_name: &str, version: &str) -> Result<Self> {
		let qdrant_url = dotenvy::var("QDRANT_URL").context("QDRANT_URL not set")?;

		let qdrant_client = Qdrant::from_url(&qdrant_url).build()?;

		// Generate deterministic names
		let collection_name = gen_table_name(crate_name, version);

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
			crate_name: crate_name.to_string(),
			version: version.to_string(),
		})
	}

	/// Reset the Qdrant collection
	pub async fn reset(&self) -> Result<()> {
		let collection_name = gen_table_name(&self.crate_name, &self.version);

		self.qdrant_client
			.delete_collection(&collection_name)
			.await?;

		let embedding_config = EmbeddingConfig::default();
		let collection = CreateCollectionBuilder::new(&collection_name).vectors_config(
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
		let collection_name = gen_table_name(&self.crate_name, &self.version);

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
		let req = UpsertPointsBuilder::new(&collection_name, points);
		self.qdrant_client.upsert_points(req).await?;

		Ok(id)
	}

	/// Query embeddings and return the corresponding text content
	pub async fn query_with_content(
		&self,
		query_vector: Vec<f32>,
		max_results: u64,
	) -> Result<Vec<(f32, String)>> {
		let collection_name = gen_table_name(&self.crate_name, &self.version);

		let search_req =
			SearchPointsBuilder::new(&collection_name, query_vector, max_results)
				.with_payload(true);
		let search_res = self.qdrant_client.search_points(search_req).await?;

		let mut results = Vec::new();

		for result in search_res.result {
			let score = result.score;

			// extract content from payload
			let content = result
				.payload
				.get("content")
				.and_then(|v| v.as_str())
				.ok_or_else(|| anyhow::anyhow!("missing content in payload"))?
				.to_string();

			results.push((score, content));
		}

		Ok(results)
	}
}

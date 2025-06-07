use anyhow::Result;
use qdrant_client::{
	Payload, Qdrant,
	qdrant::{
		CreateCollectionBuilder, Distance, PointStruct, QueryPointsBuilder,
		SearchPointsBuilder, SearchResponse, UpsertPointsBuilder, VectorParamsBuilder,
	},
};

/// Qdrant wrapper
pub struct QdrantW {
	pub qdrant_client: Qdrant,
}

impl QdrantW {
	pub async fn try_new(collection_name: &str) -> Result<Self> {
		let url = dotenvy::var("QDRANT_URL")?;
		let client = Qdrant::from_url(&url).build()?;

		// 1. delete the collection if it exists
		_ = client.delete_collection(collection_name).await;

		// 2. create the collection again from scratch
		let collection = CreateCollectionBuilder::new(collection_name)
			.vectors_config(VectorParamsBuilder::new(1024, Distance::Cosine));

		let res = client.create_collection(collection).await?;

		assert!(res.result, "collection could not be created");

		Ok(Self {
			qdrant_client: client,
		})
	}

	pub async fn add_embedding(
		&self,
		collection_name: &str,
		id: u64,
		vector: Vec<f32>,
	) -> Result<()> {
		let points = vec![PointStruct::new(id, vector, Payload::default())];

		let req = UpsertPointsBuilder::new(collection_name, points);

		_ = self.qdrant_client.upsert_points(req).await?;

		Ok(())
	}

	pub async fn query(
		&self,
		collection_name: &str,
		query: Vec<f32>,
		max_results: u64,
	) -> Result<SearchResponse> {
		let search_req = SearchPointsBuilder::new(collection_name, query, max_results);
		let res = self.qdrant_client.search_points(search_req).await?;
		Ok(res)
	}
}

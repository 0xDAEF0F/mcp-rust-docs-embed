use anyhow::Result;
use qdrant_client::{
	Payload, Qdrant,
	qdrant::{
		CreateCollectionBuilder, Distance, PointStruct, UpsertPointsBuilder,
		VectorParamsBuilder,
	},
};

const COLL: &str = "collection_a";

/// Qdrant wrapper
pub struct QdrantW {
	pub qdrant_client: Qdrant,
}

impl QdrantW {
	pub async fn try_new() -> Result<Self> {
		let url = dotenvy::var("QDRANT_URL")?;
		let client = Qdrant::from_url(&url).build()?;

		if !client.collection_exists(COLL).await? {
			let collection = CreateCollectionBuilder::new(COLL)
				.vectors_config(VectorParamsBuilder::new(1024, Distance::Cosine));
			let res = client.create_collection(collection).await?;
			assert!(res.result, "collection could not be created");
		}

		Ok(Self {
			qdrant_client: client,
		})
	}

	// todo: handle many vectors at once
	pub async fn add_embedding(&self, id: u64, vector: Vec<f32>) -> Result<()> {
		let points = vec![PointStruct::new(id, vector, Payload::default())];

		let req = UpsertPointsBuilder::new(COLL, points);

		_ = self.qdrant_client.upsert_points(req).await?;

		Ok(())
	}
}

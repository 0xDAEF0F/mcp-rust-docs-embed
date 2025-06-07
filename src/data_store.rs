use anyhow::Result;
use qdrant_client::{
	Payload, Qdrant,
	qdrant::{
		CreateCollectionBuilder, Distance, PointStruct, SearchPointsBuilder,
		SearchResponse, UpsertPointsBuilder, VectorParamsBuilder,
		point_id::PointIdOptions,
	},
};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

/// General data store combining Qdrant vector database and SQLite
pub struct DataStore {
	pub qdrant_client: Qdrant,
	pub sql_pool: SqlitePool,
}

impl DataStore {
	/// Initialize a new data store with both Qdrant and SQLite connections
	pub async fn try_new(collection_name: &str, sqlite_path: &str) -> Result<Self> {
		let url = dotenvy::var("QDRANT_URL")?;
		let qdrant_client = Qdrant::from_url(&url).build()?;

		// setup qdrant collection
		_ = qdrant_client.delete_collection(collection_name).await;

		let collection = CreateCollectionBuilder::new(collection_name)
			.vectors_config(VectorParamsBuilder::new(1024, Distance::Cosine));

		let res = qdrant_client.create_collection(collection).await?;
		assert!(res.result, "collection could not be created");

		// setup sqlite connection
		let sql_pool = SqlitePoolOptions::new().connect(sqlite_path).await?;

		Ok(Self {
			qdrant_client,
			sql_pool,
		})
	}

	/// Reset both the SQLite table and Qdrant collection
	pub async fn reset(&self, collection_name: &str, table_name: &str) -> Result<()> {
		// reset sqlite
		let query = format!("DELETE FROM {table_name}");
		sqlx::query(&query).execute(&self.sql_pool).await?;

		// reset qdrant (already done in try_new, but keeping for completeness)
		_ = self.qdrant_client.delete_collection(collection_name).await;

		let collection = CreateCollectionBuilder::new(collection_name)
			.vectors_config(VectorParamsBuilder::new(1024, Distance::Cosine));

		let res = self.qdrant_client.create_collection(collection).await?;
		assert!(res.result, "collection could not be recreated");

		Ok(())
	}

	/// Add embedding data to both SQLite (text content) and Qdrant (vector)
	pub async fn add_embedding_with_content(
		&self,
		collection_name: &str,
		table_name: &str,
		content: &str,
		vector: Vec<f32>,
	) -> Result<u64> {
		// insert content into sqlite and get the row id
		let query = format!("INSERT INTO {table_name} (contents) VALUES (?1)");
		let row_id = sqlx::query(&query)
			.bind(content)
			.execute(&self.sql_pool)
			.await?
			.last_insert_rowid();

		// add vector to qdrant using the sqlite row id
		let points = vec![PointStruct::new(row_id as u64, vector, Payload::default())];
		let req = UpsertPointsBuilder::new(collection_name, points);
		self.qdrant_client.upsert_points(req).await?;

		Ok(row_id as u64)
	}

	/// Query embeddings and return the corresponding text content
	pub async fn query_with_content(
		&self,
		collection_name: &str,
		table_name: &str,
		query_vector: Vec<f32>,
		max_results: u64,
	) -> Result<Vec<(f32, String)>> {
		// search in qdrant
		let search_req =
			SearchPointsBuilder::new(collection_name, query_vector, max_results);
		let search_res = self.qdrant_client.search_points(search_req).await?;

		let mut results = Vec::new();

		// get corresponding text content from sqlite
		for result in search_res.result {
			let score = result.score;

			let point_id = result
				.id
				.expect("expected id")
				.point_id_options
				.expect("no point id");

			let PointIdOptions::Num(n) = point_id else {
				anyhow::bail!("expected numeric point id");
			};

			let query = format!("SELECT contents FROM {table_name} WHERE id = ?");
			let row = sqlx::query_scalar::<_, String>(&query)
				.bind(n as i64)
				.fetch_one(&self.sql_pool)
				.await?;

			results.push((score, row));
		}

		Ok(results)
	}

	/// Add embedding to Qdrant only (legacy method for compatibility)
	pub async fn add_embedding(
		&self,
		collection_name: &str,
		id: u64,
		vector: Vec<f32>,
	) -> Result<()> {
		let points = vec![PointStruct::new(id, vector, Payload::default())];
		let req = UpsertPointsBuilder::new(collection_name, points);
		self.qdrant_client.upsert_points(req).await?;
		Ok(())
	}
}

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

// Root type - only keep fields we actually use
#[derive(Debug, Deserialize)]
pub struct JsonDocs {
	pub index: HashMap<String, Item>,
	// Skip all other fields we don't need
	#[serde(flatten)]
	_other: HashMap<String, Value>,
}

// Item type - only keep fields we actually use
#[derive(Debug, Deserialize)]
pub struct Item {
	pub crate_id: u32,
	pub name: Option<String>,
	pub docs: Option<String>,
	pub span: Option<Span>,
	pub inner: HashMap<String, Value>,
	// Skip all other fields we don't need
	#[serde(flatten)]
	_other: HashMap<String, Value>,
}

impl Item {
	pub fn item_type(&self) -> Option<&str> {
		self.inner.keys().next().map(|s| s.as_str())
	}
}

// Span type - only keep fields we actually use
#[derive(Debug, Deserialize, Clone)]
pub struct Span {
	pub filename: String,
	pub begin: (u32, u32),
	pub end: (u32, u32),
	// Skip other fields we don't need
	#[serde(flatten)]
	_other: HashMap<String, Value>,
}

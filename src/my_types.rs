use crate::json_types::{Item, JsonDocs};

#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
	Struct,
	Enum,
	Function,
	Constant,
	Impl,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileRange {
	pub start: (u32, u32),
	pub end: (u32, u32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocItem {
	pub name: Option<String>,
	pub doc_string: Option<String>,
	pub r#type: ItemType,
	pub file_range: FileRange,
	pub filename: String,
}

impl From<&Item> for Option<DocItem> {
	fn from(item: &Item) -> Self {
		let item_type = match item.item_type()? {
			"struct" => ItemType::Struct,
			"enum" => ItemType::Enum,
			"function" => ItemType::Function,
			"constant" => ItemType::Constant,
			"impl" => ItemType::Impl,
			_ => return None,
		};

		let span = item.span.as_ref()?;

		Some(DocItem {
			name: item.name.clone(),
			doc_string: item.docs.clone(),
			r#type: item_type,
			file_range: FileRange {
				start: span.begin,
				end: span.end,
			},
			filename: span.filename.clone(),
		})
	}
}

impl From<&JsonDocs> for Vec<DocItem> {
	fn from(docs: &JsonDocs) -> Self {
		docs.index
			.values()
			.filter(|item| {
				item.crate_id == 0
					&& item.span.is_some()
					&& !matches!(
						item.item_type(),
						Some("struct_field") | Some("variant") | Some("module")
					)
			})
			.filter_map(|item| item.into())
			.collect()
	}
}

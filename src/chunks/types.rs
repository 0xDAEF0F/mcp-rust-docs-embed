#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Chunk {
   pub kind: ChunkKind,
   pub start_line: usize,
   pub end_line: usize,
   pub content: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChunkKind {
   Struct,
   Enum,
   Function,
   Impl,
   Comment,
   MarkdownSection,
   // TypeScript-specific
   Class,
   Interface,
   TypeAlias,
   Const,
}

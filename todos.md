# todos

- refresh repos after x time has passed since they where embedded
- embed markdown files, too
- integrate `typescript-treesitter` into the embedding process

## chore

- `embedded_crates` should just return this schema:
- instead of embed crate lets change it to embed repo

```json
{
  "owner/repo": {
    "url": "https://...",
    "embedding_count": 42,
    "main_language": "rust|typescript|go|python",
    "embedded_at": "2023-05-01T00:00:00Z"
  },
  ...
}
```

## Run

### 1. Prepare you `rubberband` binary

Put it in `./rubberband` or `./rubberband.exe` (for Windows)

### 2. Run

```bash
cargo run -r
```

## Vibe

This project is vibe-coded by the collaboration of GLM-5 and Claude Opus 4.6

- It's obvious that LLM has no enough knowledge about `iced`. It hallucinates nonexistent APIs. The solution is to pull the complete `iced` repo for LLM to reference, and guide it to reference the examples as well as the source code of `iced`. This apply to other libraries.
- It's interesting that GLM-5 occasionally outperforms Claude Opus 4.6.
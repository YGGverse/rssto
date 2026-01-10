# rssto-llm

LLM daemon for the rssto DB translations

> [!NOTE]
> In development!

1. Setup `rssto-crawler` first and collect initial data

2. Run LLM server:

```
llama-server -hf ggml-org/gemma-3-1b-it-GGUF
```

3. Launch `rssto-llm` to handle `content` DB:

```
cd rssto/crates/rssto-llm
cargo run -- -c /path/to/config.toml
```
* see `--help` to display all supported options
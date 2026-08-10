# DeepSeek V4 tokenizer

- Source: https://huggingface.co/deepseek-ai/DeepSeek-V4-Pro/blob/main/tokenizer.json
- Publisher: `deepseek-ai`
- SHA-256: `8f9f37ca37fdc4f5fd36d5cf4d3b0e8392edb4e894fd10cc0d70b4957c8633cf`

The tokenizer is bundled into the Rust binary with `include_bytes!` so token
auditing never silently falls back to an unrelated encoding.

`test_input_1.json`, `test_output_1.txt`, `test_input_2.json`, and
`test_output_2.txt` come from the model repository's official `encoding/tests`
directory and are used as byte-for-byte message-template fixtures.

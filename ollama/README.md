# Ollama

Local LLMs on the RX 9070 XT (gfx1201) through `ollama-rocm` and ROCm from Arch `extra`.

| Repo file | Installed at | How |
| --- | --- | --- |
| [`system/etc/systemd/system/ollama.service.d/override.conf`](../system/etc/systemd/system/ollama.service.d/override.conf) | `/etc/systemd/system/ollama.service.d/override.conf` | `make system`, then `sudo systemctl daemon-reload && sudo systemctl restart ollama` |
| [`Modelfile`](Modelfile) | Ollama's model store, as `gpt-oss-64k` | `ollama create gpt-oss-64k -f ollama/Modelfile` |

The override keeps the API on `127.0.0.1`, disables cloud models, and sets a 64k context with a
`q8_0` KV cache and flash attention so `gpt-oss:20b` fits fully on the 16 GB GPU (`ollama ps`:
100% GPU, 12 GB).

Other models pulled: `gemma4:12b-it-qat`, `nomic-embed-text`.

## Benchmark

`ollama run gpt-oss-64k --verbose "Explain Wake-on-LAN in three sentences"` on 2026-11-07,
ollama-rocm 0.40.14, ROCm 7.2.4:

| Metric | Value |
| --- | --- |
| Eval rate | 101.93 tokens/s (398 tokens) |
| Prompt eval rate | 682.36 tokens/s (75 tokens) |
| Total duration | 4.02 s |

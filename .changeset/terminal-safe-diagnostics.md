---
amm: fix
---

# Escape terminal controls in CLI diagnostics

Neutralize control characters in every command error at the stderr rendering boundary, including failed simulation logs and RPC errors. Under `--json`, command errors emit one JSON object on stderr with an `error` string.

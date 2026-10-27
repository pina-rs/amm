---
amm: user
---

# Read RPC failures safely

## User impact

RPC errors and failed simulation logs show terminal control characters as visible text, so an endpoint cannot use diagnostics to erase terminal output or manipulate the clipboard. Scripts using `--json` can read command failures as an `error` object on stderr.

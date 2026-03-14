# Trading Bot Execution (Server 2)

High-performance trade execution engine written in Rust. Polls the Brain server (Server 1) for trade signals and executes them on Bitget.

## Architecture

```
Brain (Server 1)  ──WireGuard──▶  Execution (Server 2)  ──HTTPS──▶  Bitget API
   10.0.0.1                          10.0.0.2
```

## Configuration

Copy `config/default.toml` to `config/local.toml` and fill in your API keys:

```bash
cp config/default.toml config/local.toml
# Edit config/local.toml with your credentials
```

Environment variables override config files (prefix `EXEC__`):
```bash
EXEC__BITGET__API_KEY=xxx
EXEC__BRAIN__API_KEY=xxx
```

## Build & Run

```bash
cargo build --release
./target/release/trading-bot-execution
```

## Docker

```bash
docker build -t trading-bot-execution .
docker run -v ./config/local.toml:/app/config/local.toml trading-bot-execution
```

## Dry-Run Mode

Enabled by default (`execution.dry_run = true`). Set to `false` for live trading.

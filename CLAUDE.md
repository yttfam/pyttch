# pyttch

Fire-and-forget Telegram message sender. Library + CLI. No bot loop, no polling, no handlers.

## What You Do

Send a message to a Telegram chat. That's it.

```bash
# CLI - argument
pyttch --token BOT_TOKEN --chat-id 12345 "hello"

# CLI - stdin
echo "deploy done" | pyttch --token BOT_TOKEN --chat-id 12345

# CLI - env vars
export TELEGRAM_BOT_TOKEN=...
export TELEGRAM_CHAT_ID=...
pyttch "hello"
```

```rust
// Library
pyttch::send("BOT_TOKEN", chat_id, "hello")?;
```

## Architecture

One file could do it. But keep it clean:

- `lib.rs` — `send()` function, that's the public API
- `main.rs` — CLI wrapper (clap args + stdin fallback)

## Dependencies

- `ureq` — HTTP client (sync, no async runtime)
- `serde` — JSON serialization
- `clap` — CLI args (derive, optional via `cli` feature)
- `anyhow` — errors

No async. No tokio. No teloxide. No state.

## CLI Args

```
pyttch [OPTIONS] [MESSAGE]

Options:
  --token <TOKEN>      Bot token (or TELEGRAM_BOT_TOKEN env)
  --chat-id <ID>       Chat ID (or TELEGRAM_CHAT_ID env)
  --parse-mode <MODE>  Optional: HTML, Markdown, MarkdownV2
  -h, --help
```

MESSAGE is positional. If absent, read from stdin (for piping).

## Send Function Signature

```rust
pub fn send(token: &str, chat_id: i64, message: &str) -> anyhow::Result<()>
pub fn send_with_parse_mode(token: &str, chat_id: i64, message: &str, parse_mode: ParseMode) -> anyhow::Result<()>
```

Keep the simple version simple. Parse mode is opt-in.

## Build

- Shared target dir: `.cargo/config.toml` → `target-dir = "/Users/cali/Developer/perso/ttyfam/target"`
- Cross-compile targets: same as apytti (macOS arm64, Linux musl x86_64, Windows x86_64 gnu)
- Linux cross-linker: `x86_64-linux-musl-gcc` (installed via musl-cross)

## Tests

- Unit test: validate CLI arg parsing, stdin fallback logic
- Integration test: mock or skip (don't spam Telegram in CI). Gate behind `#[ignore]` or a feature flag.

## What You DON'T Do

- No bot loop / long polling / webhook listening
- No message receiving
- No conversation state
- No inline keyboards, media, or anything fancy
- No retry logic (caller can retry)

## Family

Part of the YTT family. Other members call you when they need to notify:

- **apytti** (`../apytti`) — REST gateway for Claude Code. Might use you for error alerts.
- **grytti** (`../grytti`) — has its own Telegram bot (bidirectional). You're the one-way little sibling.
- **shytti** (`../shytti`) — shell orchestrator. Pipe output to you.

## Cali's Preferences

- Rust, no unsafe
- Small binary, fast startup
- Ship it, iterate
- MIT license

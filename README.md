# pyttch

Telegram message sender and listener. Library + CLI. Sync, minimal, no framework.

## CLI

```bash
# Direct message
pyttch --token BOT_TOKEN --chat-id 12345 "hello"

# Pipe from stdin
echo "deploy done" | pyttch --token BOT_TOKEN --chat-id 12345

# With env vars
export TELEGRAM_BOT_TOKEN=...
export TELEGRAM_CHAT_ID=...
pyttch "hello"

# With parse mode
pyttch --parse-mode HTML "<b>bold</b> message"
```

### Options

```
pyttch [OPTIONS] [MESSAGE]

Options:
  --token <TOKEN>            Bot token (or TELEGRAM_BOT_TOKEN env)
  --chat-id <CHAT_ID>        Chat ID (or TELEGRAM_CHAT_ID env)
  --parse-mode <PARSE_MODE>  HTML, Markdown, or MarkdownV2
  -h, --help                 Print help
```

If `MESSAGE` is omitted, reads from stdin.

## Library

Add to your `Cargo.toml`:

```toml
[dependencies]
pyttch = { version = "0.1", default-features = false }
```

`default-features = false` gives you just the library without the CLI binary and its clap dependency.

### Send

```rust
// Simple
pyttch::send("BOT_TOKEN", chat_id, "hello")?;

// With parse mode
use pyttch::ParseMode;
pyttch::send_with_parse_mode("BOT_TOKEN", chat_id, "<b>bold</b>", ParseMode::Html)?;
```

### Menu (inline keyboard)

```rust
use pyttch::Button;

// Send a message with buttons — each vec is a row
pyttch::send_menu("BOT_TOKEN", chat_id, "Pick one:", vec![
    vec![Button::new("Option A", "a"), Button::new("Option B", "b")],
    vec![Button::new("Cancel", "cancel")],
])?;
```

### Listen

```rust
use pyttch::{Listener, Content};

let mut listener = Listener::new("BOT_TOKEN", chat_id);

// Iterator — blocks forever, yields messages one by one
for msg in &mut listener {
    match &msg.content {
        Content::Text(text) => println!("{text}"),
        Content::Photo { caption } => println!("photo: {}", caption.as_deref().unwrap_or("")),
        Content::Document { file_name, .. } => println!("file: {}", file_name.as_deref().unwrap_or("?")),
        Content::Sticker { emoji } => println!("sticker {}", emoji.as_deref().unwrap_or("")),
        Content::Voice { .. } => println!("voice message"),
        Content::Video { caption } => println!("video: {}", caption.as_deref().unwrap_or("")),
        Content::CallbackQuery { data, callback_id } => {
            println!("button pressed: {data}");
            pyttch::answer_callback("BOT_TOKEN", callback_id)?;
        }
        Content::Other => println!("unsupported message type"),
    }
}

// Or manual polling
loop {
    let messages = listener.poll()?;
    for msg in messages {
        if let Some(text) = msg.text_or_caption() {
            println!("{text}");
        }
    }
}
```

The listener uses Telegram's `getUpdates` long-polling (30s timeout by default). Filters by chat ID, tracks offset automatically.

Supported content types: text, photo, document, sticker, voice, video, callback query (button press). Anything else comes through as `Content::Other`.

Call `answer_callback()` after handling a `CallbackQuery` to dismiss the button's loading state.

Convenience methods on `Message`:
- `text()` — returns `Some(&str)` for text messages only
- `text_or_caption()` — returns text or caption for any type that has one

```rust
// Custom timeout
let mut listener = Listener::new("BOT_TOKEN", chat_id).with_timeout(60);
```

All calls are synchronous. No async runtime needed.

## Install

```bash
cargo install pyttch
```

## Build from source

```bash
cargo build --release
```

Release binary is ~1.9 MB (stripped, LTO).

### Cross-compile

```bash
# Linux x86_64
cargo build --release --target x86_64-unknown-linux-musl
```

## License

MIT

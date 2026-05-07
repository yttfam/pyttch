# pyttch CLI

Fire-and-forget Telegram sender. One invocation = one API call.

## Synopsis

```
pyttch --token <TOKEN> --chat-id <ID> [--parse-mode <MODE>] [MESSAGE]
pyttch --token <TOKEN> --chat-id <ID> --typing
```

## Arguments

| Name | Kind | Required | Env | Description |
|------|------|----------|-----|-------------|
| `--token` | option | yes | `TELEGRAM_BOT_TOKEN` | Bot token from BotFather. |
| `--chat-id` | option (i64) | yes | `TELEGRAM_CHAT_ID` | Target chat ID. Negative for groups/channels. |
| `--parse-mode` | option | no | — | One of: `HTML`, `Markdown`, `MarkdownV2` (case-insensitive). Mutually exclusive with `--typing`. |
| `--typing` | flag | no | — | Send a `sendChatAction: typing` ping instead of a message. Lasts ~5s on Telegram side. Mutually exclusive with `--parse-mode` and `MESSAGE`. |
| `MESSAGE` | positional | no | — | Message body. If omitted, read from stdin (trimmed; empty = error). Mutually exclusive with `--typing`. |

## Modes

### Send (default)

Calls Telegram `sendMessage`. Exits 0 on HTTP 200, non-zero otherwise.

```bash
pyttch --token "$T" --chat-id 12345 "hello"
echo "deploy done" | pyttch --token "$T" --chat-id 12345
pyttch --token "$T" --chat-id 12345 --parse-mode HTML "<b>bold</b>"
```

### Typing indicator

Calls Telegram `sendChatAction` with `action=typing`. The indicator auto-expires after ~5s; re-fire every ~4s while waiting on long work.

```bash
pyttch --token "$T" --chat-id 12345 --typing
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (HTTP 200 from Telegram). |
| non-zero | Bad args, stdin read failure, network error, or non-200 response. Stderr contains the error. |

## Environment Variables

- `TELEGRAM_BOT_TOKEN` — fallback for `--token`.
- `TELEGRAM_CHAT_ID` — fallback for `--chat-id`.

CLI flags override env vars.

## Constraints

- No retries. Caller retries on failure.
- No state, no daemon, no polling. One process = one API call.
- Stdin is read only when `MESSAGE` is absent and `--typing` is not set.
- Empty stdin (after trim) is an error, not a no-op.

## Non-goals

Receiving messages, conversation state, webhooks, inline keyboards via CLI, media uploads/downloads. Library exposes some of these (`send_menu`, `Listener`, `answer_callback`, `download_file`); CLI does not.

## Library-only APIs (not on CLI)

For consumers (e.g. the apytti bridge) — these are Rust functions, not CLI flags:

| Function | Purpose |
|----------|---------|
| `pyttch::send_returning_id(token, chat_id, text) -> i64` | Same as `send` but returns the Telegram `message_id` for later editing. |
| `pyttch::send_returning_id_with_parse_mode(...)` | Same as above with `ParseMode`. |
| `pyttch::send_menu(token, chat_id, text, rows)` | Send message with inline keyboard buttons. |
| `pyttch::edit_message(token, chat_id, message_id, text)` | Edit a previously sent message via `editMessageText`. Caller owns `message_id`. |
| `pyttch::edit_message_with_parse_mode(...)` | Same as above with `ParseMode`. |
| `pyttch::answer_callback(token, callback_id)` | Ack a button press to clear the spinner. |
| `pyttch::download_file(token, file_id) -> Vec<u8>` | Fetch the bytes of a media file. Two HTTP calls (`getFile` → file URL). |
| `pyttch::Listener::new(token, chat_id)` | Blocking long-poll listener. `poll()` or iterate. |
| `Message::file_id()` | Returns `file_id` for `Photo`/`Document`/`Voice`/`Video`/`Sticker` content; pass to `download_file`. |

pub mod listen;

use anyhow::bail;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::Read;

pub use listen::{Content, Listener, Message};

const API_BASE: &str = "https://api.telegram.org/bot";

#[derive(Debug, Clone, Copy)]
pub enum ParseMode {
    Html,
    Markdown,
    MarkdownV2,
}

impl ParseMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ParseMode::Html => "HTML",
            ParseMode::Markdown => "Markdown",
            ParseMode::MarkdownV2 => "MarkdownV2",
        }
    }
}

#[derive(Serialize, Debug)]
struct SendMessage<'a> {
    chat_id: i64,
    text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<&'a str>,
}

pub fn send(token: &str, chat_id: i64, message: &str) -> anyhow::Result<()> {
    send_inner(token, chat_id, message, None).map(|_| ())
}

pub fn send_with_parse_mode(
    token: &str,
    chat_id: i64,
    message: &str,
    parse_mode: ParseMode,
) -> anyhow::Result<()> {
    send_inner(token, chat_id, message, Some(parse_mode)).map(|_| ())
}

/// Same as `send`, but returns the Telegram `message_id` so the caller can later
/// `edit_message` it. Use this when you want to mutate the message later (e.g. a
/// "working..." status that gets edited as progress arrives).
pub fn send_returning_id(token: &str, chat_id: i64, message: &str) -> anyhow::Result<i64> {
    send_inner(token, chat_id, message, None)
}

/// Same as `send_with_parse_mode`, but returns the Telegram `message_id`.
pub fn send_returning_id_with_parse_mode(
    token: &str,
    chat_id: i64,
    message: &str,
    parse_mode: ParseMode,
) -> anyhow::Result<i64> {
    send_inner(token, chat_id, message, Some(parse_mode))
}

/// A button in an inline keyboard menu.
/// `label` is what the user sees, `data` is what you get back in the callback.
#[derive(Debug, Clone, Serialize)]
pub struct Button {
    text: String,
    callback_data: String,
}

impl Button {
    pub fn new(label: &str, data: &str) -> Self {
        Self {
            text: label.to_string(),
            callback_data: data.to_string(),
        }
    }
}

#[derive(Serialize)]
struct SendMessageWithKeyboard<'a> {
    chat_id: i64,
    text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<&'a str>,
    reply_markup: InlineKeyboardMarkup,
}

#[derive(Serialize)]
struct InlineKeyboardMarkup {
    inline_keyboard: Vec<Vec<Button>>,
}

/// Send a message with inline keyboard buttons.
/// Each inner vec is a row of buttons.
pub fn send_menu(
    token: &str,
    chat_id: i64,
    message: &str,
    rows: Vec<Vec<Button>>,
) -> anyhow::Result<()> {
    let url = format!("{API_BASE}{token}/sendMessage");
    let body = SendMessageWithKeyboard {
        chat_id,
        text: message,
        parse_mode: None,
        reply_markup: InlineKeyboardMarkup {
            inline_keyboard: rows,
        },
    };

    let mut resp = ureq::post(&url).send_json(&body)?;

    if resp.status() != 200 {
        let status = resp.status();
        let text = resp.body_mut().read_to_string()?;
        bail!("telegram api error ({status}): {text}");
    }

    Ok(())
}

/// Acknowledge a callback query (button press). Call this after handling a CallbackQuery
/// or the button will show a loading spinner.
pub fn answer_callback(token: &str, callback_id: &str) -> anyhow::Result<()> {
    let url = format!("{API_BASE}{token}/answerCallbackQuery");
    let body = serde_json::json!({ "callback_query_id": callback_id });

    let mut resp = ureq::post(&url).send_json(&body)?;

    if resp.status() != 200 {
        let status = resp.status();
        let text = resp.body_mut().read_to_string()?;
        bail!("telegram api error ({status}): {text}");
    }

    Ok(())
}

#[derive(Deserialize)]
struct GetFileResponse {
    ok: bool,
    #[serde(default)]
    result: Option<GetFileResult>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
struct GetFileResult {
    file_path: String,
}

/// Download a file's bytes given its `file_id` (from `Content::Photo { file_id, .. }`, etc.).
/// Two HTTP calls: `getFile` → file URL.
pub fn download_file(token: &str, file_id: &str) -> anyhow::Result<Vec<u8>> {
    let meta_url = format!("{API_BASE}{token}/getFile");
    let body = serde_json::json!({ "file_id": file_id });

    let mut resp = ureq::post(&meta_url).send_json(&body)?;
    if resp.status() != 200 {
        let status = resp.status();
        let text = resp.body_mut().read_to_string()?;
        bail!("telegram getFile error ({status}): {text}");
    }
    let parsed: GetFileResponse = resp.body_mut().read_json()?;
    if !parsed.ok {
        bail!("telegram getFile error: {}", parsed.description.unwrap_or_default());
    }
    let file_path = parsed
        .result
        .ok_or_else(|| anyhow::anyhow!("getFile returned no result"))?
        .file_path;

    let file_url = format!("https://api.telegram.org/file/bot{token}/{file_path}");
    let mut file_resp = ureq::get(&file_url).call()?;
    if file_resp.status() != 200 {
        let status = file_resp.status();
        bail!("telegram file download error ({status})");
    }
    let mut bytes = Vec::new();
    file_resp.body_mut().as_reader().read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Send a "typing" chat action indicator.
pub fn send_typing(token: &str, chat_id: i64) -> anyhow::Result<()> {
    let url = format!("{API_BASE}{token}/sendChatAction");
    let body = serde_json::json!({ "chat_id": chat_id, "action": "typing" });

    let mut resp = ureq::post(&url).send_json(&body)?;

    if resp.status() != 200 {
        let status = resp.status();
        let text = resp.body_mut().read_to_string()?;
        bail!("telegram api error ({status}): {text}");
    }

    Ok(())
}

/// Edit the text of a previously sent message. Caller owns the `message_id`.
/// One HTTP call, no state, no retries — same shape as `send`.
pub fn edit_message(
    token: &str,
    chat_id: i64,
    message_id: i64,
    text: &str,
) -> anyhow::Result<()> {
    edit_inner(token, chat_id, message_id, text, None)
}

/// Edit the text of a previously sent message with a parse mode.
pub fn edit_message_with_parse_mode(
    token: &str,
    chat_id: i64,
    message_id: i64,
    text: &str,
    parse_mode: ParseMode,
) -> anyhow::Result<()> {
    edit_inner(token, chat_id, message_id, text, Some(parse_mode))
}

#[derive(Serialize)]
struct EditMessageText<'a> {
    chat_id: i64,
    message_id: i64,
    text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<&'a str>,
}

fn edit_inner(
    token: &str,
    chat_id: i64,
    message_id: i64,
    text: &str,
    parse_mode: Option<ParseMode>,
) -> anyhow::Result<()> {
    let url = format!("{API_BASE}{token}/editMessageText");
    let body = EditMessageText {
        chat_id,
        message_id,
        text,
        parse_mode: parse_mode.map(|pm| pm.as_str()),
    };

    let mut resp = ureq::post(&url).send_json(&body)?;

    if resp.status() != 200 {
        let status = resp.status();
        let text = resp.body_mut().read_to_string()?;
        bail!("telegram api error ({status}): {text}");
    }

    Ok(())
}

#[derive(Deserialize)]
struct SendMessageResponse {
    ok: bool,
    #[serde(default)]
    result: Option<SentMessage>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
struct SentMessage {
    message_id: i64,
}

fn send_inner(
    token: &str,
    chat_id: i64,
    message: &str,
    parse_mode: Option<ParseMode>,
) -> anyhow::Result<i64> {
    let url = format!("{API_BASE}{token}/sendMessage");
    let body = SendMessage {
        chat_id,
        text: message,
        parse_mode: parse_mode.map(|pm| pm.as_str()),
    };

    let mut resp = ureq::post(&url).send_json(&body)?;

    if resp.status() != 200 {
        let status = resp.status();
        let text = resp.body_mut().read_to_string()?;
        bail!("telegram api error ({status}): {text}");
    }

    let parsed: SendMessageResponse = resp.body_mut().read_json()?;
    if !parsed.ok {
        bail!(
            "telegram api error: {}",
            parsed.description.unwrap_or_default()
        );
    }
    Ok(parsed
        .result
        .ok_or_else(|| anyhow::anyhow!("sendMessage returned no result"))?
        .message_id)
}

use serde::Deserialize;
use serde_json::json;

use crate::API_BASE;

/// Content type of an incoming Telegram message.
#[derive(Debug, Clone)]
pub enum Content {
    Text(String),
    Photo { file_id: String, caption: Option<String> },
    Document { file_id: String, file_name: Option<String>, caption: Option<String> },
    Sticker { file_id: String, emoji: Option<String> },
    Voice { file_id: String, caption: Option<String> },
    Video { file_id: String, caption: Option<String> },
    /// A button press from an inline keyboard. `data` is the callback_data you set on the button.
    CallbackQuery { data: String, callback_id: String },
    Other,
}

/// An incoming Telegram message.
#[derive(Debug, Clone)]
pub struct Message {
    pub message_id: i64,
    pub chat_id: i64,
    pub content: Content,
    pub from: Option<String>,
    pub date: i64,
}

impl Message {
    /// Convenience: returns the text if this is a text message.
    pub fn text(&self) -> Option<&str> {
        match &self.content {
            Content::Text(t) => Some(t),
            _ => None,
        }
    }

    /// Returns caption for media types, or the text for text messages.
    pub fn text_or_caption(&self) -> Option<&str> {
        match &self.content {
            Content::Text(t) => Some(t),
            Content::Photo { caption, .. } => caption.as_deref(),
            Content::Document { caption, .. } => caption.as_deref(),
            Content::Voice { caption, .. } => caption.as_deref(),
            Content::Video { caption, .. } => caption.as_deref(),
            _ => None,
        }
    }

    /// Returns the Telegram `file_id` for media content, or `None` for text/callback/other.
    /// Pass to `pyttch::download_file` to fetch the bytes.
    pub fn file_id(&self) -> Option<&str> {
        match &self.content {
            Content::Photo { file_id, .. }
            | Content::Document { file_id, .. }
            | Content::Sticker { file_id, .. }
            | Content::Voice { file_id, .. }
            | Content::Video { file_id, .. } => Some(file_id),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
struct GetUpdatesResponse {
    ok: bool,
    result: Vec<Update>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
struct Update {
    update_id: i64,
    #[serde(default)]
    message: Option<TgMessage>,
    #[serde(default)]
    callback_query: Option<TgCallbackQuery>,
}

#[derive(Deserialize)]
struct TgCallbackQuery {
    id: String,
    from: User,
    #[serde(default)]
    message: Option<TgCallbackMessage>,
    #[serde(default)]
    data: Option<String>,
}

#[derive(Deserialize)]
struct TgCallbackMessage {
    message_id: i64,
    chat: Chat,
    date: i64,
}

#[derive(Deserialize)]
struct TgMessage {
    message_id: i64,
    chat: Chat,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    caption: Option<String>,
    #[serde(default)]
    photo: Option<Vec<TgPhotoSize>>,
    #[serde(default)]
    document: Option<TgDocument>,
    #[serde(default)]
    sticker: Option<TgSticker>,
    #[serde(default)]
    voice: Option<TgFile>,
    #[serde(default)]
    video: Option<TgFile>,
    #[serde(default)]
    from: Option<User>,
    date: i64,
}

#[derive(Deserialize)]
struct TgDocument {
    file_id: String,
    #[serde(default)]
    file_name: Option<String>,
}

#[derive(Deserialize)]
struct TgSticker {
    file_id: String,
    #[serde(default)]
    emoji: Option<String>,
}

#[derive(Deserialize)]
struct TgPhotoSize {
    file_id: String,
}

#[derive(Deserialize)]
struct TgFile {
    file_id: String,
}

#[derive(Deserialize)]
struct Chat {
    id: i64,
}

#[derive(Deserialize)]
struct User {
    first_name: String,
    #[serde(default)]
    last_name: Option<String>,
}

fn extract_content(msg: &mut TgMessage) -> Content {
    if let Some(text) = msg.text.take() {
        return Content::Text(text);
    }
    if let Some(photos) = msg.photo.take() {
        // Telegram returns photo sizes ascending; largest is last.
        if let Some(largest) = photos.into_iter().last() {
            return Content::Photo {
                file_id: largest.file_id,
                caption: msg.caption.take(),
            };
        }
    }
    if let Some(doc) = msg.document.take() {
        return Content::Document {
            file_id: doc.file_id,
            file_name: doc.file_name,
            caption: msg.caption.take(),
        };
    }
    if let Some(sticker) = msg.sticker.take() {
        return Content::Sticker {
            file_id: sticker.file_id,
            emoji: sticker.emoji,
        };
    }
    if let Some(voice) = msg.voice.take() {
        return Content::Voice {
            file_id: voice.file_id,
            caption: msg.caption.take(),
        };
    }
    if let Some(video) = msg.video.take() {
        return Content::Video {
            file_id: video.file_id,
            caption: msg.caption.take(),
        };
    }
    Content::Other
}

fn extract_from(user: Option<User>) -> Option<String> {
    user.map(|u| match u.last_name {
        Some(last) => format!("{} {last}", u.first_name),
        None => u.first_name,
    })
}

/// Parse a raw `getUpdates` JSON response, filtering by chat_id.
pub fn parse_updates(json: &str, chat_id: i64) -> anyhow::Result<Vec<Message>> {
    let (messages, _) = parse_updates_with_offset(json, chat_id)?;
    Ok(messages)
}

/// Parse a raw `getUpdates` JSON response, filtering by chat_id.
/// Returns matching messages and the next offset.
pub fn parse_updates_with_offset(
    json: &str,
    chat_id: i64,
) -> anyhow::Result<(Vec<Message>, i64)> {
    let parsed: GetUpdatesResponse = serde_json::from_str(json)?;

    if !parsed.ok {
        anyhow::bail!(
            "telegram api error: {}",
            parsed.description.unwrap_or_default()
        );
    }

    let mut messages = Vec::new();
    let mut offset = 0i64;

    for update in parsed.result {
        offset = update.update_id + 1;

        if let Some(cb) = update.callback_query {
            if let (Some(data), Some(cb_msg)) = (cb.data, cb.message) {
                if cb_msg.chat.id != chat_id {
                    continue;
                }
                messages.push(Message {
                    message_id: cb_msg.message_id,
                    chat_id: cb_msg.chat.id,
                    content: Content::CallbackQuery {
                        data,
                        callback_id: cb.id,
                    },
                    from: extract_from(Some(cb.from)),
                    date: cb_msg.date,
                });
            }
        } else if let Some(mut msg) = update.message {
            if msg.chat.id != chat_id {
                continue;
            }
            let content = extract_content(&mut msg);
            let from = extract_from(msg.from);
            messages.push(Message {
                message_id: msg.message_id,
                chat_id: msg.chat.id,
                content,
                from,
                date: msg.date,
            });
        }
    }

    Ok((messages, offset))
}

/// Blocking long-poll listener for a single Telegram chat.
pub struct Listener {
    url: String,
    chat_id: i64,
    offset: i64,
    timeout: u64,
    client: ureq::Agent,
}

impl Listener {
    /// Create a new listener for the given bot token and chat ID.
    /// Polls with a 30-second long-poll timeout by default.
    pub fn new(token: &str, chat_id: i64) -> Self {
        Self {
            url: format!("{API_BASE}{token}/getUpdates"),
            chat_id,
            offset: 0,
            timeout: 30,
            client: ureq::Agent::new_with_config(
                ureq::config::Config::builder()
                    .timeout_global(Some(std::time::Duration::from_secs(35)))
                    .build(),
            ),
        }
    }

    pub fn chat_id(&self) -> i64 {
        self.chat_id
    }

    /// Set the long-poll timeout in seconds.
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self.client = ureq::Agent::new_with_config(
            ureq::config::Config::builder()
                .timeout_global(Some(std::time::Duration::from_secs(timeout + 5)))
                .build(),
        );
        self
    }

    /// Block and return the next batch of messages. May return an empty vec on timeout.
    pub fn poll(&mut self) -> anyhow::Result<Vec<Message>> {
        let body = json!({
            "offset": self.offset,
            "timeout": self.timeout,
            "allowed_updates": ["message", "callback_query"],
        });

        let mut resp = self.client.post(&self.url).send_json(&body)?;
        let raw = resp.body_mut().read_to_string()?;
        let (messages, offset) = parse_updates_with_offset(&raw, self.chat_id)?;

        if offset > 0 {
            self.offset = offset;
        }

        Ok(messages)
    }
}

impl Iterator for &mut Listener {
    type Item = Message;

    fn next(&mut self) -> Option<Message> {
        loop {
            match self.poll() {
                Ok(msgs) => {
                    for msg in msgs {
                        return Some(msg);
                    }
                    // empty poll (timeout), loop again
                }
                Err(_) => return None,
            }
        }
    }
}

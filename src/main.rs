use anyhow::{bail, Context};
use clap::Parser;
use std::io::{self, Read};
use pyttch::ParseMode;

#[derive(Parser)]
#[command(name = "pyttch", about = "Fire-and-forget Telegram message sender")]
struct Cli {
    /// Bot token (or TELEGRAM_BOT_TOKEN env)
    #[arg(long, env = "TELEGRAM_BOT_TOKEN")]
    token: String,

    /// Chat ID (or TELEGRAM_CHAT_ID env)
    #[arg(long, env = "TELEGRAM_CHAT_ID")]
    chat_id: i64,

    /// Parse mode: HTML, Markdown, MarkdownV2
    #[arg(long)]
    parse_mode: Option<String>,

    /// Send a "typing..." chat action instead of a message (lasts ~5s)
    #[arg(long, conflicts_with_all = ["parse_mode", "message"])]
    typing: bool,

    /// Message to send (reads from stdin if absent)
    message: Option<String>,
}

fn parse_parse_mode(s: &str) -> anyhow::Result<ParseMode> {
    match s.to_lowercase().as_str() {
        "html" => Ok(ParseMode::Html),
        "markdown" => Ok(ParseMode::Markdown),
        "markdownv2" => Ok(ParseMode::MarkdownV2),
        other => bail!("unknown parse mode: {other}"),
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.typing {
        pyttch::send_typing(&cli.token, cli.chat_id)?;
        return Ok(());
    }

    let message = match cli.message {
        Some(msg) => msg,
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .context("failed to read stdin")?;
            let trimmed = buf.trim().to_string();
            if trimmed.is_empty() {
                bail!("no message provided (pass as argument or pipe to stdin)");
            }
            trimmed
        }
    };

    match cli.parse_mode {
        Some(pm) => {
            let mode = parse_parse_mode(&pm)?;
            pyttch::send_with_parse_mode(&cli.token, cli.chat_id, &message, mode)?;
        }
        None => {
            pyttch::send(&cli.token, cli.chat_id, &message)?;
        }
    }

    Ok(())
}

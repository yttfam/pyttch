use pyttch::ParseMode;

#[test]
fn parse_mode_strings() {
    assert_eq!(ParseMode::Html.as_str(), "HTML");
    assert_eq!(ParseMode::Markdown.as_str(), "Markdown");
    assert_eq!(ParseMode::MarkdownV2.as_str(), "MarkdownV2");
}

#[test]
fn send_bad_token_fails() {
    let result = pyttch::send("bad-token", 12345, "hello");
    assert!(result.is_err());
}

#[test]
fn send_with_parse_mode_bad_token_fails() {
    let result = pyttch::send_with_parse_mode("bad-token", 12345, "hello", ParseMode::Html);
    assert!(result.is_err());
}

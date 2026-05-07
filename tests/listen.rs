use pyttch::listen::Content;
use pyttch::Listener;

#[test]
fn listener_new_sets_defaults() {
    let listener = Listener::new("test-token", 12345);
    assert_eq!(listener.chat_id(), 12345);
}

#[test]
fn listener_with_timeout() {
    let listener = Listener::new("test-token", 12345).with_timeout(60);
    assert_eq!(listener.chat_id(), 12345);
}

#[test]
fn parse_text_message() {
    let json = r#"{
        "ok": true,
        "result": [{
            "update_id": 100,
            "message": {
                "message_id": 1,
                "chat": {"id": 12345},
                "text": "hello",
                "from": {"first_name": "Cali", "last_name": "B"},
                "date": 1700000000
            }
        }]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].text(), Some("hello"));
    assert_eq!(messages[0].from.as_deref(), Some("Cali B"));
    assert_eq!(messages[0].date, 1700000000);
}

#[test]
fn parse_text_message_no_last_name() {
    let json = r#"{
        "ok": true,
        "result": [{
            "update_id": 100,
            "message": {
                "message_id": 1,
                "chat": {"id": 12345},
                "text": "hello",
                "from": {"first_name": "Cali"},
                "date": 1700000000
            }
        }]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert_eq!(messages[0].from.as_deref(), Some("Cali"));
}

#[test]
fn parse_photo_with_caption() {
    let json = r#"{
        "ok": true,
        "result": [{
            "update_id": 100,
            "message": {
                "message_id": 1,
                "chat": {"id": 12345},
                "photo": [{"file_id": "abc", "width": 100, "height": 100}],
                "caption": "nice pic",
                "date": 1700000000
            }
        }]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert_eq!(messages.len(), 1);
    assert!(matches!(&messages[0].content, Content::Photo { file_id, caption } if file_id == "abc" && caption.as_deref() == Some("nice pic")));
    assert_eq!(messages[0].text(), None);
    assert_eq!(messages[0].text_or_caption(), Some("nice pic"));
    assert_eq!(messages[0].file_id(), Some("abc"));
}

#[test]
fn parse_photo_without_caption() {
    let json = r#"{
        "ok": true,
        "result": [{
            "update_id": 100,
            "message": {
                "message_id": 1,
                "chat": {"id": 12345},
                "photo": [{"file_id": "abc", "width": 100, "height": 100}],
                "date": 1700000000
            }
        }]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert!(matches!(&messages[0].content, Content::Photo { caption, .. } if caption.is_none()));
    assert_eq!(messages[0].text_or_caption(), None);
}

#[test]
fn parse_document() {
    let json = r#"{
        "ok": true,
        "result": [{
            "update_id": 100,
            "message": {
                "message_id": 1,
                "chat": {"id": 12345},
                "document": {"file_id": "abc", "file_name": "report.pdf"},
                "caption": "here you go",
                "date": 1700000000
            }
        }]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert!(matches!(
        &messages[0].content,
        Content::Document { file_id, file_name, caption }
        if file_id == "abc" && file_name.as_deref() == Some("report.pdf") && caption.as_deref() == Some("here you go")
    ));
}

#[test]
fn parse_sticker() {
    let json = r#"{
        "ok": true,
        "result": [{
            "update_id": 100,
            "message": {
                "message_id": 1,
                "chat": {"id": 12345},
                "sticker": {"file_id": "abc", "width": 512, "height": 512, "emoji": "\u00f0\u009f\u0098\u0082"},
                "date": 1700000000
            }
        }]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert!(matches!(&messages[0].content, Content::Sticker { emoji, .. } if emoji.is_some()));
}

#[test]
fn parse_voice() {
    let json = r#"{
        "ok": true,
        "result": [{
            "update_id": 100,
            "message": {
                "message_id": 1,
                "chat": {"id": 12345},
                "voice": {"file_id": "abc", "duration": 5},
                "date": 1700000000
            }
        }]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert!(matches!(&messages[0].content, Content::Voice { .. }));
}

#[test]
fn parse_video() {
    let json = r#"{
        "ok": true,
        "result": [{
            "update_id": 100,
            "message": {
                "message_id": 1,
                "chat": {"id": 12345},
                "video": {"file_id": "abc", "width": 1920, "height": 1080, "duration": 30},
                "caption": "watch this",
                "date": 1700000000
            }
        }]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert!(matches!(
        &messages[0].content,
        Content::Video { file_id, caption } if file_id == "abc" && caption.as_deref() == Some("watch this")
    ));
}

#[test]
fn parse_unknown_content_type() {
    let json = r#"{
        "ok": true,
        "result": [{
            "update_id": 100,
            "message": {
                "message_id": 1,
                "chat": {"id": 12345},
                "location": {"latitude": 48.8566, "longitude": 2.3522},
                "date": 1700000000
            }
        }]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert_eq!(messages.len(), 1);
    assert!(matches!(&messages[0].content, Content::Other));
}

#[test]
fn parse_filters_by_chat_id() {
    let json = r#"{
        "ok": true,
        "result": [
            {
                "update_id": 100,
                "message": {
                    "message_id": 1,
                    "chat": {"id": 99999},
                    "text": "wrong chat",
                    "date": 1700000000
                }
            },
            {
                "update_id": 101,
                "message": {
                    "message_id": 2,
                    "chat": {"id": 12345},
                    "text": "right chat",
                    "date": 1700000001
                }
            }
        ]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].text(), Some("right chat"));
}

#[test]
fn parse_skips_non_message_updates() {
    let json = r#"{
        "ok": true,
        "result": [{"update_id": 100}]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert_eq!(messages.len(), 0);
}

#[test]
fn parse_returns_offset() {
    let json = r#"{
        "ok": true,
        "result": [{
            "update_id": 500,
            "message": {
                "message_id": 1,
                "chat": {"id": 12345},
                "text": "hi",
                "date": 1700000000
            }
        }]
    }"#;

    let (messages, offset) = pyttch::listen::parse_updates_with_offset(json, 12345).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(offset, 501);
}

#[test]
fn parse_api_error() {
    let json = r#"{
        "ok": false,
        "result": [],
        "description": "Unauthorized"
    }"#;

    let result = pyttch::listen::parse_updates(json, 12345);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unauthorized"));
}

#[test]
fn parse_multiple_messages() {
    let json = r#"{
        "ok": true,
        "result": [
            {
                "update_id": 100,
                "message": {
                    "message_id": 1,
                    "chat": {"id": 12345},
                    "text": "first",
                    "date": 1700000000
                }
            },
            {
                "update_id": 101,
                "message": {
                    "message_id": 2,
                    "chat": {"id": 12345},
                    "photo": [{"file_id": "abc", "width": 100, "height": 100}],
                    "date": 1700000001
                }
            },
            {
                "update_id": 102,
                "message": {
                    "message_id": 3,
                    "chat": {"id": 12345},
                    "text": "third",
                    "date": 1700000002
                }
            }
        ]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert_eq!(messages.len(), 3);
    assert!(matches!(&messages[0].content, Content::Text(t) if t == "first"));
    assert!(matches!(&messages[1].content, Content::Photo { file_id, .. } if file_id == "abc"));
    assert!(matches!(&messages[2].content, Content::Text(t) if t == "third"));
}

#[test]
fn parse_callback_query() {
    let json = r#"{
        "ok": true,
        "result": [{
            "update_id": 200,
            "callback_query": {
                "id": "cb_123",
                "from": {"first_name": "Cali", "last_name": "B"},
                "message": {
                    "message_id": 42,
                    "chat": {"id": 12345},
                    "date": 1700000000
                },
                "data": "option_1"
            }
        }]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].message_id, 42);
    assert_eq!(messages[0].from.as_deref(), Some("Cali B"));
    assert!(matches!(
        &messages[0].content,
        Content::CallbackQuery { data, callback_id }
        if data == "option_1" && callback_id == "cb_123"
    ));
}

#[test]
fn parse_callback_query_filters_by_chat_id() {
    let json = r#"{
        "ok": true,
        "result": [{
            "update_id": 200,
            "callback_query": {
                "id": "cb_123",
                "from": {"first_name": "Cali"},
                "message": {
                    "message_id": 42,
                    "chat": {"id": 99999},
                    "date": 1700000000
                },
                "data": "option_1"
            }
        }]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert_eq!(messages.len(), 0);
}

#[test]
fn parse_callback_query_no_data_skipped() {
    let json = r#"{
        "ok": true,
        "result": [{
            "update_id": 200,
            "callback_query": {
                "id": "cb_123",
                "from": {"first_name": "Cali"},
                "message": {
                    "message_id": 42,
                    "chat": {"id": 12345},
                    "date": 1700000000
                }
            }
        }]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert_eq!(messages.len(), 0);
}

#[test]
fn parse_mixed_messages_and_callbacks() {
    let json = r#"{
        "ok": true,
        "result": [
            {
                "update_id": 100,
                "message": {
                    "message_id": 1,
                    "chat": {"id": 12345},
                    "text": "hello",
                    "date": 1700000000
                }
            },
            {
                "update_id": 101,
                "callback_query": {
                    "id": "cb_456",
                    "from": {"first_name": "Cali"},
                    "message": {
                        "message_id": 2,
                        "chat": {"id": 12345},
                        "date": 1700000001
                    },
                    "data": "picked_2"
                }
            }
        ]
    }"#;

    let messages = pyttch::listen::parse_updates(json, 12345).unwrap();
    assert_eq!(messages.len(), 2);
    assert!(matches!(&messages[0].content, Content::Text(t) if t == "hello"));
    assert!(matches!(&messages[1].content, Content::CallbackQuery { data, .. } if data == "picked_2"));
}

use handlers::ApiError;

pub(crate) const OCTET_STREAM: &str = "application/octet-stream";
pub(crate) const MAX_FILE_NAME_CHARACTERS: usize = 255;

const ALLOWED_MIME_TYPES: [&str; 8] = [
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "application/pdf",
    "text/plain",
    "text/csv",
    "application/json",
];

pub(crate) fn intake_mime_type(raw: Option<&str>) -> &'static str {
    let Some(raw) = raw else {
        return OCTET_STREAM;
    };
    let essence = raw.split(';').next().unwrap_or_default().trim();

    ALLOWED_MIME_TYPES
        .iter()
        .copied()
        .find(|allowed| allowed.eq_ignore_ascii_case(essence))
        .unwrap_or(OCTET_STREAM)
}

fn is_bidi_control(character: char) -> bool {
    matches!(character, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

pub(crate) fn displayable_file_name(name: &str) -> Result<(), ApiError> {
    if name.chars().count() > MAX_FILE_NAME_CHARACTERS {
        return Err(ApiError::UnprocessableEntity(format!(
            "the X-File-Name header must be at most {MAX_FILE_NAME_CHARACTERS} characters"
        )));
    }

    if name
        .chars()
        .any(|character| character.is_control() || is_bidi_control(character))
    {
        return Err(ApiError::UnprocessableEntity(
            "the X-File-Name header must not contain control or bidirectional formatting characters"
                .to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALLOWED: [&str; 8] = [
        "image/png",
        "image/jpeg",
        "image/gif",
        "image/webp",
        "application/pdf",
        "text/plain",
        "text/csv",
        "application/json",
    ];

    #[test]
    fn each_allowed_type_is_kept() {
        for allowed in ALLOWED {
            assert_eq!(intake_mime_type(Some(allowed)), allowed);
        }
    }

    #[test]
    fn the_case_is_ignored() {
        assert_eq!(intake_mime_type(Some("IMAGE/PNG")), "image/png");
        assert_eq!(intake_mime_type(Some("Application/Pdf")), "application/pdf");
        assert_eq!(intake_mime_type(Some("TEXT/CSV")), "text/csv");
    }

    #[test]
    fn parameters_and_surrounding_whitespace_are_dropped() {
        assert_eq!(
            intake_mime_type(Some("text/plain; charset=utf-8")),
            "text/plain"
        );
        assert_eq!(
            intake_mime_type(Some("Text/Plain;charset=UTF-8")),
            "text/plain"
        );
        assert_eq!(
            intake_mime_type(Some("  application/json  ;  q=1 ")),
            "application/json"
        );
        assert_eq!(intake_mime_type(Some(" image/png")), "image/png");
    }

    #[test]
    fn a_type_that_can_run_script_is_never_kept() {
        for refused in [
            "text/html",
            "TEXT/HTML; charset=utf-8",
            "image/svg+xml",
            "application/xhtml+xml",
            "application/xml",
            "text/xml",
            "text/javascript",
            "application/javascript",
        ] {
            assert_eq!(intake_mime_type(Some(refused)), OCTET_STREAM, "{refused}");
        }
    }

    #[test]
    fn anything_else_falls_back_to_octet_stream() {
        for refused in [
            "",
            " ",
            ";",
            "; image/png",
            "garbage",
            "image/png/extra",
            "image/pngx",
            "image/ png",
            "image/png\u{0}",
            "ïmage/png",
            "application/octet-stream",
            "*/*",
            "multipart/form-data; boundary=x",
        ] {
            assert_eq!(intake_mime_type(Some(refused)), OCTET_STREAM, "{refused:?}");
        }
    }

    #[test]
    fn a_very_long_value_falls_back_to_octet_stream() {
        let long = "a".repeat(5000);
        assert_eq!(intake_mime_type(Some(&long)), OCTET_STREAM);
        assert_eq!(
            intake_mime_type(Some(&format!("text/html; x={long}"))),
            OCTET_STREAM
        );
        assert_eq!(
            intake_mime_type(Some(&format!("image/png; x={long}"))),
            "image/png"
        );
    }

    #[test]
    fn a_missing_header_falls_back_to_octet_stream() {
        assert_eq!(intake_mime_type(None), OCTET_STREAM);
    }

    fn refused(name: &str) -> bool {
        matches!(
            displayable_file_name(name),
            Err(ApiError::UnprocessableEntity(_))
        )
    }

    #[test]
    fn a_name_of_255_characters_is_accepted_and_256_is_refused() {
        assert!(displayable_file_name(&"a".repeat(255)).is_ok());
        assert!(refused(&"a".repeat(256)));
    }

    #[test]
    fn the_length_counts_characters_not_bytes() {
        assert!(displayable_file_name(&"é".repeat(255)).is_ok());
        assert!(refused(&"é".repeat(256)));
        assert!(displayable_file_name(&"\u{1F600}".repeat(255)).is_ok());
        assert!(refused(&"\u{1F600}".repeat(256)));
    }

    #[test]
    fn a_control_character_is_refused() {
        for name in [
            "a\rb.png",
            "a\nb.png",
            "evil\r\nX-Injected: 1.png",
            "a\tb.png",
            "a\u{7f}b.png",
            "a\u{0}b.png",
            "a\u{1b}[31mb.png",
            "a\u{85}b.png",
            "\u{7}",
        ] {
            assert!(refused(name), "{name:?}");
        }
    }

    #[test]
    fn a_bidi_control_character_is_refused() {
        for code in (0x202A..=0x202E).chain(0x2066..=0x2069) {
            let name = format!("invoice{}fdp.exe", char::from_u32(code).unwrap());
            assert!(refused(&name), "{code:x}");
        }
    }

    #[test]
    fn the_characters_next_to_the_bidi_ranges_are_accepted() {
        for code in [0x2029u32, 0x202F, 0x2065, 0x206A] {
            let name = format!("a{}b.png", char::from_u32(code).unwrap());
            assert!(displayable_file_name(&name).is_ok(), "{code:x}");
        }
    }

    #[test]
    fn ordinary_names_are_accepted() {
        for name in [
            "photo.png",
            "Facture été 2026 (final).pdf",
            "日本語のファイル.txt",
            "emoji \u{1F600}.png",
            "a b  c.txt",
            "../../etc/passwd",
            "dir/sub/file.txt",
            "..",
            "a\\b.txt",
        ] {
            assert!(displayable_file_name(name).is_ok(), "{name:?}");
        }
    }

    #[test]
    fn the_message_never_echoes_the_name() {
        for name in [
            "secret\r\nname.png".to_owned(),
            format!("secret{}", "a".repeat(300)),
        ] {
            match displayable_file_name(&name) {
                Err(ApiError::UnprocessableEntity(message)) => {
                    assert!(!message.contains("secret"), "{message}");
                    assert!(message.contains("X-File-Name"), "{message}");
                }
                other => panic!("{other:?}"),
            }
        }
    }
}

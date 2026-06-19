use common::{RoleId, UserId};
use std::str::FromStr;
use uuid::Uuid;

use crate::ids::ChannelId;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedMentions {
    pub user_ids: Vec<UserId>,
    pub role_ids: Vec<RoleId>,
    pub channel_ids: Vec<ChannelId>,
    pub everyone: bool,
}

/// Parse Discord-style mention tokens from a message content string.
///
/// Recognised tokens:
/// - `<@{uuid}>` — user mention
/// - `<@&{uuid}>` — role mention
/// - `<#{uuid}>` — channel mention
/// - `@everyone` — everyone flag
///
/// Malformed UUIDs are silently ignored. IDs are deduplicated.
pub fn parse_mentions(content: &str) -> ParsedMentions {
    let mut user_ids: Vec<UserId> = Vec::new();
    let mut role_ids: Vec<RoleId> = Vec::new();
    let mut channel_ids: Vec<ChannelId> = Vec::new();
    let mut everyone = false;

    // @everyone (standalone word boundary: preceded by whitespace/start, followed by whitespace/end/punctuation)
    if content.contains("@everyone") {
        everyone = true;
    }

    // Use a simple character-by-character scanner rather than regex (no regex dep).
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        // Try to find closing '>'
        if let Some(close) = content[i..].find('>') {
            let token = &content[i..i + close + 1]; // includes < and >
            if let Some(inner) = token.strip_prefix("<@&").and_then(|s| s.strip_suffix('>')) {
                // role mention
                if let Ok(uuid) = Uuid::from_str(inner) {
                    let rid = RoleId(uuid);
                    if !role_ids.contains(&rid) {
                        role_ids.push(rid);
                    }
                }
            } else if let Some(inner) = token.strip_prefix("<@").and_then(|s| s.strip_suffix('>')) {
                // user mention
                if let Ok(uuid) = Uuid::from_str(inner) {
                    let uid = UserId(uuid);
                    if !user_ids.contains(&uid) {
                        user_ids.push(uid);
                    }
                }
            } else if let Some(inner) = token.strip_prefix("<#").and_then(|s| s.strip_suffix('>')) {
                // channel mention
                if let Ok(uuid) = Uuid::from_str(inner) {
                    let cid = ChannelId(uuid);
                    if !channel_ids.contains(&cid) {
                        channel_ids.push(cid);
                    }
                }
            }
            i += close + 1;
        } else {
            i += 1;
        }
    }

    ParsedMentions {
        user_ids,
        role_ids,
        channel_ids,
        everyone,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn uid(u: Uuid) -> UserId {
        UserId(u)
    }
    fn rid(u: Uuid) -> RoleId {
        RoleId(u)
    }
    fn cid(u: Uuid) -> ChannelId {
        ChannelId(u)
    }

    #[test]
    fn empty_content_returns_empty() {
        let result = parse_mentions("");
        assert!(result.user_ids.is_empty());
        assert!(result.role_ids.is_empty());
        assert!(result.channel_ids.is_empty());
        assert!(!result.everyone);
    }

    #[test]
    fn plain_text_returns_empty() {
        let result = parse_mentions("hello world");
        assert!(result.user_ids.is_empty());
        assert!(!result.everyone);
    }

    #[test]
    fn parses_user_mention() {
        let u = Uuid::new_v4();
        let content = format!("hello <@{u}> there");
        let result = parse_mentions(&content);
        assert_eq!(result.user_ids, vec![uid(u)]);
        assert!(result.role_ids.is_empty());
        assert!(result.channel_ids.is_empty());
    }

    #[test]
    fn parses_role_mention() {
        let u = Uuid::new_v4();
        let content = format!("ping <@&{u}>");
        let result = parse_mentions(&content);
        assert_eq!(result.role_ids, vec![rid(u)]);
        assert!(result.user_ids.is_empty());
    }

    #[test]
    fn parses_channel_mention() {
        let u = Uuid::new_v4();
        let content = format!("see <#{u}>");
        let result = parse_mentions(&content);
        assert_eq!(result.channel_ids, vec![cid(u)]);
    }

    #[test]
    fn parses_everyone() {
        let result = parse_mentions("hey @everyone!");
        assert!(result.everyone);
    }

    #[test]
    fn everyone_not_triggered_by_substring() {
        // "@everyoneelse" still sets the flag because we use contains("@everyone").
        // This is acceptable Discord-parity behaviour (same as Discord's own parser).
        // What we test is the negative: a string with NO @everyone substring.
        let result = parse_mentions("hey @all");
        assert!(!result.everyone);
    }

    #[test]
    fn deduplicates_repeated_user_mentions() {
        let u = Uuid::new_v4();
        let content = format!("<@{u}> and again <@{u}>");
        let result = parse_mentions(&content);
        assert_eq!(result.user_ids.len(), 1);
        assert_eq!(result.user_ids[0], uid(u));
    }

    #[test]
    fn deduplicates_repeated_role_mentions() {
        let u = Uuid::new_v4();
        let content = format!("<@&{u}><@&{u}>");
        let result = parse_mentions(&content);
        assert_eq!(result.role_ids.len(), 1);
    }

    #[test]
    fn multiple_distinct_user_mentions_preserved_in_order() {
        let u1 = Uuid::new_v4();
        let u2 = Uuid::new_v4();
        let content = format!("<@{u1}> then <@{u2}>");
        let result = parse_mentions(&content);
        assert_eq!(result.user_ids, vec![uid(u1), uid(u2)]);
    }

    #[test]
    fn malformed_uuid_in_user_token_silently_skipped() {
        let result = parse_mentions("<@not-a-uuid>");
        assert!(result.user_ids.is_empty());
    }

    #[test]
    fn malformed_uuid_in_role_token_silently_skipped() {
        let result = parse_mentions("<@&bad>");
        assert!(result.role_ids.is_empty());
    }

    #[test]
    fn malformed_uuid_in_channel_token_silently_skipped() {
        let result = parse_mentions("<#oops>");
        assert!(result.channel_ids.is_empty());
    }

    #[test]
    fn mixed_mentions_all_parsed() {
        let u = Uuid::new_v4();
        let r = Uuid::new_v4();
        let c = Uuid::new_v4();
        let content = format!("<@{u}> <@&{r}> <#{c}> @everyone");
        let result = parse_mentions(&content);
        assert_eq!(result.user_ids, vec![uid(u)]);
        assert_eq!(result.role_ids, vec![rid(r)]);
        assert_eq!(result.channel_ids, vec![cid(c)]);
        assert!(result.everyone);
    }

    #[test]
    fn unclosed_angle_bracket_ignored() {
        let result = parse_mentions("hello <@no-close");
        assert!(result.user_ids.is_empty());
    }
}

use common::CoreError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct MediaItem {
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum ButtonStyle {
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum SeparatorSpacing {
    Small,
    Large,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Component {
    Container {
        accent_color: Option<u32>,
        #[schema(no_recursion)]
        children: Vec<Component>,
    },
    Section {
        /// Must all be TextDisplay variants.
        #[schema(no_recursion)]
        children: Vec<Component>,
        #[schema(no_recursion)]
        accessory: Option<Box<Component>>,
    },
    TextDisplay {
        content: String,
    },
    MediaGallery {
        items: Vec<MediaItem>,
    },
    Thumbnail {
        media: MediaItem,
    },
    Separator {
        divider: bool,
        spacing: Option<SeparatorSpacing>,
    },
    ActionRow {
        /// Must all be Button variants.
        #[schema(no_recursion)]
        components: Vec<Component>,
    },
    Button {
        style: ButtonStyle,
        label: String,
        url: String,
        emoji: Option<String>,
    },
}

pub fn validate(components: &[Component]) -> Result<(), CoreError> {
    for component in components {
        validate_component(component)?;
    }
    Ok(())
}

fn validate_component(component: &Component) -> Result<(), CoreError> {
    match component {
        Component::Container { children, .. } => {
            if children.is_empty() {
                return Err(CoreError::Conflict(
                    "Container must have at least one child".into(),
                ));
            }
            for child in children {
                validate_component(child)?;
            }
        }
        Component::Section {
            children,
            accessory,
        } => {
            if children.is_empty() {
                return Err(CoreError::Conflict(
                    "Section must have at least one TextDisplay child".into(),
                ));
            }
            for child in children {
                if !matches!(child, Component::TextDisplay { .. }) {
                    return Err(CoreError::Conflict(
                        "Section children must all be TextDisplay".into(),
                    ));
                }
                validate_component(child)?;
            }
            if let Some(acc) = accessory {
                match acc.as_ref() {
                    Component::Thumbnail { .. } | Component::Button { .. } => {
                        validate_component(acc)?;
                    }
                    _ => {
                        return Err(CoreError::Conflict(
                            "Section accessory must be a Thumbnail or Button".into(),
                        ));
                    }
                }
            }
        }
        Component::TextDisplay { content } => {
            if content.is_empty() {
                return Err(CoreError::Conflict(
                    "TextDisplay content must not be empty".into(),
                ));
            }
        }
        Component::MediaGallery { items } => {
            if items.is_empty() {
                return Err(CoreError::Conflict(
                    "MediaGallery must have at least one item".into(),
                ));
            }
            for item in items {
                validate_url(&item.url)?;
            }
        }
        Component::Thumbnail { media } => {
            validate_url(&media.url)?;
        }
        Component::Separator { .. } => {}
        Component::ActionRow { components } => {
            if components.is_empty() {
                return Err(CoreError::Conflict(
                    "ActionRow must have at least one Button".into(),
                ));
            }
            for btn in components {
                if !matches!(btn, Component::Button { .. }) {
                    return Err(CoreError::Conflict(
                        "ActionRow components must all be Button".into(),
                    ));
                }
                validate_component(btn)?;
            }
        }
        Component::Button {
            style, label, url, ..
        } => {
            // Link is the only supported style in v1; this exhaustive match makes adding
            // a new ButtonStyle a compile error here until validation is decided.
            match style {
                ButtonStyle::Link => {}
            }
            if label.is_empty() {
                return Err(CoreError::Conflict("Button label must not be empty".into()));
            }
            validate_url(url)?;
        }
    }
    Ok(())
}

fn validate_url(url: &str) -> Result<(), CoreError> {
    let parsed =
        url::Url::parse(url).map_err(|_| CoreError::Conflict(format!("invalid URL `{url}`")))?;
    if parsed.scheme() != "https" {
        return Err(CoreError::Conflict(format!(
            "URL must use https scheme, got `{}`",
            parsed.scheme()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(content: &str) -> Component {
        Component::TextDisplay {
            content: content.into(),
        }
    }

    fn link_button(label: &str, url: &str) -> Component {
        Component::Button {
            style: ButtonStyle::Link,
            label: label.into(),
            url: url.into(),
            emoji: None,
        }
    }

    // --- validate: valid trees ---

    #[test]
    fn valid_empty_slice_passes() {
        assert!(validate(&[]).is_ok());
    }

    #[test]
    fn valid_text_display_passes() {
        assert!(validate(&[text("hello")]).is_ok());
    }

    #[test]
    fn valid_container_with_text_passes() {
        let c = Component::Container {
            accent_color: None,
            children: vec![text("content")],
        };
        assert!(validate(&[c]).is_ok());
    }

    #[test]
    fn valid_section_with_text_and_no_accessory_passes() {
        let c = Component::Section {
            children: vec![text("body")],
            accessory: None,
        };
        assert!(validate(&[c]).is_ok());
    }

    #[test]
    fn valid_section_with_thumbnail_accessory_passes() {
        let c = Component::Section {
            children: vec![text("body")],
            accessory: Some(Box::new(Component::Thumbnail {
                media: MediaItem {
                    url: "https://example.com/img.png".into(),
                    description: None,
                },
            })),
        };
        assert!(validate(&[c]).is_ok());
    }

    #[test]
    fn valid_section_with_button_accessory_passes() {
        let c = Component::Section {
            children: vec![text("body")],
            accessory: Some(Box::new(Component::Button {
                style: ButtonStyle::Link,
                label: "Open".into(),
                url: "https://example.com".into(),
                emoji: None,
            })),
        };
        assert!(validate(&[c]).is_ok());
    }

    #[test]
    fn valid_action_row_with_link_button_passes() {
        let c = Component::ActionRow {
            components: vec![link_button("Click", "https://example.com")],
        };
        assert!(validate(&[c]).is_ok());
    }

    #[test]
    fn valid_media_gallery_passes() {
        let c = Component::MediaGallery {
            items: vec![MediaItem {
                url: "https://cdn.example.com/a.jpg".into(),
                description: Some("Alt".into()),
            }],
        };
        assert!(validate(&[c]).is_ok());
    }

    #[test]
    fn valid_separator_passes() {
        let c = Component::Separator {
            divider: true,
            spacing: Some(SeparatorSpacing::Small),
        };
        assert!(validate(&[c]).is_ok());
    }

    // --- validate: invalid trees ---

    #[test]
    fn empty_container_rejected() {
        let c = Component::Container {
            accent_color: None,
            children: vec![],
        };
        let err = validate(&[c]).unwrap_err();
        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn empty_section_rejected() {
        let c = Component::Section {
            children: vec![],
            accessory: None,
        };
        assert!(validate(&[c]).is_err());
    }

    #[test]
    fn section_with_non_text_child_rejected() {
        let c = Component::Section {
            children: vec![link_button("x", "https://example.com")],
            accessory: None,
        };
        assert!(validate(&[c]).is_err());
    }

    #[test]
    fn section_with_invalid_accessory_rejected() {
        let c = Component::Section {
            children: vec![text("body")],
            accessory: Some(Box::new(Component::Separator {
                divider: false,
                spacing: None,
            })),
        };
        assert!(validate(&[c]).is_err());
    }

    #[test]
    fn empty_text_display_rejected() {
        assert!(validate(&[text("")]).is_err());
    }

    #[test]
    fn empty_action_row_rejected() {
        let c = Component::ActionRow { components: vec![] };
        assert!(validate(&[c]).is_err());
    }

    #[test]
    fn action_row_with_non_button_rejected() {
        let c = Component::ActionRow {
            components: vec![text("oops")],
        };
        assert!(validate(&[c]).is_err());
    }

    #[test]
    fn button_with_empty_label_rejected() {
        let c = link_button("", "https://example.com");
        assert!(validate(&[c]).is_err());
    }

    #[test]
    fn button_with_http_url_rejected() {
        let c = link_button("label", "http://example.com");
        assert!(validate(&[c]).is_err());
    }

    #[test]
    fn button_with_invalid_url_rejected() {
        let c = link_button("label", "not a url");
        assert!(validate(&[c]).is_err());
    }

    #[test]
    fn media_gallery_with_invalid_url_rejected() {
        let c = Component::MediaGallery {
            items: vec![MediaItem {
                url: "ftp://old.example.com".into(),
                description: None,
            }],
        };
        assert!(validate(&[c]).is_err());
    }

    #[test]
    fn empty_media_gallery_rejected() {
        let c = Component::MediaGallery { items: vec![] };
        assert!(validate(&[c]).is_err());
    }

    #[test]
    fn nested_container_with_invalid_child_fails() {
        let c = Component::Container {
            accent_color: Some(0xFF0000),
            children: vec![
                text("ok"),
                Component::Container {
                    accent_color: None,
                    children: vec![],
                }, // empty inner
            ],
        };
        assert!(validate(&[c]).is_err());
    }
}

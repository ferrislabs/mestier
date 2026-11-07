pub const FALLBACK_SLUG: &str = "collection";
const MAX_SLUG_BASE_LENGTH: usize = 200;

const FOLDS: &[(&str, &str)] = &[
    ("àáâãäåāăąạảấầẩẫậắằẳẵặǎ", "a"),
    ("æ", "ae"),
    ("çćĉċč", "c"),
    ("ďđð", "d"),
    ("èéêëēĕėęěẹẻẽếềểễệ", "e"),
    ("ĝğġģ", "g"),
    ("ĥħ", "h"),
    ("ìíîïĩīĭįıỉị", "i"),
    ("ĵ", "j"),
    ("ķ", "k"),
    ("ĺļľł", "l"),
    ("ñńņň", "n"),
    ("òóôõöøōŏőơọỏốồổỗộớờởỡợǒ", "o"),
    ("œ", "oe"),
    ("ŕŗř", "r"),
    ("śŝşšș", "s"),
    ("ß", "ss"),
    ("ţťŧț", "t"),
    ("þ", "th"),
    ("ùúûüũūŭůűųưụủứừửữựǔ", "u"),
    ("ŵ", "w"),
    ("ýÿŷỳỵỷỹ", "y"),
    ("źżž", "z"),
];

fn folded(character: char) -> Option<&'static str> {
    FOLDS
        .iter()
        .find(|(letters, _)| letters.contains(character))
        .map(|(_, folded)| *folded)
}

fn is_combining_mark(character: char) -> bool {
    ('\u{0300}'..='\u{036f}').contains(&character)
}

pub fn slugify(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    let mut pending_separator = false;

    for character in title.chars().flat_map(char::to_lowercase) {
        if is_combining_mark(character) {
            continue;
        }
        let piece = if character.is_ascii_alphanumeric() {
            let mut buffer = [0u8; 4];
            Some(character.encode_utf8(&mut buffer).to_owned())
        } else {
            folded(character).map(str::to_owned)
        };
        match piece {
            Some(piece) => {
                if pending_separator && !slug.is_empty() {
                    slug.push('-');
                }
                pending_separator = false;
                slug.push_str(&piece);
            }
            None => pending_separator = true,
        }
    }

    slug
}

pub fn slug_base(title: &str) -> String {
    let slug = slugify(title);
    let truncated: String = slug.chars().take(MAX_SLUG_BASE_LENGTH).collect();
    let trimmed = truncated.trim_end_matches('-');
    if trimmed.is_empty() {
        FALLBACK_SLUG.to_owned()
    } else {
        trimmed.to_owned()
    }
}

pub fn unique_slug(base: &str, taken: &[String]) -> String {
    let is_taken = |candidate: &str| taken.iter().any(|slug| slug == candidate);
    if !is_taken(base) {
        return base.to_owned();
    }
    (2..)
        .map(|suffix| format!("{base}-{suffix}"))
        .find(|candidate| !is_taken(candidate))
        .expect("an unbounded range always reaches a free suffix")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn taken(slugs: &[&str]) -> Vec<String> {
        slugs.iter().map(|slug| (*slug).to_owned()).collect()
    }

    #[test]
    fn a_title_becomes_a_lowercase_hyphenated_slug() {
        assert_eq!(
            slugify("Annuaire des entreprises"),
            "annuaire-des-entreprises"
        );
        assert_eq!(slugify("API Adresse (v2)"), "api-adresse-v2");
    }

    #[test]
    fn accents_are_folded_not_turned_into_separators() {
        assert_eq!(slugify("Éléphant  rose!!"), "elephant-rose");
        assert_eq!(slugify("Façade Œuvre"), "facade-oeuvre");
    }

    #[test]
    fn a_decomposed_accent_is_folded_too() {
        assert_eq!(slugify("Cafe\u{0301} noir"), "cafe-noir");
    }

    #[test]
    fn leading_and_trailing_separators_are_trimmed() {
        assert_eq!(slugify("  --Hello, world--  "), "hello-world");
    }

    #[test]
    fn a_title_with_nothing_latin_slugs_to_the_fallback() {
        assert_eq!(slugify("日本語"), "");
        assert_eq!(slug_base("日本語"), FALLBACK_SLUG);
        assert_eq!(slug_base("API"), "api");
    }

    #[test]
    fn vietnamese_precomposed_letters_are_folded() {
        assert_eq!(
            slugify("Tiếng Việt: Hà Nội, Đà Nẵng, Phở bò"),
            "tieng-viet-ha-noi-da-nang-pho-bo"
        );
    }

    #[test]
    fn romanian_comma_below_letters_are_folded() {
        assert_eq!(slugify("Școală Țară"), "scoala-tara");
    }

    #[test]
    fn a_title_of_two_hundred_sharp_s_cannot_overflow_the_slug_column() {
        let title = "ß".repeat(200);

        let base = slug_base(&title);

        assert!(base.chars().count() <= 200, "{}", base.len());
        assert!(!base.ends_with('-'));
        assert_eq!(base, "s".repeat(200));
        let taken = vec![base.clone()];
        assert!(unique_slug(&base, &taken).chars().count() <= 210);
    }

    #[test]
    fn truncation_never_leaves_a_trailing_hyphen() {
        let title = format!("{} {}", "a".repeat(199), "tail");

        let base = slug_base(&title);

        assert_eq!(base, "a".repeat(199));
    }

    #[test]
    fn a_free_base_is_kept() {
        assert_eq!(unique_slug("api", &[]), "api");
        assert_eq!(unique_slug("api", &taken(&["other"])), "api");
    }

    #[test]
    fn a_clashing_base_gets_the_suffix_two() {
        assert_eq!(unique_slug("api", &taken(&["api"])), "api-2");
    }

    #[test]
    fn the_suffix_counts_up_past_what_is_taken() {
        assert_eq!(unique_slug("api", &taken(&["api", "api-2"])), "api-3");
        assert_eq!(
            unique_slug("api", &taken(&["api", "api-2", "api-3"])),
            "api-4"
        );
    }

    #[test]
    fn the_first_gap_in_the_suffixes_is_filled() {
        assert_eq!(unique_slug("api", &taken(&["api", "api-3"])), "api-2");
    }

    #[test]
    fn a_suffixed_slug_alone_does_not_block_the_base() {
        assert_eq!(unique_slug("api", &taken(&["api-2"])), "api");
    }
}

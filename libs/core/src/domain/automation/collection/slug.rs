pub const FALLBACK_SLUG: &str = "collection";

fn folded(character: char) -> Option<&'static str> {
    match character {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => Some("a"),
        'æ' => Some("ae"),
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => Some("c"),
        'ď' | 'đ' | 'ð' => Some("d"),
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => Some("e"),
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => Some("g"),
        'ĥ' | 'ħ' => Some("h"),
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => Some("i"),
        'ĵ' => Some("j"),
        'ķ' => Some("k"),
        'ĺ' | 'ļ' | 'ľ' | 'ł' => Some("l"),
        'ñ' | 'ń' | 'ņ' | 'ň' => Some("n"),
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => Some("o"),
        'œ' => Some("oe"),
        'ŕ' | 'ŗ' | 'ř' => Some("r"),
        'ś' | 'ŝ' | 'ş' | 'š' => Some("s"),
        'ß' => Some("ss"),
        'ţ' | 'ť' | 'ŧ' => Some("t"),
        'þ' => Some("th"),
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => Some("u"),
        'ŵ' => Some("w"),
        'ý' | 'ÿ' | 'ŷ' => Some("y"),
        'ź' | 'ż' | 'ž' => Some("z"),
        _ => None,
    }
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
    if slug.is_empty() {
        FALLBACK_SLUG.to_owned()
    } else {
        slug
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

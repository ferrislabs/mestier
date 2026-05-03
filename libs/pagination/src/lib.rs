use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

pub const DEFAULT_PER_PAGE: u64 = 20;
pub const MAX_PER_PAGE: u64 = 100;

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaginationParams {
    #[serde(default)]
    pub page: Option<u64>,
    #[serde(default)]
    pub per_page: Option<u64>,
}

impl PaginationParams {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn per_page(&self) -> u64 {
        self.per_page
            .unwrap_or(DEFAULT_PER_PAGE)
            .clamp(1, MAX_PER_PAGE)
    }

    pub fn offset(&self) -> u64 {
        (self.page() - 1) * self.per_page()
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct PaginationMetadata {
    pub per_page: u64,
    pub current_page: u64,
    pub first_page: u64,
    pub is_empty: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_page: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_page: Option<u64>,
}

impl PaginationMetadata {
    pub fn new(per_page: u64, current_page: u64, total: Option<u64>, is_empty: bool) -> Self {
        let last_page = total.map(|t| t.div_ceil(per_page).max(1));
        let next_page = last_page.and_then(|last| (current_page < last).then(|| current_page + 1));
        let prev_page = (current_page > 1).then(|| current_page - 1);

        Self {
            per_page,
            current_page,
            first_page: 1,
            is_empty,
            total,
            last_page,
            next_page,
            prev_page,
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Page<T: Serialize + PartialEq> {
    pub items: Vec<T>,
    pub meta: PaginationMetadata,
}

impl<T: Serialize + PartialEq> Page<T> {
    pub fn new(items: Vec<T>, meta: PaginationMetadata) -> Self {
        Self { items, meta }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(per_page: u64, current_page: u64, total: Option<u64>) -> PaginationMetadata {
        PaginationMetadata::new(per_page, current_page, total, false)
    }

    #[test]
    fn derives_last_page_from_total() {
        let m = meta(10, 1, Some(50));
        assert_eq!(m.last_page, Some(5));
    }

    #[test]
    fn last_page_at_least_one() {
        let m = meta(10, 1, Some(0));
        assert_eq!(m.last_page, Some(1));
    }

    #[test]
    fn last_page_absent_when_total_unknown() {
        let m = meta(10, 1, None);
        assert_eq!(m.last_page, None);
    }

    #[test]
    fn next_page_set_when_more_pages_remain() {
        let m = meta(10, 2, Some(50));
        assert_eq!(m.next_page, Some(3));
    }

    #[test]
    fn next_page_absent_on_last_page() {
        let m = meta(10, 5, Some(50));
        assert_eq!(m.next_page, None);
    }

    #[test]
    fn prev_page_set_when_not_first() {
        let m = meta(10, 3, Some(50));
        assert_eq!(m.prev_page, Some(2));
    }

    #[test]
    fn prev_page_absent_on_first_page() {
        let m = meta(10, 1, Some(50));
        assert_eq!(m.prev_page, None);
    }

    #[test]
    fn optional_fields_omitted_when_none() {
        let m = meta(10, 1, None);
        let json = serde_json::to_value(&m).unwrap();
        assert!(!json.as_object().unwrap().contains_key("total"));
        assert!(!json.as_object().unwrap().contains_key("last_page"));
        assert!(!json.as_object().unwrap().contains_key("next_page"));
        assert!(!json.as_object().unwrap().contains_key("prev_page"));
    }

    #[test]
    fn first_page_always_one() {
        let m = meta(20, 3, Some(200));
        assert_eq!(m.first_page, 1);
    }
}

use mestier_core::{BillingFrequency, CatalogProfileInput};

pub fn catalog_input(
    category: Option<String>,
    billing_frequency: Option<BillingFrequency>,
    cost_cents: Option<i32>,
    archived: Option<bool>,
) -> Option<CatalogProfileInput> {
    if category.is_none()
        && billing_frequency.is_none()
        && cost_cents.is_none()
        && archived.is_none()
    {
        return None;
    }
    Some(CatalogProfileInput {
        category: category.unwrap_or_default(),
        billing_frequency: billing_frequency.unwrap_or_default(),
        cost_cents,
        archived: archived.unwrap_or(false),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_catalogue_field_keeps_the_stored_profile() {
        assert_eq!(catalog_input(None, None, None, None), None);
    }

    #[test]
    fn any_catalogue_field_replaces_the_four_with_defaults_for_the_missing_ones() {
        let input = catalog_input(None, None, None, Some(true)).unwrap();

        assert!(input.archived);
        assert_eq!(input.category, "");
        assert_eq!(input.billing_frequency, BillingFrequency::OneShot);
        assert_eq!(input.cost_cents, None);
    }

    #[test]
    fn a_cost_without_the_other_fields_is_stored() {
        let input = catalog_input(None, None, Some(250), None).unwrap();

        assert_eq!(input.cost_cents, Some(250));
        assert!(!input.archived);
    }
}

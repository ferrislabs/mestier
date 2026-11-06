use std::str::FromStr;

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use common::OrganizationId;
use serde_json::{Map, Value, json};

use super::fields;
use crate::QuoteStatus;
use crate::application::MestierUseCase;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};
use crate::domain::quote::search::{
    DEFAULT_QUOTE_SEARCH_RESULTS, MAX_QUOTE_SEARCH_RESULTS, QuoteSearch, QuoteSearchLimit,
    QuoteSummary,
};

pub struct QuoteSearchConnector {
    usecase: MestierUseCase,
}

impl QuoteSearchConnector {
    pub fn new(usecase: MestierUseCase) -> Self {
        Self { usecase }
    }
}

fn status_from(config: &Map<String, Value>) -> Result<Option<QuoteStatus>, String> {
    fields::optional_text(config, "status")?
        .map(|status| {
            QuoteStatus::from_str(status).map_err(|_| {
                "`status` must be DRAFT, SENT, ACCEPTED, DECLINED or CANCELLED".to_owned()
            })
        })
        .transpose()
}

fn updated_before_from(
    config: &Map<String, Value>,
    now: DateTime<Utc>,
) -> Result<Option<DateTime<Utc>>, String> {
    let too_far = || "`since_days` is too large".to_owned();
    fields::optional_whole_number(config, "since_days")?
        .map(|days| {
            let days = i64::try_from(days).map_err(|_| too_far())?;
            let age = Duration::try_days(days).ok_or_else(too_far)?;
            now.checked_sub_signed(age).ok_or_else(too_far)
        })
        .transpose()
}

fn limit_from(config: &Map<String, Value>) -> Result<QuoteSearchLimit, String> {
    let requested =
        fields::optional_whole_number(config, "limit")?.unwrap_or(DEFAULT_QUOTE_SEARCH_RESULTS);
    QuoteSearchLimit::new(requested)
        .map_err(|_| format!("`limit` must be between 1 and {MAX_QUOTE_SEARCH_RESULTS}"))
}

fn search_from(
    organization_id: OrganizationId,
    config: &Map<String, Value>,
    now: DateTime<Utc>,
) -> Result<QuoteSearch, String> {
    Ok(QuoteSearch {
        organization_id,
        status: status_from(config)?,
        updated_before: updated_before_from(config, now)?,
        limit: limit_from(config)?,
    })
}

fn instant(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn summary_json(summary: &QuoteSummary) -> Value {
    json!({
        "id": summary.id.0,
        "reference": summary.reference,
        "title": summary.title,
        "status": summary.status.as_str(),
        "customer_id": summary.customer_id.0,
        "net_cents": summary.net_cents,
        "gross_cents": summary.gross_cents,
        "created_at": instant(summary.created_at),
        "updated_at": instant(summary.updated_at),
    })
}

impl Connector for QuoteSearchConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        let search = match search_from(input.org_id, input.config, Utc::now()) {
            Ok(search) => search,
            Err(error) => return ConnectorOutcome::Failed { error },
        };

        match self.usecase.search_quotes(search).await {
            Ok(results) => {
                let items: Vec<Value> = results.items().iter().map(summary_json).collect();
                ConnectorOutcome::Produced(Value::Array(items))
            }
            Err(error) => ConnectorOutcome::Failed {
                error: error.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::application::default_authorizer;
    use crate::application::test_support::automation_pool;
    use crate::domain::automation::connector::connector_catalogue;
    use crate::domain::automation::run::DryRunInput;
    use crate::infrastructure::automation::connectors::test_support::assert_same_shape;
    use crate::infrastructure::automation::connectors::world_support;
    use crate::infrastructure::automation::postgres::run::RUN_CLAIM_LOCK;
    use crate::infrastructure::realtime::EventHub;

    fn noon() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 6, 12, 0, 0).unwrap()
    }

    fn org() -> OrganizationId {
        OrganizationId(Uuid::from_u128(1))
    }

    fn config(entries: &[(&str, Value)]) -> Map<String, Value> {
        entries
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect()
    }

    #[test]
    fn an_empty_config_searches_every_quote_up_to_the_default_limit() {
        let search = search_from(org(), &Map::new(), noon()).unwrap();

        assert_eq!(search.status, None);
        assert_eq!(search.updated_before, None);
        assert_eq!(search.limit.get(), 50);
        assert_eq!(search.organization_id, org());
    }

    #[test]
    fn the_filters_are_read_from_the_config() {
        let search = search_from(
            org(),
            &config(&[
                ("status", json!("SENT")),
                ("since_days", json!("7")),
                ("limit", json!(200)),
            ]),
            noon(),
        )
        .unwrap();

        assert_eq!(search.status, Some(QuoteStatus::Sent));
        assert_eq!(search.updated_before, Some(noon() - Duration::days(7)));
        assert_eq!(search.limit.get(), 200);
    }

    #[test]
    fn a_blank_filter_is_no_filter() {
        let search = search_from(
            org(),
            &config(&[
                ("status", json!("")),
                ("since_days", json!(null)),
                ("limit", json!("")),
            ]),
            noon(),
        )
        .unwrap();

        assert_eq!(search.status, None);
        assert_eq!(search.updated_before, None);
        assert_eq!(search.limit.get(), 50);
    }

    #[test]
    fn an_invalid_filter_fails_naming_the_field_without_echoing_it() {
        let secret = "do-not-leak-this-token";
        let cases = [
            (
                config(&[("status", json!(secret))]),
                "`status` must be DRAFT, SENT, ACCEPTED, DECLINED or CANCELLED",
            ),
            (
                config(&[("status", json!(4))]),
                "`status` must resolve to a string",
            ),
            (
                config(&[("since_days", json!(secret))]),
                "`since_days` must be a whole number",
            ),
            (
                config(&[("since_days", json!(u64::MAX))]),
                "`since_days` is too large",
            ),
            (
                config(&[("limit", json!(0))]),
                "`limit` must be between 1 and 200",
            ),
            (
                config(&[("limit", json!(201))]),
                "`limit` must be between 1 and 200",
            ),
            (
                config(&[("limit", json!(secret))]),
                "`limit` must be a whole number",
            ),
        ];

        for (config, expected) in cases {
            let error = search_from(org(), &config, noon()).unwrap_err();

            assert_eq!(error, expected);
            assert!(!error.contains(secret));
        }
    }

    #[test]
    fn the_dry_run_answers_with_the_sample_and_its_items_name_the_fields_of_a_quote() {
        let catalogue = connector_catalogue();
        let descriptor = catalogue.get("mestier.quote.search", 1).unwrap();
        let config = Map::new();

        let outcome = QuoteSearchConnector::dry_run(
            descriptor,
            &DryRunInput {
                config: &config,
                stored: None,
            },
        );

        assert_eq!(
            outcome,
            ConnectorOutcome::Produced(descriptor.sample_output.clone())
        );
        let summary = QuoteSummary {
            id: crate::QuoteId(Uuid::from_u128(1)),
            reference: Some("DV-1".to_owned()),
            title: "T".to_owned(),
            customer_id: crate::CustomerId(Uuid::from_u128(2)),
            status: QuoteStatus::Sent,
            net_cents: 1,
            gross_cents: 2,
            created_at: noon(),
            updated_at: noon(),
        };
        let item = summary_json(&summary);
        let sample_item = &descriptor.sample_output[0];
        let names = |value: &Value| {
            let mut names: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
            names.sort();
            names
        };
        assert_eq!(names(&item), names(sample_item));
        assert_same_shape(&json!([item]), "mestier.quote.search");
    }

    fn connector(pool: PgPool) -> QuoteSearchConnector {
        QuoteSearchConnector::new(MestierUseCase::new(
            pool,
            default_authorizer(),
            EventHub::new(),
        ))
    }

    fn input<'a>(org: OrganizationId, config: &'a Map<String, Value>) -> ConnectorInput<'a> {
        ConnectorInput {
            org_id: org,
            run_id: Uuid::from_u128(2),
            config,
            credential_id: None,
            operation: None,
        }
    }

    fn titles(outcome: ConnectorOutcome) -> Vec<String> {
        match outcome {
            ConnectorOutcome::Produced(output) => output
                .as_array()
                .unwrap()
                .iter()
                .map(|item| item["title"].as_str().unwrap().to_owned())
                .collect(),
            other => panic!("expected an output, got {other:?}"),
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_search_returns_the_matching_quotes_of_the_run_organization_only() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = automation_pool().await;
        let acme = world_support::organization(&pool, "quote-search-acme").await;
        let globex = world_support::organization(&pool, "quote-search-globex").await;
        let customer = world_support::customer(&pool, acme).await;
        let foreign = world_support::customer(&pool, globex).await;
        world_support::quote(&pool, acme, &customer, "SENT", "Sent long ago", 30).await;
        world_support::quote(&pool, acme, &customer, "SENT", "Sent yesterday", 1).await;
        world_support::quote(&pool, acme, &customer, "DRAFT", "Draft long ago", 30).await;
        world_support::quote(
            &pool,
            globex,
            &foreign,
            "SENT",
            "Of another organization",
            30,
        )
        .await;
        let connector = connector(pool.clone());

        let sent = connector
            .execute(input(acme, &config(&[("status", json!("SENT"))])))
            .await;
        let stale = connector
            .execute(input(
                acme,
                &config(&[("status", json!("SENT")), ("since_days", json!(7))]),
            ))
            .await;
        let everything = connector.execute(input(acme, &Map::new())).await;
        let nothing = connector
            .execute(input(acme, &config(&[("status", json!("ACCEPTED"))])))
            .await;

        let mut sent = titles(sent);
        sent.sort();
        assert_eq!(sent, ["Sent long ago", "Sent yesterday"]);
        assert_eq!(titles(stale), ["Sent long ago"]);
        assert_eq!(titles(everything).len(), 3);
        assert!(titles(nothing).is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_search_returns_at_most_the_limit_and_never_more_than_two_hundred() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = automation_pool().await;
        let org = world_support::organization(&pool, "quote-search-limit").await;
        let customer = world_support::customer(&pool, org).await;
        for index in 0..205 {
            world_support::quote(&pool, org, &customer, "SENT", &format!("Quote {index}"), 3).await;
        }
        let connector = connector(pool.clone());

        let three = connector
            .execute(input(org, &config(&[("limit", json!(3))])))
            .await;
        let most = connector
            .execute(input(org, &config(&[("limit", json!(200))])))
            .await;
        let too_many = connector
            .execute(input(org, &config(&[("limit", json!(201))])))
            .await;

        assert_eq!(titles(three).len(), 3);
        assert_eq!(titles(most).len(), 200);
        assert!(matches!(too_many, ConnectorOutcome::Failed { .. }));
    }
}

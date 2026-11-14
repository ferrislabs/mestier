use serde_json::json;
use uuid::Uuid;

use super::super::clients::tests_clients::{config, failure, produced, registry, run};
use super::{answers_payload, public_path};
use crate::application::test_support::automation_pool;
use crate::domain::survey::{
    Answer, Answers, Branch, Edge, EndNode, Graph, Metadata, Node, PublicSlug, QuestionId,
    QuestionKind, QuestionNode, Visibility, Window,
};
use crate::infrastructure::automation::connectors::test_support::assert_same_shape;
use crate::infrastructure::automation::connectors::world_support;
use crate::infrastructure::automation::postgres::run::RUN_CLAIM_LOCK;
use crate::{CreateCampaignCommand, CreateSurveyCommand, SurveyPatch, UpdateSurveyCommand};

fn qid(n: u128) -> QuestionId {
    QuestionId(Uuid::from_u128(n))
}

fn question(id: u128, kind: QuestionKind, label: &str, options: &[&str]) -> Node {
    Node::Question(QuestionNode {
        id: qid(id),
        kind,
        label: label.to_owned(),
        required: false,
        options: options.iter().map(|option| (*option).to_owned()).collect(),
        max_photos: None,
        identifier: false,
    })
}

fn two_questions() -> Graph {
    Graph::new(
        vec![
            question(
                10,
                QuestionKind::SingleChoice,
                "Satisfait ?",
                &["Oui", "Non"],
            ),
            question(11, QuestionKind::Text, "Un commentaire ?", &[]),
            Node::End(EndNode {
                id: qid(12),
                message: None,
            }),
        ],
        vec![
            Edge {
                from: qid(10),
                to: qid(11),
                branch: Branch::Next,
            },
            Edge {
                from: qid(11),
                to: qid(12),
                branch: Branch::Next,
            },
        ],
        qid(10),
    )
    .unwrap()
}

fn lazy_registry() -> crate::infrastructure::automation::connectors::ConnectorRegistry {
    registry(&sqlx::PgPool::connect_lazy("postgres://unused:unused@localhost/unused").unwrap())
}

fn organization() -> common::OrganizationId {
    common::OrganizationId(Uuid::from_u128(1))
}

#[test]
fn the_answers_follow_the_path_with_the_label_and_kind_of_each_question() {
    let answers = Answers::from([
        (qid(10), Answer::Choice("Oui".to_owned())),
        (qid(99), Answer::Text("hors chemin".to_owned())),
    ]);

    let payload = answers_payload(&two_questions(), &answers);

    assert_eq!(
        payload,
        vec![
            json!({
                "question_id": qid(10).0,
                "label": "Satisfait ?",
                "kind": "SINGLE_CHOICE",
                "value": "Oui",
            }),
            json!({
                "question_id": qid(11).0,
                "label": "Un commentaire ?",
                "kind": "TEXT",
                "value": null,
            }),
        ]
    );
}

#[test]
fn the_public_path_is_the_slug_under_s() {
    let slug = PublicSlug::new("q7Xk2mP9vR4tL8wZ3nB6cA".to_owned()).unwrap();

    assert_eq!(public_path(&slug), "/s/q7Xk2mP9vR4tL8wZ3nB6cA");
}

#[tokio::test]
async fn an_invalid_config_is_refused_before_the_database() {
    let registry = lazy_registry();
    let survey = json!(Uuid::from_u128(5).to_string());
    let cases = [
        (
            "mestier.survey.open",
            config(&[]),
            "`survey_id` must resolve to a string",
        ),
        (
            "mestier.survey.open",
            config(&[("survey_id", survey.clone()), ("campaign_id", json!("x"))]),
            "`campaign_id` must be a UUID",
        ),
        (
            "mestier.survey.close",
            config(&[("survey_id", json!("x"))]),
            "`survey_id` must be a UUID",
        ),
        (
            "mestier.survey.get_link",
            config(&[("survey_id", json!(7))]),
            "`survey_id` must resolve to a string",
        ),
        (
            "mestier.survey.list_responses",
            config(&[("survey_id", survey), ("limit", json!("many"))]),
            "`limit` must be a whole number",
        ),
    ];

    for (kind, config, expected) in cases {
        assert_eq!(
            failure(run(&registry, kind, organization(), &config).await),
            expected,
            "{kind}"
        );
    }
}

async fn enable_surveys(pool: &sqlx::PgPool, organization: common::OrganizationId) {
    sqlx::query("INSERT INTO organization_feature_flags (org_id, flag) VALUES ($1, 'surveys')")
        .bind(organization.0)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_public_survey_is_opened_linked_answered_listed_and_closed_by_actions() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "survey-actions").await;
    enable_surveys(&pool, acme).await;
    let registry = registry(&pool);
    let usecase = crate::application::MestierUseCase::new(
        pool.clone(),
        crate::application::default_authorizer(),
        crate::infrastructure::realtime::EventHub::new(),
    );
    let draft = usecase
        .create_survey(CreateSurveyCommand {
            actor: authz::Subject::system(),
            organization_id: acme,
            title: "Bilan".to_owned(),
            visibility: Visibility::Public,
        })
        .await
        .unwrap();
    usecase
        .update_survey(UpdateSurveyCommand {
            actor: authz::Subject::system(),
            organization_id: acme,
            id: draft.id(),
            patch: SurveyPatch {
                graph: Some(Some(two_questions())),
                ..SurveyPatch::default()
            },
            campaign_id: None,
        })
        .await
        .unwrap();
    let target = config(&[("survey_id", json!(draft.id().0.to_string()))]);

    let opened = produced(run(&registry, "mestier.survey.open", acme, &target).await);
    let link = produced(run(&registry, "mestier.survey.get_link", acme, &target).await);
    let slug = PublicSlug::new(link["public_slug"].as_str().unwrap().to_owned()).unwrap();
    let started = usecase.start_public_response(&slug).await.unwrap();
    let token = started.resume_token().unwrap().clone();
    usecase
        .save_public_response(
            &slug,
            token.clone(),
            Answers::from([(qid(10), Answer::Choice("Oui".to_owned()))]),
        )
        .await
        .unwrap();
    usecase
        .complete_public_response(&slug, token)
        .await
        .unwrap();
    usecase.start_public_response(&slug).await.unwrap();
    let listed = produced(run(&registry, "mestier.survey.list_responses", acme, &target).await);
    let skipped = produced(
        run(
            &registry,
            "mestier.survey.list_responses",
            acme,
            &config(&[
                ("survey_id", json!(draft.id().0.to_string())),
                ("offset", json!(1)),
            ]),
        )
        .await,
    );
    let closed = produced(run(&registry, "mestier.survey.close", acme, &target).await);
    let reopened = failure(run(&registry, "mestier.survey.open", acme, &target).await);

    assert_same_shape(&opened, "mestier.survey.open");
    assert_eq!(opened["status"], "OPENED");
    assert_same_shape(&link, "mestier.survey.get_link");
    assert_eq!(link["path"], format!("/s/{}", slug.as_str()));
    assert_same_shape(&listed, "mestier.survey.list_responses");
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(listed[0]["answers"][0]["value"], "Oui");
    assert_eq!(listed[0]["answers"][1]["value"], json!(null));
    assert_eq!(skipped, json!([]));
    assert_same_shape(&closed, "mestier.survey.close");
    assert_eq!(closed["status"], "CLOSED");
    assert!(!reopened.is_empty());
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_survey_of_an_organization_without_the_flag_is_not_found() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "survey-noflag").await;
    let registry = registry(&pool);

    let error = failure(
        run(
            &registry,
            "mestier.survey.get_link",
            acme,
            &config(&[("survey_id", json!(Uuid::from_u128(9).to_string()))]),
        )
        .await,
    );

    assert_eq!(error, super::super::clients::tests_clients::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_members_survey_is_opened_in_its_campaign_and_has_no_public_link() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "survey-campaign").await;
    enable_surveys(&pool, acme).await;
    let registry = registry(&pool);
    let usecase = crate::application::MestierUseCase::new(
        pool.clone(),
        crate::application::default_authorizer(),
        crate::infrastructure::realtime::EventHub::new(),
    );
    let campaign = usecase
        .create_campaign(CreateCampaignCommand {
            actor: authz::Subject::system(),
            organization_id: acme,
            name: "Printemps".to_owned(),
            window: Window::new(None, None).unwrap(),
            metadata: Metadata::new(Default::default()).unwrap(),
        })
        .await
        .unwrap();
    let draft = usecase
        .create_survey(CreateSurveyCommand {
            actor: authz::Subject::system(),
            organization_id: acme,
            title: "Équipe".to_owned(),
            visibility: Visibility::Members,
        })
        .await
        .unwrap();
    usecase
        .update_survey(UpdateSurveyCommand {
            actor: authz::Subject::system(),
            organization_id: acme,
            id: draft.id(),
            patch: SurveyPatch {
                graph: Some(Some(two_questions())),
                ..SurveyPatch::default()
            },
            campaign_id: None,
        })
        .await
        .unwrap();
    let target = config(&[("survey_id", json!(draft.id().0.to_string()))]);

    let opened = produced(
        run(
            &registry,
            "mestier.survey.open",
            acme,
            &config(&[
                ("survey_id", json!(draft.id().0.to_string())),
                ("campaign_id", json!(campaign.id().0.to_string())),
            ]),
        )
        .await,
    );
    let link = failure(run(&registry, "mestier.survey.get_link", acme, &target).await);

    assert_eq!(opened["campaign_id"], json!(campaign.id().0));
    assert_eq!(opened["public_slug"], json!(null));
    assert!(link.contains("MEMBERS"), "{link}");
}

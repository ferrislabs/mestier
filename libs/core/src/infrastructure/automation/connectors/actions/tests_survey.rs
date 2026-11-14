use events::DomainEvent;
use serde_json::json;
use uuid::Uuid;

use super::super::clients::tests_clients::{NOT_FOUND, config, failure, produced, registry, run};
use crate::application::MestierUseCase;
use crate::application::test_support::automation_pool;
use crate::domain::survey::events::answers_payload;
use crate::domain::survey::{
    Answer, Answers, Branch, CampaignId, Edge, EndNode, Graph, Metadata, Node, PublicSlug,
    QuestionId, QuestionKind, QuestionNode, SurveyId, SurveyRecord, Visibility, Window,
};
use crate::infrastructure::automation::connectors::test_support::assert_same_shape;
use crate::infrastructure::automation::connectors::world_support;
use crate::infrastructure::automation::postgres::run::RUN_CLAIM_LOCK;
use crate::{
    CreateCampaignCommand, CreateSurveyCommand, SurveyCommand, SurveyPatch,
    SurveyResponseCompleted, UpdateSurveyCommand,
};

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

fn usecase(pool: &sqlx::PgPool) -> MestierUseCase {
    MestierUseCase::new(
        pool.clone(),
        crate::application::default_authorizer(),
        crate::infrastructure::realtime::EventHub::new(),
    )
}

fn target(survey: SurveyId) -> serde_json::Map<String, serde_json::Value> {
    config(&[("survey_id", json!(survey.0.to_string()))])
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
            config(&[("survey_id", survey.clone()), ("limit", json!("many"))]),
            "`limit` must be a whole number",
        ),
        (
            "mestier.survey.list_responses",
            config(&[("survey_id", survey), ("limit", json!(0))]),
            "`limit` must be between 1 and 100: leave it empty to list 50 responses",
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

async fn disable_surveys(pool: &sqlx::PgPool, organization: common::OrganizationId) {
    sqlx::query("DELETE FROM organization_feature_flags WHERE org_id = $1 AND flag = 'surveys'")
        .bind(organization.0)
        .execute(pool)
        .await
        .unwrap();
}

async fn draft(
    usecase: &MestierUseCase,
    organization: common::OrganizationId,
    visibility: Visibility,
    graph: Option<Graph>,
) -> SurveyId {
    let draft = usecase
        .create_survey(CreateSurveyCommand {
            actor: authz::Subject::system(),
            organization_id: organization,
            title: "Bilan".to_owned(),
            visibility,
        })
        .await
        .unwrap();
    if graph.is_some() {
        usecase
            .update_survey(UpdateSurveyCommand {
                actor: authz::Subject::system(),
                organization_id: organization,
                id: draft.id(),
                patch: SurveyPatch {
                    graph: Some(graph),
                    ..SurveyPatch::default()
                },
                campaign_id: None,
            })
            .await
            .unwrap();
    }
    draft.id()
}

async fn campaign(usecase: &MestierUseCase, organization: common::OrganizationId) -> CampaignId {
    usecase
        .create_campaign(CreateCampaignCommand {
            actor: authz::Subject::system(),
            organization_id: organization,
            name: "Printemps".to_owned(),
            window: Window::new(None, None).unwrap(),
            metadata: Metadata::new(Default::default()).unwrap(),
        })
        .await
        .unwrap()
        .id()
}

async fn stored(
    usecase: &MestierUseCase,
    organization: common::OrganizationId,
    survey: SurveyId,
) -> SurveyRecord {
    usecase
        .get_survey(SurveyCommand {
            actor: authz::Subject::system(),
            organization_id: organization,
            id: survey,
        })
        .await
        .unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_public_survey_is_opened_linked_answered_listed_and_closed_by_actions() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "survey-actions").await;
    enable_surveys(&pool, acme).await;
    let registry = registry(&pool);
    let usecase = usecase(&pool);
    let survey = draft(&usecase, acme, Visibility::Public, Some(two_questions())).await;
    let target = target(survey);

    let opened = produced(run(&registry, "mestier.survey.open", acme, &target).await);
    let link = produced(run(&registry, "mestier.survey.get_link", acme, &target).await);
    let slug = PublicSlug::new(link["public_slug"].as_str().unwrap().to_owned()).unwrap();
    let mut completed = Vec::new();
    for choice in ["Oui", "Non", "Oui"] {
        let started = usecase.start_public_response(&slug).await.unwrap();
        let token = started.resume_token().unwrap().clone();
        usecase
            .save_public_response(
                &slug,
                token.clone(),
                Answers::from([(qid(10), Answer::Choice(choice.to_owned()))]),
            )
            .await
            .unwrap();
        completed.push(
            usecase
                .complete_public_response(&slug, token)
                .await
                .unwrap(),
        );
    }
    usecase.start_public_response(&slug).await.unwrap();
    usecase.start_public_response(&slug).await.unwrap();
    let listed = produced(run(&registry, "mestier.survey.list_responses", acme, &target).await);
    let second = produced(
        run(
            &registry,
            "mestier.survey.list_responses",
            acme,
            &config(&[
                ("survey_id", json!(survey.0.to_string())),
                ("limit", json!(1)),
                ("offset", json!(1)),
            ]),
        )
        .await,
    );
    let SurveyRecord::Opened(frozen) = stored(&usecase, acme, survey).await else {
        panic!("the survey is opened");
    };
    let event = SurveyResponseCompleted::new(&frozen, None, &completed[2])
        .unwrap()
        .payload();
    let closed = produced(run(&registry, "mestier.survey.close", acme, &target).await);
    let reopened = failure(run(&registry, "mestier.survey.open", acme, &target).await);

    assert_same_shape(&opened, "mestier.survey.open");
    assert_eq!(opened["status"], "OPENED");
    assert_same_shape(&link, "mestier.survey.get_link");
    assert_eq!(link["path"], slug.path());
    assert_same_shape(&listed, "mestier.survey.list_responses");
    assert_eq!(listed["total"], 3);
    let ids: Vec<_> = listed["responses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|response| response["id"].clone())
        .collect();
    assert_eq!(
        ids,
        [
            json!(completed[2].id().0),
            json!(completed[1].id().0),
            json!(completed[0].id().0),
        ]
    );
    assert_eq!(listed["responses"][1]["answers"][0]["value"], "Non");
    assert_eq!(listed["responses"][0]["answers"][1]["value"], json!(null));
    assert_eq!(listed["responses"][0]["answers"], event["answers"]);
    assert_eq!(second["total"], 3);
    assert_eq!(second["responses"].as_array().unwrap().len(), 1);
    assert_eq!(second["responses"][0]["id"], json!(completed[1].id().0));
    assert_same_shape(&closed, "mestier.survey.close");
    assert_eq!(closed["status"], "CLOSED");
    assert!(!reopened.is_empty());
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_survey_of_an_organization_whose_flag_was_turned_off_is_not_found() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "survey-flag-off").await;
    enable_surveys(&pool, acme).await;
    let registry = registry(&pool);
    let usecase = usecase(&pool);
    let survey = draft(&usecase, acme, Visibility::Public, Some(two_questions())).await;
    produced(run(&registry, "mestier.survey.open", acme, &target(survey)).await);
    disable_surveys(&pool, acme).await;

    for kind in [
        "mestier.survey.get_link",
        "mestier.survey.list_responses",
        "mestier.survey.close",
    ] {
        assert_eq!(
            failure(run(&registry, kind, acme, &target(survey)).await),
            NOT_FOUND,
            "{kind}"
        );
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_unknown_survey_is_not_found() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "survey-unknown").await;
    enable_surveys(&pool, acme).await;
    let registry = registry(&pool);

    let error = failure(
        run(
            &registry,
            "mestier.survey.get_link",
            acme,
            &target(SurveyId(Uuid::from_u128(9))),
        )
        .await,
    );

    assert_eq!(error, NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn another_organization_reaches_neither_the_survey_nor_lends_its_campaign() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "survey-owner").await;
    let rival = world_support::organization(&pool, "survey-rival").await;
    enable_surveys(&pool, acme).await;
    enable_surveys(&pool, rival).await;
    let registry = registry(&pool);
    let usecase = usecase(&pool);
    let survey = draft(&usecase, acme, Visibility::Public, Some(two_questions())).await;
    let foreign = campaign(&usecase, rival).await;

    for kind in [
        "mestier.survey.open",
        "mestier.survey.close",
        "mestier.survey.get_link",
        "mestier.survey.list_responses",
    ] {
        assert_eq!(
            failure(run(&registry, kind, rival, &target(survey)).await),
            NOT_FOUND,
            "{kind}"
        );
    }
    let borrowed = failure(
        run(
            &registry,
            "mestier.survey.open",
            acme,
            &config(&[
                ("survey_id", json!(survey.0.to_string())),
                ("campaign_id", json!(foreign.0.to_string())),
            ]),
        )
        .await,
    );

    assert_eq!(borrowed, NOT_FOUND);
    let SurveyRecord::Draft(untouched) = stored(&usecase, acme, survey).await else {
        panic!("the survey is still a draft");
    };
    assert_eq!(untouched.campaign_id(), None);
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_failed_opening_leaves_the_draft_out_of_the_campaign() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "survey-open-fails").await;
    enable_surveys(&pool, acme).await;
    let registry = registry(&pool);
    let usecase = usecase(&pool);
    let survey = draft(&usecase, acme, Visibility::Members, None).await;
    let spring = campaign(&usecase, acme).await;

    let error = failure(
        run(
            &registry,
            "mestier.survey.open",
            acme,
            &config(&[
                ("survey_id", json!(survey.0.to_string())),
                ("campaign_id", json!(spring.0.to_string())),
            ]),
        )
        .await,
    );

    assert!(!error.is_empty());
    let SurveyRecord::Draft(untouched) = stored(&usecase, acme, survey).await else {
        panic!("the survey is still a draft");
    };
    assert_eq!(untouched.campaign_id(), None);
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_members_survey_is_opened_in_its_campaign_and_has_no_public_link() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "survey-campaign").await;
    enable_surveys(&pool, acme).await;
    let registry = registry(&pool);
    let usecase = usecase(&pool);
    let spring = campaign(&usecase, acme).await;
    let survey = draft(&usecase, acme, Visibility::Members, Some(two_questions())).await;

    let opened = produced(
        run(
            &registry,
            "mestier.survey.open",
            acme,
            &config(&[
                ("survey_id", json!(survey.0.to_string())),
                ("campaign_id", json!(spring.0.to_string())),
            ]),
        )
        .await,
    );
    let link = failure(run(&registry, "mestier.survey.get_link", acme, &target(survey)).await);

    assert_eq!(opened["campaign_id"], json!(spring.0));
    assert_eq!(opened["public_slug"], json!(null));
    assert!(link.contains("MEMBERS"), "{link}");
}

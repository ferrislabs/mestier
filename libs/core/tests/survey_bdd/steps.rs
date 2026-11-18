use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use common::{CoreError, generate_uuid_v7};
use cucumber::{given, then, when};
use mestier_core::{
    Answer, Answers, CampaignId, CampaignPatch, MemberId, Metadata, OrganizationId, PublicSlug,
    QuestionId, QuestionKind, ResponseId, ResponseLocator, ResponseRecord, ResponseRepository,
    ResumeToken, SurveyBranch, SurveyEdge, SurveyEndNode, SurveyGraph, SurveyId, SurveyNode,
    SurveyPatch, SurveyQuestionNode, SurveyRecord, SurveyWindow, Visibility,
};
use serde_json::json;

use crate::{
    repository,
    world::{Respondent, SurveyWorld},
};

fn time(text: &str) -> DateTime<Utc> {
    text.parse()
        .unwrap_or_else(|_| panic!("the scenario writes the invalid instant `{text}`"))
}

fn visibility(word: &str) -> Visibility {
    match word {
        "PUBLIC" => Visibility::Public,
        "MEMBERS" => Visibility::Members,
        other => panic!("the scenario names an unknown visibility `{other}`"),
    }
}

fn describe(error: &CoreError) -> String {
    match error {
        CoreError::NotFound => "not found".to_owned(),
        CoreError::Conflict(message) => message.clone(),
        other => other.to_string(),
    }
}

fn survey_named(world: &SurveyWorld, title: &str) -> SurveyId {
    *world
        .surveys
        .get(title)
        .unwrap_or_else(|| panic!("the scenario never created the survey `{title}`"))
}

fn campaign_named(world: &SurveyWorld, name: &str) -> CampaignId {
    *world
        .campaigns
        .get(name)
        .unwrap_or_else(|| panic!("the scenario never created the campaign `{name}`"))
}

fn question_named(world: &SurveyWorld, survey: &str, label: &str) -> QuestionId {
    *world
        .questions
        .get(&(survey.to_owned(), label.to_owned()))
        .unwrap_or_else(|| panic!("the survey `{survey}` has no question `{label}`"))
}

fn stored_survey(world: &SurveyWorld, title: &str) -> SurveyRecord {
    let id = survey_named(world, title);
    repository::state(&world.store)
        .surveys
        .get(&id)
        .cloned()
        .expect("the survey was stored")
}

fn stored_campaign(world: &SurveyWorld, name: &str) -> mestier_core::Campaign {
    let id = campaign_named(world, name);
    repository::state(&world.store)
        .campaigns
        .get(&id)
        .cloned()
        .expect("the campaign was stored")
}

fn stored_response(world: &SurveyWorld, id: ResponseId) -> ResponseRecord {
    repository::state(&world.store)
        .responses
        .get(&id)
        .cloned()
        .expect("the response was stored")
}

fn response_of(world: &SurveyWorld, who: &str, survey: &str) -> ResponseId {
    *world
        .responses
        .get(&(who.to_owned(), survey.to_owned()))
        .unwrap_or_else(|| panic!("`{who}` never started a response to `{survey}`"))
}

fn answers_of(record: &ResponseRecord) -> &Answers {
    match record {
        ResponseRecord::InProgress(response) => response.answers(),
        ResponseRecord::Completed(response) => response.answers(),
    }
}

fn state_of(record: &SurveyRecord) -> &'static str {
    match record {
        SurveyRecord::Draft(_) => "DRAFT",
        SurveyRecord::Opened(_) => "OPENED",
        SurveyRecord::Closed(_) => "CLOSED",
    }
}

fn title_of(record: &SurveyRecord) -> &str {
    match record {
        SurveyRecord::Draft(survey) => survey.title(),
        SurveyRecord::Opened(survey) => survey.title(),
        SurveyRecord::Closed(survey) => survey.title(),
    }
}

fn with_entry<'a>(
    existing: impl Iterator<Item = (&'a str, &'a str)>,
    key: &str,
    value: &str,
) -> Metadata {
    let mut entries: BTreeMap<String, String> = existing
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
    entries.insert(key.to_owned(), value.to_owned());
    Metadata::new(entries).expect("the scenario writes a valid metadata entry")
}

fn keep<T>(world: &mut SurveyWorld, result: Result<T, CoreError>) {
    if let Err(error) = result {
        world.refusal = Some(error);
    }
}

fn definition(label: &str, identifier: bool) -> (SurveyGraph, QuestionId) {
    let question = QuestionId(generate_uuid_v7());
    let end = QuestionId(generate_uuid_v7());
    let graph = SurveyGraph::new(
        vec![
            SurveyNode::Question(SurveyQuestionNode {
                id: question,
                kind: QuestionKind::Text,
                label: label.to_owned(),
                required: true,
                options: Vec::new(),
                max_photos: None,
                identifier,
            }),
            SurveyNode::End(SurveyEndNode {
                id: end,
                message: None,
            }),
        ],
        vec![SurveyEdge {
            from: question,
            to: end,
            branch: SurveyBranch::Next,
        }],
        question,
    )
    .expect("the scenario draws a valid graph");
    (graph, question)
}

async fn draft_survey(
    world: &mut SurveyWorld,
    kind: &str,
    title: &str,
    label: &str,
    identifier: bool,
) {
    let id = SurveyId(generate_uuid_v7());
    let organization_id = world.organization_id();
    let (graph, question) = definition(label, identifier);
    world
        .surveys()
        .create(organization_id, id, title.to_owned(), visibility(kind))
        .await
        .expect("the survey is drafted");
    world
        .surveys()
        .update_draft(
            organization_id,
            id,
            SurveyPatch {
                graph: Some(Some(graph)),
                ..SurveyPatch::default()
            },
        )
        .await
        .expect("the graph is attached");
    world.surveys.insert(title.to_owned(), id);
    world
        .questions
        .insert((title.to_owned(), label.to_owned()), question);
}

async fn open_survey(world: &SurveyWorld, title: &str) -> Result<(), CoreError> {
    let id = survey_named(world, title);
    let slug = match stored_survey(world, title) {
        SurveyRecord::Draft(survey) if survey.visibility() == Visibility::Public => {
            Some(PublicSlug::new(format!("slug-{}", id.0.simple())).expect("a valid slug"))
        }
        _ => None,
    };
    world
        .surveys()
        .open(world.organization_id(), id, world.now(), slug)
        .await
        .map(|_| ())
}

async fn create_campaign(
    world: &mut SurveyWorld,
    name: &str,
    starts_at: &str,
    ends_at: &str,
) -> Result<(), CoreError> {
    let window =
        SurveyWindow::new(Some(time(starts_at)), Some(time(ends_at))).map_err(CoreError::from)?;
    let id = CampaignId(generate_uuid_v7());
    let created = world
        .campaign_service()
        .create(
            world.organization_id(),
            id,
            name.to_owned(),
            window,
            Metadata::default(),
            world.now(),
        )
        .await?;
    world.campaigns.insert(name.to_owned(), created.id());
    Ok(())
}

async fn start_response(
    world: &mut SurveyWorld,
    who: &str,
    survey: &str,
) -> Result<ResponseId, CoreError> {
    let organization_id = world.organization_id();
    let survey_id = survey_named(world, survey);
    let id = ResponseId(generate_uuid_v7());
    let respondent = world
        .respondents
        .get(who)
        .cloned()
        .unwrap_or_else(|| panic!("the scenario never introduced `{who}`"));
    let started = match respondent {
        Respondent::Member(member_id) => {
            world
                .responses()
                .start_member(organization_id, survey_id, id, member_id, world.now())
                .await?
        }
        Respondent::Visitor(token) => {
            world
                .responses()
                .start_public(organization_id, survey_id, id, token, world.now())
                .await?
        }
    };
    world
        .responses
        .insert((who.to_owned(), survey.to_owned()), started.id());
    Ok(started.id())
}

fn locator(world: &SurveyWorld, who: &str, survey: &str) -> ResponseLocator {
    match world.respondents.get(who) {
        Some(Respondent::Member(member_id)) => ResponseLocator::Member {
            id: response_of(world, who, survey),
            member_id: *member_id,
        },
        Some(Respondent::Visitor(token)) => ResponseLocator::Public {
            survey_id: survey_named(world, survey),
            token: token.clone(),
        },
        None => panic!("the scenario never introduced `{who}`"),
    }
}

async fn answer(
    world: &SurveyWorld,
    organization_id: OrganizationId,
    who: &str,
    survey: &str,
    text: &str,
    label: &str,
) -> Result<(), CoreError> {
    let question = question_named(world, survey, label);
    let answers = Answers::from([(question, Answer::Text(text.to_owned()))]);
    world
        .responses()
        .save(
            organization_id,
            &locator(world, who, survey),
            answers,
            world.now(),
        )
        .await
        .map(|_| ())
}

async fn complete(world: &SurveyWorld, who: &str, survey: &str) -> Result<(), CoreError> {
    world
        .responses()
        .complete(
            world.organization_id(),
            &locator(world, who, survey),
            world.now(),
        )
        .await
        .map(|_| ())
}

async fn complete_whole(
    world: &mut SurveyWorld,
    who: &str,
    survey: &str,
    text: &str,
    label: &str,
) -> Result<(), CoreError> {
    start_response(world, who, survey).await?;
    answer(world, world.organization_id(), who, survey, text, label).await?;
    complete(world, who, survey).await
}

#[given("a company registered on Mestier")]
async fn a_company(world: &mut SurveyWorld) {
    world.organization_id = Some(OrganizationId(generate_uuid_v7()));
}

#[given("another company registered on Mestier")]
async fn another_company(world: &mut SurveyWorld) {
    world.other_organization_id = Some(OrganizationId(generate_uuid_v7()));
}

#[given(expr = "a member {string} of the company")]
async fn a_member(world: &mut SurveyWorld, name: String) {
    world
        .respondents
        .insert(name, Respondent::Member(MemberId(generate_uuid_v7())));
}

#[given(expr = "a visitor {string}")]
async fn a_visitor(world: &mut SurveyWorld, name: String) {
    let token = ResumeToken::new(format!("token-{name}")).expect("a token");
    world.respondents.insert(name, Respondent::Visitor(token));
}

#[given(expr = "a draft {word} survey {string} with a required question {string}")]
async fn a_draft_survey(world: &mut SurveyWorld, kind: String, title: String, label: String) {
    draft_survey(world, &kind, &title, &label, false).await;
}

#[given(expr = "an opened {word} survey {string} with a required question {string}")]
async fn an_opened_survey(world: &mut SurveyWorld, kind: String, title: String, label: String) {
    draft_survey(world, &kind, &title, &label, false).await;
    open_survey(world, &title).await.expect("the survey opens");
}

#[given(expr = "an opened {word} survey {string} with a required identifying question {string}")]
async fn an_opened_identifying_survey(
    world: &mut SurveyWorld,
    kind: String,
    title: String,
    label: String,
) {
    draft_survey(world, &kind, &title, &label, true).await;
    open_survey(world, &title).await.expect("the survey opens");
}

#[given(expr = "the clock reads {string}")]
async fn the_clock_reads(world: &mut SurveyWorld, instant: String) {
    world.clock = Some(time(&instant));
}

#[given(expr = "a campaign {string} from {string} to {string}")]
async fn a_campaign(world: &mut SurveyWorld, name: String, from: String, to: String) {
    create_campaign(world, &name, &from, &to)
        .await
        .expect("the campaign is created");
}

#[given(expr = "the survey {string} belongs to the campaign {string}")]
async fn the_survey_belongs_to(world: &mut SurveyWorld, survey: String, campaign: String) {
    world
        .surveys()
        .attach_campaign(
            world.organization_id(),
            survey_named(world, &survey),
            Some(campaign_named(world, &campaign)),
        )
        .await
        .expect("the campaign is attached");
}

#[given(expr = "the survey {string} allows multiple responses")]
async fn the_survey_allows_multiple(world: &mut SurveyWorld, survey: String) {
    world
        .surveys()
        .update_draft(
            world.organization_id(),
            survey_named(world, &survey),
            SurveyPatch {
                allow_multiple_responses: Some(true),
                ..SurveyPatch::default()
            },
        )
        .await
        .expect("the draft is updated");
}

#[given(expr = "the survey {string} is opened")]
async fn the_survey_is_opened(world: &mut SurveyWorld, survey: String) {
    open_survey(world, &survey).await.expect("the survey opens");
}

#[given(expr = "the campaign {string} carries the metadata {string} equal to {string}")]
async fn the_campaign_carries(world: &mut SurveyWorld, name: String, key: String, value: String) {
    let metadata = with_entry(
        stored_campaign(world, &name).metadata().iter(),
        &key,
        &value,
    );
    world
        .campaign_service()
        .update(
            world.organization_id(),
            campaign_named(world, &name),
            CampaignPatch {
                metadata: Some(metadata),
                ..CampaignPatch::default()
            },
            world.now(),
        )
        .await
        .expect("the campaign is updated");
}

#[given(expr = "the survey {string} carries the metadata {string} equal to {string}")]
async fn the_survey_carries(world: &mut SurveyWorld, title: String, key: String, value: String) {
    let SurveyRecord::Draft(draft) = stored_survey(world, &title) else {
        panic!("the survey `{title}` is no longer a draft");
    };
    let metadata = with_entry(draft.metadata().iter(), &key, &value);
    world
        .surveys()
        .update_draft(
            world.organization_id(),
            survey_named(world, &title),
            SurveyPatch {
                metadata: Some(metadata),
                ..SurveyPatch::default()
            },
        )
        .await
        .expect("the draft is updated");
}

#[given(expr = "{string} has started a response to the survey {string}")]
async fn has_started(world: &mut SurveyWorld, who: String, survey: String) {
    start_response(world, &who, &survey)
        .await
        .expect("the response starts");
}

#[given(expr = "{string} has answered {string} to the question {string} of the survey {string}")]
async fn has_answered(
    world: &mut SurveyWorld,
    who: String,
    text: String,
    label: String,
    survey: String,
) {
    answer(world, world.organization_id(), &who, &survey, &text, &label)
        .await
        .expect("the answer is saved");
}

#[given(
    expr = "{string} has completed a response to the survey {string} answering {string} to the question {string}"
)]
async fn has_completed(
    world: &mut SurveyWorld,
    who: String,
    survey: String,
    text: String,
    label: String,
) {
    complete_whole(world, &who, &survey, &text, &label)
        .await
        .expect("the response completes");
}

#[when(expr = "the artisan opens the survey {string}")]
async fn the_artisan_opens(world: &mut SurveyWorld, survey: String) {
    let result = open_survey(world, &survey).await;
    keep(world, result);
}

#[when(expr = "the artisan retitles the survey {string} to {string}")]
async fn the_artisan_retitles(world: &mut SurveyWorld, survey: String, title: String) {
    let result = world
        .surveys()
        .update_draft(
            world.organization_id(),
            survey_named(world, &survey),
            SurveyPatch {
                title: Some(title),
                ..SurveyPatch::default()
            },
        )
        .await;
    keep(world, result);
}

#[when(expr = "the artisan creates the campaign {string} from {string} to {string}")]
async fn the_artisan_creates_a_campaign(
    world: &mut SurveyWorld,
    name: String,
    from: String,
    to: String,
) {
    let result = create_campaign(world, &name, &from, &to).await;
    keep(world, result);
}

#[when(expr = "the clock ticks at {string} on the campaign {string}")]
async fn the_clock_ticks(world: &mut SurveyWorld, instant: String, campaign: String) {
    world.clock = Some(time(&instant));
    let result = world
        .campaign_service()
        .advance(
            world.organization_id(),
            campaign_named(world, &campaign),
            world.now(),
        )
        .await;
    keep(world, result);
}

#[when(expr = "{string} starts a response to the survey {string}")]
async fn starts(world: &mut SurveyWorld, who: String, survey: String) {
    let result = start_response(world, &who, &survey).await;
    keep(world, result);
}

#[when(expr = "{string} answers {string} to the question {string} of the survey {string}")]
async fn answers(
    world: &mut SurveyWorld,
    who: String,
    text: String,
    label: String,
    survey: String,
) {
    let result = answer(world, world.organization_id(), &who, &survey, &text, &label).await;
    keep(world, result);
}

#[when(expr = "{string} completes the response to the survey {string}")]
async fn completes(world: &mut SurveyWorld, who: String, survey: String) {
    let result = complete(world, &who, &survey).await;
    keep(world, result);
}

#[when(
    expr = "{string} completes a response to the survey {string} answering {string} to the question {string}"
)]
async fn completes_a_response(
    world: &mut SurveyWorld,
    who: String,
    survey: String,
    text: String,
    label: String,
) {
    let result = complete_whole(world, &who, &survey, &text, &label).await;
    keep(world, result);
}

#[when(expr = "{string} comes back to the survey {string} with her resume token")]
async fn comes_back(world: &mut SurveyWorld, who: String, survey: String) {
    let key = (who.clone(), survey.clone());
    let previous = world.responses.get(&key).copied();
    match start_response(world, &who, &survey).await {
        Ok(resumed) => {
            world.resumed = Some(resumed);
            if let Some(previous) = previous {
                world.responses.insert(key, previous);
            }
        }
        Err(error) => world.refusal = Some(error),
    }
}

#[when(expr = "the other company lists the responses of the survey {string}")]
async fn the_other_company_lists(world: &mut SurveyWorld, survey: String) {
    let (listed, _) = repository::responses(&world.store)
        .list_for_survey(
            world.other_organization_id(),
            survey_named(world, &survey),
            100,
            0,
        )
        .await
        .expect("no infrastructure failure here");
    world.listed = Some(listed.len());
}

#[when(
    expr = "the other company lets {string} answer {string} to the question {string} of the survey {string}"
)]
async fn the_other_company_answers(
    world: &mut SurveyWorld,
    who: String,
    text: String,
    label: String,
    survey: String,
) {
    let result = answer(
        world,
        world.other_organization_id(),
        &who,
        &survey,
        &text,
        &label,
    )
    .await;
    keep(world, result);
}

#[when(expr = "the other company deletes the survey {string}")]
async fn the_other_company_deletes(world: &mut SurveyWorld, survey: String) {
    let result = world
        .surveys()
        .delete(world.other_organization_id(), survey_named(world, &survey))
        .await;
    keep(world, result);
}

#[then(expr = "the survey {string} is {word}")]
async fn the_survey_is(world: &mut SurveyWorld, survey: String, state: String) {
    assert_eq!(state_of(&stored_survey(world, &survey)), state);
}

#[then(expr = "the survey {string} is titled {string}")]
async fn the_survey_is_titled(world: &mut SurveyWorld, survey: String, title: String) {
    assert_eq!(title_of(&stored_survey(world, &survey)), title);
}

#[then(expr = "the survey {string} has a frozen definition of {int} nodes")]
async fn the_survey_has_a_frozen_definition(world: &mut SurveyWorld, survey: String, nodes: usize) {
    let SurveyRecord::Opened(opened) = stored_survey(world, &survey) else {
        panic!("the survey `{survey}` is not opened");
    };
    assert_eq!(opened.definition().nodes().count(), nodes);
}

#[then(expr = "the company is notified of the event {string}")]
async fn notified_of(world: &mut SurveyWorld, name: String) {
    world.emitter.only(&name);
}

#[then(expr = "the company is notified of no {string} event")]
async fn notified_of_none(world: &mut SurveyWorld, name: String) {
    assert!(
        !world.emitter.names().contains(&name),
        "unexpected `{name}` in {:?}",
        world.emitter.names()
    );
}

#[then(expr = "the company is notified of the campaign events {string} then {string}")]
async fn notified_of_campaign_events(world: &mut SurveyWorld, first: String, second: String) {
    let names: Vec<String> = world
        .emitter
        .names()
        .into_iter()
        .filter(|name| name.starts_with("survey_campaign."))
        .collect();
    assert_eq!(names, vec![first, second]);
}

#[then(expr = "the company is notified of {int} survey events")]
async fn notified_of_survey_events(world: &mut SurveyWorld, count: usize) {
    let names = world.emitter.names();
    let matching = names
        .iter()
        .filter(|name| name.starts_with("survey."))
        .count();
    assert_eq!(matching, count, "{names:?}");
}

#[then(expr = "the company is notified of {int} campaign events")]
async fn notified_of_campaign_count(world: &mut SurveyWorld, count: usize) {
    let names = world.emitter.names();
    let matching = names
        .iter()
        .filter(|name| name.starts_with("survey_campaign."))
        .count();
    assert_eq!(matching, count, "{names:?}");
}

#[then(expr = "the event {string} carries {string} equal to {string}")]
async fn event_carries(world: &mut SurveyWorld, name: String, pointer: String, expected: String) {
    let envelope = world.emitter.only(&name);
    assert_eq!(envelope.payload.pointer(&pointer), Some(&json!(expected)));
}

#[then(expr = "every survey event carries the id of the survey {string} at {string}")]
async fn every_survey_event_carries_survey(
    world: &mut SurveyWorld,
    survey: String,
    pointer: String,
) {
    let expected = json!(survey_named(world, &survey).0);
    every_event_carries(world, "survey.", &pointer, &expected);
}

#[then(expr = "every survey event carries the id of the campaign {string} at {string}")]
async fn every_survey_event_carries_campaign(
    world: &mut SurveyWorld,
    campaign: String,
    pointer: String,
) {
    let expected = json!(campaign_named(world, &campaign).0);
    every_event_carries(world, "survey.", &pointer, &expected);
}

#[then(expr = "every survey event carries a null {string}")]
async fn every_survey_event_carries_null(world: &mut SurveyWorld, pointer: String) {
    every_event_carries(world, "survey.", &pointer, &serde_json::Value::Null);
}

#[then(expr = "every campaign event carries the id of the campaign {string} at {string}")]
async fn every_campaign_event_carries(world: &mut SurveyWorld, campaign: String, pointer: String) {
    let expected = json!(campaign_named(world, &campaign).0);
    every_event_carries(world, "survey_campaign.", &pointer, &expected);
}

fn every_event_carries(
    world: &SurveyWorld,
    prefix: &str,
    pointer: &str,
    expected: &serde_json::Value,
) {
    let envelopes: Vec<_> = world
        .emitter
        .recorded()
        .into_iter()
        .filter(|envelope| envelope.name.starts_with(prefix))
        .collect();
    assert!(!envelopes.is_empty(), "no `{prefix}` event was recorded");
    for envelope in envelopes {
        assert_eq!(
            envelope.payload.pointer(pointer),
            Some(expected),
            "{}",
            envelope.name
        );
    }
}

#[then(expr = "the refusal is {string}")]
async fn the_refusal_is(world: &mut SurveyWorld, message: String) {
    let refusal = world.refusal.as_ref().expect("the action was refused");
    assert_eq!(describe(refusal), message);
}

#[then(expr = "the refusal mentions {string}")]
async fn the_refusal_mentions(world: &mut SurveyWorld, fragment: String) {
    let refusal = world.refusal.as_ref().expect("the action was refused");
    assert!(
        describe(refusal).contains(&fragment),
        "`{}` does not mention `{fragment}`",
        describe(refusal)
    );
}

#[then("nothing is refused")]
async fn nothing_is_refused(world: &mut SurveyWorld) {
    assert!(
        world.refusal.is_none(),
        "unexpected refusal {:?}",
        world.refusal
    );
}

#[then(
    expr = "the response of {string} to the survey {string} holds the answer {string} to the question {string}"
)]
async fn the_response_holds(
    world: &mut SurveyWorld,
    who: String,
    survey: String,
    text: String,
    label: String,
) {
    let record = stored_response(world, response_of(world, &who, &survey));
    let question = question_named(world, &survey, &label);
    assert_eq!(
        answers_of(&record).get(&question),
        Some(&Answer::Text(text))
    );
}

#[then(expr = "the response of {string} to the survey {string} holds no answer")]
async fn the_response_holds_none(world: &mut SurveyWorld, who: String, survey: String) {
    let record = stored_response(world, response_of(world, &who, &survey));
    assert!(answers_of(&record).is_empty());
}

#[then(expr = "the survey {string} holds {int} completed response(s)")]
async fn holds_completed(world: &mut SurveyWorld, survey: String, count: usize) {
    let id = survey_named(world, &survey);
    let completed = repository::state(&world.store)
        .responses
        .values()
        .filter(|record| record.survey_id() == id)
        .filter(|record| matches!(record, ResponseRecord::Completed(_)))
        .count();
    assert_eq!(completed, count);
}

#[then(expr = "the survey {string} holds {int} responses")]
async fn holds_responses(world: &mut SurveyWorld, survey: String, count: usize) {
    let id = survey_named(world, &survey);
    let stored = repository::state(&world.store)
        .responses
        .values()
        .filter(|record| record.survey_id() == id)
        .count();
    assert_eq!(stored, count);
}

#[then(expr = "the campaign {string} is {word}")]
async fn the_campaign_is(world: &mut SurveyWorld, campaign: String, status: String) {
    assert_eq!(stored_campaign(world, &campaign).status().as_str(), status);
}

#[then("no campaign is stored")]
async fn no_campaign_is_stored(world: &mut SurveyWorld) {
    assert!(repository::state(&world.store).campaigns.is_empty());
}

#[then(expr = "the resumed response is the one {string} started on the survey {string}")]
async fn the_resumed_response_is(world: &mut SurveyWorld, who: String, survey: String) {
    assert_eq!(world.resumed, Some(response_of(world, &who, &survey)));
}

#[then(
    expr = "the resumed response holds the answer {string} to the question {string} of the survey {string}"
)]
async fn the_resumed_response_holds(
    world: &mut SurveyWorld,
    text: String,
    label: String,
    survey: String,
) {
    let id = world.resumed.expect("a response was resumed");
    let question = question_named(world, &survey, &label);
    assert_eq!(
        answers_of(&stored_response(world, id)).get(&question),
        Some(&Answer::Text(text))
    );
}

#[then(expr = "the other company sees {int} responses")]
async fn the_other_company_sees(world: &mut SurveyWorld, count: usize) {
    assert_eq!(world.listed, Some(count));
}

#[then(expr = "the company sees {int} response(s) of the survey {string}")]
async fn the_company_sees(world: &mut SurveyWorld, count: usize, survey: String) {
    let (listed, _) = repository::responses(&world.store)
        .list_for_survey(
            world.organization_id(),
            survey_named(world, &survey),
            100,
            0,
        )
        .await
        .expect("no infrastructure failure here");
    assert_eq!(listed.len(), count);
}

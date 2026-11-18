Feature: Surveys, the questionnaires answered by members and by the public

  A survey is drafted, then opened, which freezes its graph of questions. A
  respondent starts a response, saves it as often as needed and completes it.
  A survey may belong to a campaign whose window decides when it accepts
  responses, and the clock of the campaigns announces when a window starts and
  ends.

  Background:
    Given a company registered on Mestier
    And a member "Claire" of the company

  @spec-survey-4
  Scenario: Opening a survey freezes its definition and emits survey.opened
    Given a draft MEMBERS survey "Satisfaction" with a required question "Nom"
    When the artisan opens the survey "Satisfaction"
    Then the survey "Satisfaction" is OPENED
    And the survey "Satisfaction" has a frozen definition of 2 nodes
    And the company is notified of the event "survey.opened"
    And the event "survey.opened" carries "/survey/title" equal to "Satisfaction"

  @spec-survey-4
  Scenario: An opened survey refuses edits to its definition
    Given an opened MEMBERS survey "Satisfaction" with a required question "Nom"
    When the artisan retitles the survey "Satisfaction" to "Autre titre"
    Then the refusal is "survey: is not a draft"
    And the survey "Satisfaction" is titled "Satisfaction"

  @spec-survey-5
  Scenario: Saving an in-progress response emits survey.response_submitted
    Given an opened MEMBERS survey "Satisfaction" with a required question "Nom"
    And "Claire" has started a response to the survey "Satisfaction"
    When "Claire" answers "Dupont" to the question "Nom" of the survey "Satisfaction"
    Then the company is notified of the event "survey.response_submitted"
    And the event "survey.response_submitted" carries "/response/status" equal to "IN_PROGRESS"
    And the response of "Claire" to the survey "Satisfaction" holds the answer "Dupont" to the question "Nom"

  @spec-survey-6
  Scenario: Completing a response emits survey.response_completed with all the answers
    Given an opened MEMBERS survey "Satisfaction" with a required question "Nom"
    And "Claire" has started a response to the survey "Satisfaction"
    And "Claire" has answered "Dupont" to the question "Nom" of the survey "Satisfaction"
    When "Claire" completes the response to the survey "Satisfaction"
    Then the company is notified of the event "survey.response_completed"
    And the event "survey.response_completed" carries "/answers/0/label" equal to "Nom"
    And the event "survey.response_completed" carries "/answers/0/value" equal to "Dupont"
    And the survey "Satisfaction" holds 1 completed response

  @spec-survey-6
  Scenario: Completing a response a second time emits nothing more
    Given an opened MEMBERS survey "Satisfaction" with a required question "Nom"
    And "Claire" has completed a response to the survey "Satisfaction" answering "Dupont" to the question "Nom"
    When "Claire" completes the response to the survey "Satisfaction"
    Then the refusal is "response: is already complete"
    And the company is notified of the event "survey.response_completed"

  @spec-survey-7
  Scenario: Completing without answering a required question is refused
    Given an opened MEMBERS survey "Satisfaction" with a required question "Nom"
    And "Claire" has started a response to the survey "Satisfaction"
    When "Claire" completes the response to the survey "Satisfaction"
    Then the refusal mentions "is unanswered"
    And the company is notified of no "survey.response_completed" event
    And the survey "Satisfaction" holds 0 completed responses

  @spec-survey-8
  Scenario: A completed response can no longer be modified
    Given an opened MEMBERS survey "Satisfaction" with a required question "Nom"
    And "Claire" has completed a response to the survey "Satisfaction" answering "Dupont" to the question "Nom"
    When "Claire" answers "Martin" to the question "Nom" of the survey "Satisfaction"
    Then the refusal is "response: is already complete"
    And the response of "Claire" to the survey "Satisfaction" holds the answer "Dupont" to the question "Nom"

  @spec-survey-9
  Scenario: A campaign survey refuses a response before the window starts
    Given the clock reads "2026-02-28T23:59:59Z"
    And a campaign "Printemps" from "2026-03-01T00:00:00Z" to "2026-03-31T00:00:00Z"
    And a draft MEMBERS survey "Satisfaction" with a required question "Nom"
    And the survey "Satisfaction" belongs to the campaign "Printemps"
    And the survey "Satisfaction" is opened
    When "Claire" starts a response to the survey "Satisfaction"
    Then the refusal is "survey: its campaign has not started"
    And the survey "Satisfaction" holds 0 responses

  @spec-survey-10
  Scenario: A campaign survey refuses a new response after the window ends
    Given the clock reads "2026-03-10T12:00:00Z"
    And a campaign "Printemps" from "2026-03-01T00:00:00Z" to "2026-03-31T00:00:00Z"
    And a draft MEMBERS survey "Satisfaction" with a required question "Nom"
    And the survey "Satisfaction" belongs to the campaign "Printemps"
    And the survey "Satisfaction" is opened
    And the clock reads "2026-03-31T00:00:00Z"
    When "Claire" starts a response to the survey "Satisfaction"
    Then the refusal is "survey: its campaign has ended"
    And the survey "Satisfaction" holds 0 responses

  @spec-survey-10
  Scenario: A campaign survey refuses to save a response after the window ends
    Given the clock reads "2026-03-10T12:00:00Z"
    And a campaign "Printemps" from "2026-03-01T00:00:00Z" to "2026-03-31T00:00:00Z"
    And a draft MEMBERS survey "Satisfaction" with a required question "Nom"
    And the survey "Satisfaction" belongs to the campaign "Printemps"
    And the survey "Satisfaction" is opened
    And "Claire" has started a response to the survey "Satisfaction"
    And the clock reads "2026-03-31T00:00:00Z"
    When "Claire" answers "Dupont" to the question "Nom" of the survey "Satisfaction"
    Then the refusal is "survey: its campaign has ended"
    And the response of "Claire" to the survey "Satisfaction" holds no answer

  @spec-survey-11
  Scenario: The clock starts the campaign at its start date and emits survey_campaign.started
    Given the clock reads "2026-02-20T12:00:00Z"
    And a campaign "Printemps" from "2026-03-01T00:00:00Z" to "2026-03-31T00:00:00Z"
    When the clock ticks at "2026-03-01T00:00:00Z" on the campaign "Printemps"
    Then the campaign "Printemps" is RUNNING
    And the company is notified of the event "survey_campaign.started"
    And the event "survey_campaign.started" carries "/campaign/name" equal to "Printemps"
    And the company is notified of no "survey_campaign.ended" event

  @spec-survey-11
  Scenario: The clock leaves the campaign scheduled before its start date
    Given the clock reads "2026-02-20T12:00:00Z"
    And a campaign "Printemps" from "2026-03-01T00:00:00Z" to "2026-03-31T00:00:00Z"
    When the clock ticks at "2026-02-28T23:59:59Z" on the campaign "Printemps"
    Then the campaign "Printemps" is SCHEDULED
    And the company is notified of no "survey_campaign.started" event

  @spec-survey-12
  Scenario: The clock ends the campaign at its end date and emits survey_campaign.ended
    Given the clock reads "2026-03-10T12:00:00Z"
    And a campaign "Printemps" from "2026-03-01T00:00:00Z" to "2026-03-31T00:00:00Z"
    When the clock ticks at "2026-03-31T00:00:00Z" on the campaign "Printemps"
    Then the campaign "Printemps" is ENDED
    And the company is notified of the event "survey_campaign.ended"
    And the company is notified of the campaign events "survey_campaign.started" then "survey_campaign.ended"

  @spec-survey-12
  Scenario: A campaign that ended before the clock noticed it is started then ended
    Given the clock reads "2026-02-20T12:00:00Z"
    And a campaign "Printemps" from "2026-03-01T00:00:00Z" to "2026-03-31T00:00:00Z"
    When the clock ticks at "2026-04-15T12:00:00Z" on the campaign "Printemps"
    Then the campaign "Printemps" is ENDED
    And the company is notified of the campaign events "survey_campaign.started" then "survey_campaign.ended"

  @spec-survey-12
  Scenario: A campaign created after its end is started then ended
    Given the clock reads "2026-04-15T12:00:00Z"
    When the artisan creates the campaign "Printemps" from "2026-03-01T00:00:00Z" to "2026-03-31T00:00:00Z"
    Then the campaign "Printemps" is ENDED
    And the company is notified of the campaign events "survey_campaign.started" then "survey_campaign.ended"

  @spec-survey-13
  Scenario: A campaign whose end precedes its start is refused
    When the artisan creates the campaign "Inversée" from "2026-03-31T00:00:00Z" to "2026-03-01T00:00:00Z"
    Then the refusal is "window: the start is not before the end"
    And no campaign is stored
    And the company is notified of no "survey_campaign.started" event

  @spec-survey-16
  Scenario: The resume token lets a visitor resume an in-progress public response
    Given an opened PUBLIC survey "Satisfaction" with a required question "Nom"
    And a visitor "Anne"
    And "Anne" has started a response to the survey "Satisfaction"
    And "Anne" has answered "Dupont" to the question "Nom" of the survey "Satisfaction"
    When "Anne" comes back to the survey "Satisfaction" with her resume token
    Then the resumed response is the one "Anne" started on the survey "Satisfaction"
    And the resumed response holds the answer "Dupont" to the question "Nom" of the survey "Satisfaction"

  @spec-survey-17
  Scenario: Another company sees none of the responses of a survey
    Given another company registered on Mestier
    And an opened MEMBERS survey "Satisfaction" with a required question "Nom"
    And "Claire" has started a response to the survey "Satisfaction"
    When the other company lists the responses of the survey "Satisfaction"
    Then the other company sees 0 responses
    And the company sees 1 response of the survey "Satisfaction"

  @spec-survey-17
  Scenario: Only the completed responses are listed, the most recent first
    Given another company registered on Mestier
    And an opened PUBLIC survey "Satisfaction" with a required question "Nom"
    And a visitor "Anne"
    And a visitor "Bruno"
    And a visitor "Chloé"
    And a visitor "Denis"
    And a visitor "Emma"
    And the clock reads "2026-03-10T09:00:00Z"
    And "Anne" has completed a response to the survey "Satisfaction" answering "Arnaud" to the question "Nom"
    And the clock reads "2026-03-10T11:00:00Z"
    And "Bruno" has completed a response to the survey "Satisfaction" answering "Bernard" to the question "Nom"
    And the clock reads "2026-03-10T10:00:00Z"
    And "Chloé" has completed a response to the survey "Satisfaction" answering "Colin" to the question "Nom"
    And "Denis" has started a response to the survey "Satisfaction"
    And "Emma" has started a response to the survey "Satisfaction"
    Then the company lists 3 completed responses of the survey "Satisfaction" answering "Bernard, Colin, Arnaud"
    And the other company lists 0 completed responses of the survey "Satisfaction" answering ""

  @spec-survey-17
  Scenario: Another company cannot answer on behalf of a member
    Given another company registered on Mestier
    And an opened MEMBERS survey "Satisfaction" with a required question "Nom"
    And "Claire" has started a response to the survey "Satisfaction"
    And "Claire" has answered "Dupont" to the question "Nom" of the survey "Satisfaction"
    When the other company lets "Claire" answer "Intrus" to the question "Nom" of the survey "Satisfaction"
    Then the refusal is "not found"
    And the response of "Claire" to the survey "Satisfaction" holds the answer "Dupont" to the question "Nom"

  @spec-survey-17
  Scenario: Another company cannot delete a survey
    Given another company registered on Mestier
    And an opened MEMBERS survey "Satisfaction" with a required question "Nom"
    When the other company deletes the survey "Satisfaction"
    Then the refusal is "not found"
    And the survey "Satisfaction" is OPENED

  @spec-survey-18
  Scenario: A member cannot answer twice a survey that refuses multiple responses
    Given an opened MEMBERS survey "Satisfaction" with a required question "Nom"
    And "Claire" has completed a response to the survey "Satisfaction" answering "Dupont" to the question "Nom"
    When "Claire" starts a response to the survey "Satisfaction"
    Then the refusal is "response: the respondent has already answered this survey"
    And the survey "Satisfaction" holds 1 completed response

  @spec-survey-19
  Scenario: A member answers again a survey that allows multiple responses
    Given a draft MEMBERS survey "Satisfaction" with a required question "Nom"
    And the survey "Satisfaction" allows multiple responses
    And the survey "Satisfaction" is opened
    And "Claire" has completed a response to the survey "Satisfaction" answering "Dupont" to the question "Nom"
    When "Claire" completes a response to the survey "Satisfaction" answering "Martin" to the question "Nom"
    Then nothing is refused
    And the survey "Satisfaction" holds 2 completed responses

  @spec-survey-20
  Scenario: A public response with an identifier already used is refused
    Given an opened PUBLIC survey "Satisfaction" with a required identifying question "Courriel"
    And a visitor "Anne"
    And a visitor "Bruno"
    And "Anne" has completed a response to the survey "Satisfaction" answering "Dupont" to the question "Courriel"
    When "Bruno" completes a response to the survey "Satisfaction" answering " dupont " to the question "Courriel"
    Then the refusal is "response: the respondent has already answered this survey"
    And the survey "Satisfaction" holds 1 completed response

  @spec-survey-24
  Scenario: The metadata of the survey and of its campaign travel in the opening event
    Given a campaign "Printemps" from "2026-03-01T00:00:00Z" to "2026-03-31T00:00:00Z"
    And the campaign "Printemps" carries the metadata "season" equal to "spring"
    And the campaign "Printemps" carries the metadata "theme" equal to "campaign"
    And a draft MEMBERS survey "Satisfaction" with a required question "Nom"
    And the survey "Satisfaction" belongs to the campaign "Printemps"
    And the survey "Satisfaction" carries the metadata "theme" equal to "survey"
    When the artisan opens the survey "Satisfaction"
    Then the event "survey.opened" carries "/metadata/season" equal to "spring"
    And the event "survey.opened" carries "/metadata/theme" equal to "survey"

  @spec-survey-24
  Scenario: The metadata of the survey and of its campaign travel in the response events
    Given a campaign "Printemps" from "2026-03-01T00:00:00Z" to "2026-03-31T00:00:00Z"
    And the campaign "Printemps" carries the metadata "season" equal to "spring"
    And a draft MEMBERS survey "Satisfaction" with a required question "Nom"
    And the survey "Satisfaction" belongs to the campaign "Printemps"
    And the survey "Satisfaction" carries the metadata "theme" equal to "survey"
    And the survey "Satisfaction" is opened
    When "Claire" completes a response to the survey "Satisfaction" answering "Dupont" to the question "Nom"
    Then the event "survey.response_submitted" carries "/metadata/season" equal to "spring"
    And the event "survey.response_completed" carries "/metadata/season" equal to "spring"
    And the event "survey.response_completed" carries "/metadata/theme" equal to "survey"

  @spec-survey-25
  Scenario: Every event of a standalone survey carries its id and a null campaign
    Given an opened MEMBERS survey "Satisfaction" with a required question "Nom"
    When "Claire" completes a response to the survey "Satisfaction" answering "Dupont" to the question "Nom"
    Then the company is notified of 3 survey events
    And every survey event carries the id of the survey "Satisfaction" at "/survey_id"
    And every survey event carries a null "/campaign_id"

  @spec-survey-25
  Scenario: Every event of a campaign survey carries its id and the id of its campaign
    Given a campaign "Printemps" from "2026-03-01T00:00:00Z" to "2026-03-31T00:00:00Z"
    And a draft MEMBERS survey "Satisfaction" with a required question "Nom"
    And the survey "Satisfaction" belongs to the campaign "Printemps"
    And the survey "Satisfaction" is opened
    When "Claire" completes a response to the survey "Satisfaction" answering "Dupont" to the question "Nom"
    Then the company is notified of 3 survey events
    And every survey event carries the id of the survey "Satisfaction" at "/survey_id"
    And every survey event carries the id of the campaign "Printemps" at "/campaign_id"

  @spec-survey-26
  Scenario: Every event of a campaign carries its id
    Given the clock reads "2026-02-20T12:00:00Z"
    And a campaign "Printemps" from "2026-03-01T00:00:00Z" to "2026-03-31T00:00:00Z"
    When the clock ticks at "2026-04-15T12:00:00Z" on the campaign "Printemps"
    Then the company is notified of 2 campaign events
    And every campaign event carries the id of the campaign "Printemps" at "/campaign_id"

use common::{CoreError, Refusal, RefusalKind};

use super::value::QuestionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SurveyError {
    #[error("email: not a valid email address")]
    EmailInvalid,
    #[error("phone: not a valid phone number")]
    PhoneInvalid,
    #[error("graph: has no node")]
    GraphEmpty,
    #[error("graph: has no end node")]
    GraphWithoutEnd,
    #[error("graph: two nodes share the id {0}")]
    GraphDuplicateNode(QuestionId),
    #[error("graph: the start is not a node of the graph")]
    GraphUnknownStart,
    #[error("graph: the start is not a question")]
    GraphStartNotQuestion,
    #[error("graph: an edge points at the unknown node {0}")]
    GraphUnknownEdgeNode(QuestionId),
    #[error("graph: node {0} has an edge with a branch its kind does not allow")]
    GraphInvalidBranch(QuestionId),
    #[error("graph: node {0} has two edges on the same branch")]
    GraphDuplicateExit(QuestionId),
    #[error("graph: node {0} has no way to an end")]
    GraphDeadEnd(QuestionId),
    #[error("graph: node {0} closes a cycle")]
    GraphCycle(QuestionId),
    #[error("graph: node {0} is unreachable from the start")]
    GraphUnreachableNode(QuestionId),
    #[error("graph: question {0} is a second identifier")]
    GraphSecondIdentifier(QuestionId),
    #[error("graph: question {0} is an identifier but is not an email, phone or text question")]
    GraphIdentifierOnInvalidKind(QuestionId),
    #[error("graph: question {0} has a photo limit but is not a photo question")]
    GraphMaxPhotosOnInvalidKind(QuestionId),
    #[error("graph: question {0} has a photo limit of zero")]
    GraphZeroMaxPhotos(QuestionId),
    #[error("graph: choice question {0} has no option")]
    GraphChoiceWithoutOptions(QuestionId),
    #[error("graph: question {0} has an empty label")]
    GraphQuestionLabelEmpty(QuestionId),
    #[error("graph: question {0} has an empty option")]
    GraphOptionEmpty(QuestionId),
    #[error("graph: question {0} repeats an option")]
    GraphDuplicateOption(QuestionId),
    #[error("graph: condition {0} refers to an unknown question")]
    ConditionUnknownQuestion(QuestionId),
    #[error("graph: condition {0} refers to a node that is not a choice question")]
    ConditionOnNonChoice(QuestionId),
    #[error("graph: condition {0} refers to an unknown option")]
    ConditionUnknownOption(QuestionId),
    #[error("graph: condition {0} refers to a question not asked on every path to it")]
    ConditionQuestionNotAsked(QuestionId),
    #[error("graph: question {0} is the only question of its block")]
    GraphBlockTooSmall(QuestionId),
    #[error("graph: question {0} does not follow the previous question of its block")]
    GraphBlockNotContiguous(QuestionId),
    #[error("graph: question {0} is guarded by a condition inside its block")]
    GraphBlockMemberHasCondition(QuestionId),
    #[error("window: the start is not before the end")]
    WindowInverted,
    #[error("metadata: a key is empty")]
    MetadataEmptyKey,
    #[error("metadata: the value of a key is empty")]
    MetadataEmptyValue,
    #[error("survey: the title is empty")]
    SurveyTitleEmpty,
    #[error("survey: has no graph")]
    SurveyWithoutGraph,
    #[error("survey: is closed")]
    SurveyClosed,
    #[error("survey: its campaign has not started")]
    CampaignNotStarted,
    #[error("survey: its campaign has ended")]
    CampaignEnded,
    #[error("survey: the stored state does not match its status")]
    SurveyInconsistentState,
    #[error("response: the resume token is empty")]
    ResumeTokenEmpty,
    #[error("response: required question {0} is unanswered")]
    RequiredQuestionUnanswered(QuestionId),
    #[error("response: the answer to question {0} is not a finite number")]
    AnswerNotFinite(QuestionId),
    #[error("response: the stored state does not match its status")]
    ResponseInconsistentState,
    #[error("survey: cannot be closed before it was opened")]
    SurveyClosedBeforeOpened,
    #[error("response: question {0} is not a question of the survey")]
    AnswerUnknownQuestion(QuestionId),
    #[error("response: the answer to question {0} does not match its kind")]
    AnswerKindMismatch(QuestionId),
    #[error("response: the answer to question {0} names an unknown option")]
    AnswerUnknownOption(QuestionId),
    #[error("response: the answer to question {0} repeats an option")]
    AnswerDuplicateOption(QuestionId),
    #[error("response: the answer to question {0} has too many photos")]
    AnswerTooManyPhotos(QuestionId),
    #[error("response: the answer to question {0} has an empty photo")]
    AnswerEmptyPhoto(QuestionId),
    #[error(
        "response: the answer to question {0} names a photo that was not uploaded for this response"
    )]
    AnswerForeignPhoto(QuestionId),
    #[error("public_slug: must be 8 to 64 ASCII letters, digits, dashes or underscores")]
    PublicSlugInvalid,
    #[error("survey: a public survey needs a public slug to open")]
    SurveyPublicSlugRequired,
    #[error("survey: a members survey cannot have a public slug")]
    SurveyPublicSlugForbidden,
    #[error("survey: the campaign is not the campaign of the survey")]
    SurveyCampaignMismatch,
    #[error("survey: the campaign belongs to another organization")]
    SurveyCampaignOrganizationMismatch,
    #[error("campaign: the name is empty")]
    CampaignNameEmpty,
    #[error("response: belongs to another survey")]
    ResponseSurveyMismatch,
    #[error("response: a members survey needs a member")]
    ResponseMemberRequired,
    #[error("response: a public survey cannot record a member")]
    ResponseMemberForbidden,
    #[error("response: a public survey needs a resume token")]
    ResponseResumeTokenRequired,
    #[error("response: a members survey cannot use a resume token")]
    ResponseResumeTokenForbidden,
    #[error("survey: not found")]
    SurveyNotFound,
    #[error("campaign: not found")]
    CampaignNotFound,
    #[error("response: not found")]
    ResponseNotFound,
    #[error("survey: is not a draft")]
    SurveyNotDraft,
    #[error("survey: is not opened")]
    SurveyNotOpened,
    #[error("survey: has responses and cannot be deleted")]
    SurveyHasResponses,
    #[error("campaign: has surveys and cannot be deleted")]
    CampaignHasSurveys,
    #[error("response: is already complete")]
    ResponseAlreadyCompleted,
    #[error("response: the survey has reached its response cap")]
    ResponseCapReached,
    #[error("response: the respondent has already answered this survey")]
    RespondentAlreadyAnswered,
    #[error("survey: its state changed since it was read")]
    SurveyStateChanged,
    #[error("response: has reached its photo upload cap")]
    ResponsePhotoCapReached,
    #[error("survey: its visibility cannot change once it was opened")]
    SurveyVisibilityFrozen,
    #[error("survey: a graph version is a positive number")]
    GraphVersionInvalid,
    #[error("survey: graph version not found")]
    GraphVersionNotFound,
    #[error("survey: its graph has reached the last version number")]
    GraphVersionExhausted,
    #[error("response: was given on a graph version other than the current one")]
    ResponseGraphVersionMismatch,
}

impl SurveyError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::EmailInvalid => "email_invalid",
            Self::PhoneInvalid => "phone_invalid",
            Self::GraphEmpty => "graph_empty",
            Self::GraphWithoutEnd => "graph_without_end",
            Self::GraphDuplicateNode(_) => "graph_duplicate_node",
            Self::GraphUnknownStart => "graph_unknown_start",
            Self::GraphStartNotQuestion => "graph_start_not_question",
            Self::GraphUnknownEdgeNode(_) => "graph_unknown_edge_node",
            Self::GraphInvalidBranch(_) => "graph_invalid_branch",
            Self::GraphDuplicateExit(_) => "graph_duplicate_exit",
            Self::GraphDeadEnd(_) => "graph_dead_end",
            Self::GraphCycle(_) => "graph_cycle",
            Self::GraphUnreachableNode(_) => "graph_unreachable_node",
            Self::GraphSecondIdentifier(_) => "graph_second_identifier",
            Self::GraphIdentifierOnInvalidKind(_) => "graph_identifier_on_invalid_kind",
            Self::GraphMaxPhotosOnInvalidKind(_) => "graph_max_photos_on_invalid_kind",
            Self::GraphZeroMaxPhotos(_) => "graph_zero_max_photos",
            Self::GraphChoiceWithoutOptions(_) => "graph_choice_without_options",
            Self::GraphQuestionLabelEmpty(_) => "graph_question_label_empty",
            Self::GraphOptionEmpty(_) => "graph_option_empty",
            Self::GraphDuplicateOption(_) => "graph_duplicate_option",
            Self::ConditionUnknownQuestion(_) => "condition_unknown_question",
            Self::ConditionOnNonChoice(_) => "condition_on_non_choice",
            Self::ConditionUnknownOption(_) => "condition_unknown_option",
            Self::ConditionQuestionNotAsked(_) => "condition_question_not_asked",
            Self::GraphBlockTooSmall(_) => "graph_block_too_small",
            Self::GraphBlockNotContiguous(_) => "graph_block_not_contiguous",
            Self::GraphBlockMemberHasCondition(_) => "graph_block_member_has_condition",
            Self::WindowInverted => "window_inverted",
            Self::MetadataEmptyKey => "metadata_empty_key",
            Self::MetadataEmptyValue => "metadata_empty_value",
            Self::SurveyTitleEmpty => "survey_title_empty",
            Self::SurveyWithoutGraph => "survey_without_graph",
            Self::SurveyClosed => "survey_closed",
            Self::CampaignNotStarted => "campaign_not_started",
            Self::CampaignEnded => "campaign_ended",
            Self::SurveyInconsistentState => "survey_inconsistent_state",
            Self::ResumeTokenEmpty => "resume_token_empty",
            Self::RequiredQuestionUnanswered(_) => "required_question_unanswered",
            Self::AnswerNotFinite(_) => "answer_not_finite",
            Self::ResponseInconsistentState => "response_inconsistent_state",
            Self::SurveyClosedBeforeOpened => "survey_closed_before_opened",
            Self::AnswerUnknownQuestion(_) => "answer_unknown_question",
            Self::AnswerKindMismatch(_) => "answer_kind_mismatch",
            Self::AnswerUnknownOption(_) => "answer_unknown_option",
            Self::AnswerDuplicateOption(_) => "answer_duplicate_option",
            Self::AnswerTooManyPhotos(_) => "answer_too_many_photos",
            Self::AnswerEmptyPhoto(_) => "answer_empty_photo",
            Self::AnswerForeignPhoto(_) => "answer_foreign_photo",
            Self::PublicSlugInvalid => "public_slug_invalid",
            Self::SurveyPublicSlugRequired => "survey_public_slug_required",
            Self::SurveyPublicSlugForbidden => "survey_public_slug_forbidden",
            Self::SurveyCampaignMismatch => "survey_campaign_mismatch",
            Self::SurveyCampaignOrganizationMismatch => "survey_campaign_organization_mismatch",
            Self::CampaignNameEmpty => "campaign_name_empty",
            Self::ResponseSurveyMismatch => "response_survey_mismatch",
            Self::ResponseMemberRequired => "response_member_required",
            Self::ResponseMemberForbidden => "response_member_forbidden",
            Self::ResponseResumeTokenRequired => "response_resume_token_required",
            Self::ResponseResumeTokenForbidden => "response_resume_token_forbidden",
            Self::SurveyNotFound => "survey_not_found",
            Self::CampaignNotFound => "campaign_not_found",
            Self::ResponseNotFound => "response_not_found",
            Self::SurveyNotDraft => "survey_not_draft",
            Self::SurveyNotOpened => "survey_not_opened",
            Self::SurveyHasResponses => "survey_has_responses",
            Self::CampaignHasSurveys => "campaign_has_surveys",
            Self::ResponseAlreadyCompleted => "response_already_completed",
            Self::ResponseCapReached => "response_cap_reached",
            Self::RespondentAlreadyAnswered => "respondent_already_answered",
            Self::SurveyStateChanged => "survey_state_changed",
            Self::ResponsePhotoCapReached => "response_photo_cap_reached",
            Self::SurveyVisibilityFrozen => "survey_visibility_frozen",
            Self::GraphVersionInvalid => "graph_version_invalid",
            Self::GraphVersionNotFound => "graph_version_not_found",
            Self::GraphVersionExhausted => "graph_version_exhausted",
            Self::ResponseGraphVersionMismatch => "response_graph_version_mismatch",
        }
    }

    pub fn question_id(&self) -> Option<QuestionId> {
        match self {
            Self::GraphDuplicateNode(id)
            | Self::GraphUnknownEdgeNode(id)
            | Self::GraphInvalidBranch(id)
            | Self::GraphDuplicateExit(id)
            | Self::GraphDeadEnd(id)
            | Self::GraphCycle(id)
            | Self::GraphUnreachableNode(id)
            | Self::GraphSecondIdentifier(id)
            | Self::GraphIdentifierOnInvalidKind(id)
            | Self::GraphMaxPhotosOnInvalidKind(id)
            | Self::GraphZeroMaxPhotos(id)
            | Self::GraphChoiceWithoutOptions(id)
            | Self::GraphQuestionLabelEmpty(id)
            | Self::GraphOptionEmpty(id)
            | Self::GraphDuplicateOption(id)
            | Self::ConditionUnknownQuestion(id)
            | Self::ConditionOnNonChoice(id)
            | Self::ConditionUnknownOption(id)
            | Self::ConditionQuestionNotAsked(id)
            | Self::GraphBlockTooSmall(id)
            | Self::GraphBlockNotContiguous(id)
            | Self::GraphBlockMemberHasCondition(id)
            | Self::RequiredQuestionUnanswered(id)
            | Self::AnswerNotFinite(id)
            | Self::AnswerUnknownQuestion(id)
            | Self::AnswerKindMismatch(id)
            | Self::AnswerUnknownOption(id)
            | Self::AnswerDuplicateOption(id)
            | Self::AnswerTooManyPhotos(id)
            | Self::AnswerEmptyPhoto(id)
            | Self::AnswerForeignPhoto(id) => Some(*id),
            Self::EmailInvalid
            | Self::PhoneInvalid
            | Self::GraphEmpty
            | Self::GraphWithoutEnd
            | Self::GraphUnknownStart
            | Self::GraphStartNotQuestion
            | Self::WindowInverted
            | Self::MetadataEmptyKey
            | Self::MetadataEmptyValue
            | Self::SurveyTitleEmpty
            | Self::SurveyWithoutGraph
            | Self::SurveyClosed
            | Self::CampaignNotStarted
            | Self::CampaignEnded
            | Self::SurveyInconsistentState
            | Self::ResumeTokenEmpty
            | Self::ResponseInconsistentState
            | Self::SurveyClosedBeforeOpened
            | Self::PublicSlugInvalid
            | Self::SurveyPublicSlugRequired
            | Self::SurveyPublicSlugForbidden
            | Self::SurveyCampaignMismatch
            | Self::SurveyCampaignOrganizationMismatch
            | Self::CampaignNameEmpty
            | Self::ResponseSurveyMismatch
            | Self::ResponseMemberRequired
            | Self::ResponseMemberForbidden
            | Self::ResponseResumeTokenRequired
            | Self::ResponseResumeTokenForbidden
            | Self::SurveyNotFound
            | Self::CampaignNotFound
            | Self::ResponseNotFound
            | Self::SurveyNotDraft
            | Self::SurveyNotOpened
            | Self::SurveyHasResponses
            | Self::CampaignHasSurveys
            | Self::ResponseAlreadyCompleted
            | Self::ResponseCapReached
            | Self::RespondentAlreadyAnswered
            | Self::SurveyStateChanged
            | Self::ResponsePhotoCapReached
            | Self::SurveyVisibilityFrozen
            | Self::GraphVersionInvalid
            | Self::GraphVersionNotFound
            | Self::GraphVersionExhausted
            | Self::ResponseGraphVersionMismatch => None,
        }
    }

    fn refused(self, kind: RefusalKind) -> CoreError {
        CoreError::Refused(Refusal {
            kind,
            reason: self.code(),
            question_id: self.question_id().map(|id| id.0),
            message: self.to_string(),
        })
    }
}

impl From<SurveyError> for CoreError {
    fn from(error: SurveyError) -> Self {
        match error {
            SurveyError::SurveyNotFound
            | SurveyError::CampaignNotFound
            | SurveyError::ResponseNotFound
            | SurveyError::GraphVersionNotFound => Self::NotFound,
            SurveyError::EmailInvalid
            | SurveyError::PhoneInvalid
            | SurveyError::GraphEmpty
            | SurveyError::GraphWithoutEnd
            | SurveyError::GraphDuplicateNode(_)
            | SurveyError::GraphUnknownStart
            | SurveyError::GraphStartNotQuestion
            | SurveyError::GraphUnknownEdgeNode(_)
            | SurveyError::GraphInvalidBranch(_)
            | SurveyError::GraphDuplicateExit(_)
            | SurveyError::GraphDeadEnd(_)
            | SurveyError::GraphCycle(_)
            | SurveyError::GraphUnreachableNode(_)
            | SurveyError::GraphSecondIdentifier(_)
            | SurveyError::GraphIdentifierOnInvalidKind(_)
            | SurveyError::GraphMaxPhotosOnInvalidKind(_)
            | SurveyError::GraphZeroMaxPhotos(_)
            | SurveyError::GraphChoiceWithoutOptions(_)
            | SurveyError::GraphQuestionLabelEmpty(_)
            | SurveyError::GraphOptionEmpty(_)
            | SurveyError::GraphDuplicateOption(_)
            | SurveyError::ConditionUnknownQuestion(_)
            | SurveyError::ConditionOnNonChoice(_)
            | SurveyError::ConditionUnknownOption(_)
            | SurveyError::ConditionQuestionNotAsked(_)
            | SurveyError::GraphBlockTooSmall(_)
            | SurveyError::GraphBlockNotContiguous(_)
            | SurveyError::GraphBlockMemberHasCondition(_)
            | SurveyError::WindowInverted
            | SurveyError::MetadataEmptyKey
            | SurveyError::MetadataEmptyValue
            | SurveyError::SurveyTitleEmpty
            | SurveyError::CampaignNameEmpty
            | SurveyError::PublicSlugInvalid
            | SurveyError::ResumeTokenEmpty
            | SurveyError::RequiredQuestionUnanswered(_)
            | SurveyError::AnswerNotFinite(_)
            | SurveyError::AnswerUnknownQuestion(_)
            | SurveyError::AnswerKindMismatch(_)
            | SurveyError::AnswerUnknownOption(_)
            | SurveyError::AnswerDuplicateOption(_)
            | SurveyError::AnswerTooManyPhotos(_)
            | SurveyError::AnswerEmptyPhoto(_)
            | SurveyError::AnswerForeignPhoto(_) => error.refused(RefusalKind::Validation),
            SurveyError::ResponseMemberRequired
            | SurveyError::ResponseMemberForbidden
            | SurveyError::ResponseResumeTokenRequired
            | SurveyError::ResponseResumeTokenForbidden
            | SurveyError::SurveyInconsistentState
            | SurveyError::ResponseInconsistentState
            | SurveyError::GraphVersionInvalid
            | SurveyError::ResponseGraphVersionMismatch => Self::Internal(error.to_string()),
            SurveyError::SurveyWithoutGraph
            | SurveyError::SurveyClosed
            | SurveyError::CampaignNotStarted
            | SurveyError::CampaignEnded
            | SurveyError::SurveyClosedBeforeOpened
            | SurveyError::SurveyPublicSlugRequired
            | SurveyError::SurveyPublicSlugForbidden
            | SurveyError::SurveyCampaignMismatch
            | SurveyError::SurveyCampaignOrganizationMismatch
            | SurveyError::ResponseSurveyMismatch
            | SurveyError::SurveyNotDraft
            | SurveyError::SurveyNotOpened
            | SurveyError::SurveyHasResponses
            | SurveyError::CampaignHasSurveys
            | SurveyError::ResponseAlreadyCompleted
            | SurveyError::ResponseCapReached
            | SurveyError::RespondentAlreadyAnswered
            | SurveyError::SurveyStateChanged
            | SurveyError::ResponsePhotoCapReached
            | SurveyError::SurveyVisibilityFrozen
            | SurveyError::GraphVersionExhausted => error.refused(RefusalKind::Conflict),
        }
    }
}

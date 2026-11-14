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
    #[error("graph: condition {0} refers to an unknown question")]
    ConditionUnknownQuestion(QuestionId),
    #[error("graph: condition {0} refers to a node that is not a choice question")]
    ConditionOnNonChoice(QuestionId),
    #[error("graph: condition {0} refers to an unknown option")]
    ConditionUnknownOption(QuestionId),
    #[error("graph: condition {0} refers to a question not asked on every path to it")]
    ConditionQuestionNotAsked(QuestionId),
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
}

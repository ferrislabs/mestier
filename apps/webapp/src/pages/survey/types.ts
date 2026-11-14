import type { Endpoints, Schemas } from '#/api/api.client'

export type Survey = Endpoints.get_GetSurvey['responses'][200]['data']
export type Campaign = Endpoints.get_GetSurveyCampaign['responses'][200]['data']
export type SurveyResponse =
	Endpoints.get_GetSurveyResponse['responses'][200]['data']

export type SurveyStatus = Schemas.SurveyStatus
export type CampaignStatus = Schemas.CampaignStatus
export type ResponseStatus = Schemas.ResponseStatus
export type Visibility = Schemas.Visibility
export type QuestionKind = Schemas.QuestionKind
export type Branch = Schemas.SurveyBranchSchema
export type SurveyAnswer = Schemas.SurveyAnswerSchema

export type QuestionNode = Schemas.SurveyQuestionNodeSchema & {
	type: 'QUESTION'
}
export type ConditionNode = Schemas.SurveyConditionNodeSchema & {
	type: 'CONDITION'
}
export type EndNode = Schemas.SurveyEndNodeSchema & { type: 'END' }
export type GraphNode = QuestionNode | ConditionNode | EndNode
export type GraphEdge = Schemas.SurveyEdgeSchema

export interface EditorGraph {
	start: string | null
	nodes: GraphNode[]
	edges: GraphEdge[]
}

export type NodeChoice = QuestionKind | 'CONDITION' | 'END'

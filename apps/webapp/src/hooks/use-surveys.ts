import {
	useMutation,
	useQueries,
	useQuery,
	useQueryClient,
} from '@tanstack/react-query'

const SURVEYS_PATH = '/api/v1/organizations/{organization_id}/surveys'
const SURVEY_PATH =
	'/api/v1/organizations/{organization_id}/surveys/{survey_id}'
const SURVEY_OPEN_PATH =
	'/api/v1/organizations/{organization_id}/surveys/{survey_id}/open'
const SURVEY_CLOSE_PATH =
	'/api/v1/organizations/{organization_id}/surveys/{survey_id}/close'
const SURVEY_DUPLICATE_PATH =
	'/api/v1/organizations/{organization_id}/surveys/{survey_id}/duplicate'
const SURVEY_RESPONSES_PATH =
	'/api/v1/organizations/{organization_id}/surveys/{survey_id}/responses'
const SURVEY_RESPONSE_PATH =
	'/api/v1/organizations/{organization_id}/survey-responses/{response_id}'
const CAMPAIGNS_PATH =
	'/api/v1/organizations/{organization_id}/survey-campaigns'
const CAMPAIGN_PATH =
	'/api/v1/organizations/{organization_id}/survey-campaigns/{campaign_id}'
const CAMPAIGN_SURVEYS_PATH =
	'/api/v1/organizations/{organization_id}/survey-campaigns/{campaign_id}/surveys'
const SURVEY_GRAPH_VERSION_PATH =
	'/api/v1/organizations/{organization_id}/surveys/{survey_id}/graph-versions/{version}'

const SURVEY_KEYS = new Set<string>([
	SURVEYS_PATH,
	SURVEY_PATH,
	SURVEY_RESPONSES_PATH,
	SURVEY_RESPONSE_PATH,
	CAMPAIGNS_PATH,
	CAMPAIGN_PATH,
	CAMPAIGN_SURVEYS_PATH,
])

export const SURVEY_LIST_PAGE_SIZE = 100
export const RESPONSES_PAGE_SIZE = 25
export const CAMPAIGN_SURVEYS_PAGE_SIZE = 200

interface PagedResponse<T> {
	data: T[]
	pagination?: {
		next_page?: number | null
		total?: number | null
	} | null
}

export interface PagedList<T> {
	items: T[]
	total: number | null
	hasMore: boolean
	isLoading: boolean
	isFetchingMore: boolean
	error: unknown
}

function mergePages<T>(
	results: readonly {
		data?: PagedResponse<T>
		isLoading: boolean
		error: unknown
	}[],
): PagedList<T> {
	const last = results[results.length - 1]
	const pagination = last?.data?.pagination
	return {
		items: results.flatMap((result) => result.data?.data ?? []),
		total: pagination?.total ?? null,
		hasMore: pagination?.next_page != null,
		isLoading: results[0]?.isLoading ?? false,
		isFetchingMore: results.length > 1 && (last?.isLoading ?? false),
		error: results.find((result) => result.error)?.error ?? null,
	}
}

function pageNumbers(pageCount: number): number[] {
	return Array.from({ length: Math.max(pageCount, 1) }, (_, index) => index + 1)
}

function useSurveyInvalidation() {
	const queryClient = useQueryClient()
	return () =>
		queryClient.invalidateQueries({
			predicate: (query) => {
				const meta = query.queryKey[0]
				const id =
					typeof meta === 'object' && meta !== null
						? (meta as { _id?: unknown })._id
						: undefined
				return typeof id === 'string' && SURVEY_KEYS.has(id)
			},
		})
}

export function useSurveys(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(SURVEYS_PATH, {
			path: { organization_id: organizationId },
			query: { per_page: SURVEY_LIST_PAGE_SIZE },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

export function useSurvey(organizationId: string, surveyId: string) {
	return useQuery({
		...window.tanstackApi.get(SURVEY_PATH, {
			path: { organization_id: organizationId, survey_id: surveyId },
		}).queryOptions,
		enabled: Boolean(organizationId) && Boolean(surveyId),
	})
}

export function useCampaigns(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(CAMPAIGNS_PATH, {
			path: { organization_id: organizationId },
			query: { per_page: SURVEY_LIST_PAGE_SIZE },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

export function useCampaign(organizationId: string, campaignId: string) {
	return useQuery({
		...window.tanstackApi.get(CAMPAIGN_PATH, {
			path: { organization_id: organizationId, campaign_id: campaignId },
		}).queryOptions,
		enabled: Boolean(organizationId) && Boolean(campaignId),
	})
}

export function useCampaignSurveys(
	organizationId: string,
	campaignId: string,
	pageCount: number,
) {
	return useQueries({
		queries: pageNumbers(pageCount).map((page) => ({
			...window.tanstackApi.get(CAMPAIGN_SURVEYS_PATH, {
				path: { organization_id: organizationId, campaign_id: campaignId },
				query: { page, per_page: CAMPAIGN_SURVEYS_PAGE_SIZE },
			}).queryOptions,
			enabled: Boolean(organizationId) && Boolean(campaignId),
		})),
		combine: mergePages,
	})
}

export function useCampaignSurveyList(
	organizationId: string,
	campaignId: string | null,
	pageCount: number,
) {
	return useQueries({
		queries: pageNumbers(pageCount).map((page) => ({
			...window.tanstackApi.get(SURVEYS_PATH, {
				path: { organization_id: organizationId },
				query: {
					page,
					per_page: SURVEY_LIST_PAGE_SIZE,
					campaign_id: campaignId ?? '',
				},
			}).queryOptions,
			enabled: Boolean(organizationId) && Boolean(campaignId),
		})),
		combine: mergePages,
	})
}

export function useSurveyGraphVersion(
	organizationId: string,
	surveyId: string,
	version: number | null,
) {
	return useQuery({
		...window.tanstackApi.get(SURVEY_GRAPH_VERSION_PATH, {
			path: {
				organization_id: organizationId,
				survey_id: surveyId,
				version: version ?? 0,
			},
		}).queryOptions,
		enabled: Boolean(organizationId) && Boolean(surveyId) && version !== null,
		staleTime: Number.POSITIVE_INFINITY,
	})
}

export function useSurveyResponses(
	organizationId: string,
	surveyId: string,
	page: number,
) {
	return useQuery({
		...window.tanstackApi.get(SURVEY_RESPONSES_PATH, {
			path: { organization_id: organizationId, survey_id: surveyId },
			query: { page, per_page: RESPONSES_PAGE_SIZE },
		}).queryOptions,
		enabled: Boolean(organizationId) && Boolean(surveyId),
	})
}

export function useCreateSurvey() {
	const invalidate = useSurveyInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('post', SURVEYS_PATH).mutationOptions,
		onSuccess: invalidate,
	})
}

export function useUpdateSurvey() {
	const invalidate = useSurveyInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('patch', SURVEY_PATH).mutationOptions,
		onSuccess: invalidate,
	})
}

export function useDeleteSurvey() {
	const invalidate = useSurveyInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('delete', SURVEY_PATH).mutationOptions,
		onSuccess: invalidate,
	})
}

export function useOpenSurvey() {
	const invalidate = useSurveyInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('post', SURVEY_OPEN_PATH).mutationOptions,
		onSuccess: invalidate,
	})
}

export function useCloseSurvey() {
	const invalidate = useSurveyInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('post', SURVEY_CLOSE_PATH).mutationOptions,
		onSuccess: invalidate,
	})
}

export function useDuplicateSurvey() {
	const invalidate = useSurveyInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('post', SURVEY_DUPLICATE_PATH)
			.mutationOptions,
		onSuccess: invalidate,
	})
}

export function useDeleteSurveyResponse() {
	const invalidate = useSurveyInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('delete', SURVEY_RESPONSE_PATH)
			.mutationOptions,
		onSuccess: invalidate,
	})
}

export function useCreateCampaign() {
	const invalidate = useSurveyInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('post', CAMPAIGNS_PATH).mutationOptions,
		onSuccess: invalidate,
	})
}

export function useUpdateCampaign() {
	const invalidate = useSurveyInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('patch', CAMPAIGN_PATH).mutationOptions,
		onSuccess: invalidate,
	})
}

export function useDeleteCampaign() {
	const invalidate = useSurveyInvalidation()
	return useMutation({
		...window.tanstackApi.mutation('delete', CAMPAIGN_PATH).mutationOptions,
		onSuccess: invalidate,
	})
}

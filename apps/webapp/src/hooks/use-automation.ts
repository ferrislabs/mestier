import {
	useMutation,
	useQueries,
	useQuery,
	useQueryClient,
} from '@tanstack/react-query'
import { useCallback } from 'react'
import type { Schemas } from '#/api/api.client'
import { isCollectionMissing } from '#/pages/automation/lib/collection-format'
import type { CollectionRead } from '#/pages/automation/lib/collection-node'
import {
	type Collection,
	type CollectionDraft,
	fromCollectionResponse,
	fromCollectionSummary,
	toCollectionRequest,
} from '#/pages/automation/lib/collections'
import { isTerminalRunStatus } from '#/pages/automation/lib/workflow-runs'

const CONNECTORS_PATH =
	'/api/v1/organizations/{organization_id}/automation/connectors'
const EVENTS_PATH = '/api/v1/organizations/{organization_id}/automation/events'
const CREDENTIALS_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials'
const CREDENTIAL_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}'
const CREDENTIAL_ROTATE_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/rotate'
const SETTINGS_PATH =
	'/api/v1/organizations/{organization_id}/automation/settings'
const FOLDERS_PATH =
	'/api/v1/organizations/{organization_id}/automation/folders'
const WORKFLOWS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows'
const WORKFLOW_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}'
const WORKFLOW_VERSIONS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/versions'
const RUNS_PATH = '/api/v1/organizations/{organization_id}/automation/runs'
const RUN_PATH =
	'/api/v1/organizations/{organization_id}/automation/runs/{run_id}'
const RUN_REPLAY_PATH =
	'/api/v1/organizations/{organization_id}/automation/runs/{run_id}/replay'
const WORKFLOW_RUNS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/runs'
const COLLECTIONS_PATH =
	'/api/v1/organizations/{organization_id}/automation/collections'
const COLLECTION_PATH =
	'/api/v1/organizations/{organization_id}/automation/collections/{collection_id}'
const RUN_POLL_INTERVAL_MS = 2000
const QUERY_RETRIES = 3
const EXPRESSIONS_EVALUATE_PATH =
	'/api/v1/organizations/{organization_id}/automation/expressions/evaluate'

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function invalidate(
	queryClient: ReturnType<typeof useQueryClient>,
	path: string,
) {
	return queryClient.invalidateQueries({
		predicate: (query) => queryKeyMeta(query.queryKey)?._id === path,
	})
}

/** Auth schemes and connector descriptors share one endpoint on the backend
 * (the connector catalogue) — this screen only ever needs
 * `data.auth_schemes`, to populate the credential kind picker. */
export function useConnectorCatalogue(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(CONNECTORS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
	)
}

export function useAutomationEvents(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(EVENTS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
	)
}

export function useAutomationCredentials(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(CREDENTIALS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
	)
}

export function useCreateCredential(organizationId: string) {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', CREDENTIALS_PATH).mutationOptions,
		onSuccess: () => invalidate(queryClient, CREDENTIALS_PATH),
		meta: { organizationId },
	})
}

export function useUpdateCredential() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('patch', CREDENTIAL_PATH).mutationOptions,
		onSuccess: () => invalidate(queryClient, CREDENTIALS_PATH),
	})
}

export function useDeleteCredential() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('delete', CREDENTIAL_PATH).mutationOptions,
		onSuccess: () => invalidate(queryClient, CREDENTIALS_PATH),
	})
}

/** Returns the freshly generated secret alongside the credential — see
 * `RotatedCredential`'s doc comment: shown once, same as at creation. */
export function useRotateCredential() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', CREDENTIAL_ROTATE_PATH)
			.mutationOptions,
		onSuccess: () => invalidate(queryClient, CREDENTIALS_PATH),
	})
}

export function useAutomationSettings(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(SETTINGS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
	)
}

export function useUpdateAutomationSettings() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('put', SETTINGS_PATH).mutationOptions,
		onSuccess: () => invalidate(queryClient, SETTINGS_PATH),
	})
}

/** Unpaginated — the backend hands back every run for the organization,
 * most recent first (see `libs/handlers-automation/src/run/list.rs`). */
export function useAutomationRuns(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(RUNS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
	)
}

export function useReplayRun() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', RUN_REPLAY_PATH).mutationOptions,
		onSuccess: () => invalidate(queryClient, RUNS_PATH),
	})
}

export function useAutomationFolders(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(FOLDERS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
	)
}

export function useAutomationWorkflows(
	organizationId: string,
	archived = false,
) {
	return useQuery(
		window.tanstackApi.get(WORKFLOWS_PATH, {
			path: { organization_id: organizationId },
			query: archived ? { archived: true } : {},
		}).queryOptions,
	)
}

export function useCreateWorkflow(organizationId: string) {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', WORKFLOWS_PATH).mutationOptions,
		onSuccess: () => invalidate(queryClient, WORKFLOWS_PATH),
		meta: { organizationId },
	})
}

export function useUpdateWorkflow() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('patch', WORKFLOW_PATH).mutationOptions,
		onSuccess: () =>
			Promise.all([
				invalidate(queryClient, WORKFLOWS_PATH),
				invalidate(queryClient, WORKFLOW_PATH),
			]),
	})
}

export function useDeleteWorkflow() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('delete', WORKFLOW_PATH).mutationOptions,
		onSuccess: () => invalidate(queryClient, WORKFLOWS_PATH),
	})
}

export function useWorkflow(organizationId: string, workflowId: string | null) {
	return useQuery({
		...window.tanstackApi.get(WORKFLOW_PATH, {
			path: { organization_id: organizationId, workflow_id: workflowId ?? '' },
		}).queryOptions,
		enabled: workflowId !== null,
	})
}

export function useSaveWorkflowVersion() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('put', WORKFLOW_VERSIONS_PATH)
			.mutationOptions,
		onSuccess: () =>
			Promise.all([
				invalidate(queryClient, WORKFLOWS_PATH),
				invalidate(queryClient, WORKFLOW_PATH),
			]),
	})
}

/** A run's steps — fetched lazily, only once its detail sheet opens. */
export function useRun(organizationId: string, runId: string | null) {
	return useQuery({
		...window.tanstackApi.get(RUN_PATH, {
			path: { organization_id: organizationId, run_id: runId ?? '' },
		}).queryOptions,
		enabled: runId !== null,
	})
}

export function useStartRun() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', WORKFLOW_RUNS_PATH).mutationOptions,
		onSuccess: () => invalidate(queryClient, RUNS_PATH),
	})
}

function graphsByVersion(
	results: readonly { data?: { data: Schemas.RunDetailResponse } }[],
): ReadonlyMap<string, Schemas.GraphDto | null> {
	const graphs = new Map<string, Schemas.GraphDto | null>()
	for (const result of results) {
		const detail = result.data?.data
		if (detail) graphs.set(detail.workflow_version_id, detail.graph ?? null)
	}
	return graphs
}

export function useRunGraphs(
	organizationId: string,
	runIds: readonly string[],
) {
	return useQueries({
		queries: runIds.map((runId) => ({
			...window.tanstackApi.get(RUN_PATH, {
				path: { organization_id: organizationId, run_id: runId },
			}).queryOptions,
			staleTime: Number.POSITIVE_INFINITY,
		})),
		combine: graphsByVersion,
	})
}

export function useRunPolling(organizationId: string, runId: string | null) {
	return useQuery({
		...window.tanstackApi.get(RUN_PATH, {
			path: { organization_id: organizationId, run_id: runId ?? '' },
		}).queryOptions,
		enabled: runId !== null,
		refetchInterval: (query) => {
			const status = query.state.data?.data.status
			return status && !isTerminalRunStatus(status)
				? RUN_POLL_INTERVAL_MS
				: false
		},
	})
}

export function useEvaluateExpression() {
	return useMutation({
		...window.tanstackApi.mutation('post', EXPRESSIONS_EVALUATE_PATH)
			.mutationOptions,
	})
}

function summariesOf(response: {
	data: readonly Schemas.CollectionSummaryResponse[]
}) {
	return response.data.map(fromCollectionSummary)
}

function collectionOf(response: { data: Schemas.CollectionResponse }) {
	return fromCollectionResponse(response.data)
}

interface CollectionResult {
	data?: Collection
	error: unknown
	isError: boolean
}

function readOf(result: CollectionResult): CollectionRead {
	if (result.data) return { status: 'found', collection: result.data }
	if (!result.isError) return { status: 'loading' }
	return isCollectionMissing(result.error)
		? { status: 'notFound' }
		: { status: 'error' }
}

function collectionsOf(
	ids: readonly string[],
	results: readonly CollectionResult[],
) {
	const reads = new Map<string, CollectionRead>(
		ids.map((id, index) => [
			id,
			results[index] ? readOf(results[index]) : { status: 'loading' },
		]),
	)
	return {
		collections: [...reads.values()].flatMap((read) =>
			read.status === 'found' ? [read.collection] : [],
		),
		reads,
		isLoading: [...reads.values()].some((read) => read.status === 'loading'),
	}
}

function retryUnlessMissing(failureCount: number, error: unknown): boolean {
	return !isCollectionMissing(error) && failureCount < QUERY_RETRIES
}

export function useCollections(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(COLLECTIONS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		select: summariesOf,
	})
}

export function useCollection(
	organizationId: string,
	collectionId: string,
	{ enabled = true }: { enabled?: boolean } = {},
) {
	return useQuery({
		...window.tanstackApi.get(COLLECTION_PATH, {
			path: { organization_id: organizationId, collection_id: collectionId },
		}).queryOptions,
		select: collectionOf,
		retry: retryUnlessMissing,
		enabled,
	})
}

export function useCollectionDetails(
	organizationId: string,
	collectionIds: readonly string[],
) {
	const idsKey = collectionIds.join(',')
	const combine = useCallback(
		(results: readonly CollectionResult[]) =>
			collectionsOf(idsKey === '' ? [] : idsKey.split(','), results),
		[idsKey],
	)
	return useQueries({
		queries: collectionIds.map((collectionId) => ({
			...window.tanstackApi.get(COLLECTION_PATH, {
				path: { organization_id: organizationId, collection_id: collectionId },
			}).queryOptions,
			select: collectionOf,
			retry: retryUnlessMissing,
		})),
		combine,
	})
}

export function useImportCollection(organizationId: string) {
	const queryClient = useQueryClient()
	const { mutationOptions } = window.tanstackApi.mutation(
		'post',
		COLLECTIONS_PATH,
	)
	return useMutation({
		mutationKey: mutationOptions.mutationKey,
		mutationFn: async (draft: CollectionDraft) =>
			collectionOf(
				await mutationOptions.mutationFn({
					path: { organization_id: organizationId },
					body: toCollectionRequest(draft),
				}),
			),
		onSuccess: () => invalidate(queryClient, COLLECTIONS_PATH),
		meta: { organizationId },
	})
}

export function useReplaceCollection(organizationId: string) {
	const queryClient = useQueryClient()
	const { mutationOptions } = window.tanstackApi.mutation(
		'put',
		COLLECTION_PATH,
	)
	return useMutation({
		mutationKey: mutationOptions.mutationKey,
		mutationFn: async (input: {
			collectionId: string
			draft: CollectionDraft
		}) =>
			collectionOf(
				await mutationOptions.mutationFn({
					path: {
						organization_id: organizationId,
						collection_id: input.collectionId,
					},
					body: toCollectionRequest(input.draft),
				}),
			),
		onSuccess: () =>
			Promise.all([
				invalidate(queryClient, COLLECTIONS_PATH),
				invalidate(queryClient, COLLECTION_PATH),
			]),
	})
}

export function useDeleteCollection(organizationId: string) {
	const queryClient = useQueryClient()
	const { mutationOptions } = window.tanstackApi.mutation(
		'delete',
		COLLECTION_PATH,
	)
	return useMutation({
		mutationKey: mutationOptions.mutationKey,
		mutationFn: async (collectionId: string) => {
			await mutationOptions.mutationFn({
				path: {
					organization_id: organizationId,
					collection_id: collectionId,
				},
			})
		},
		onSuccess: () =>
			Promise.all([
				invalidate(queryClient, COLLECTIONS_PATH),
				invalidate(queryClient, COLLECTION_PATH),
			]),
	})
}

export type AuthScheme = Schemas.AuthSchemeResponse
export type AuthField = Schemas.FieldResponse
export type EventDescriptor = Schemas.EventDescriptorResponse
export type Credential = Schemas.CredentialResponse
/** Only present in the create/rotate response — never on list/update. */
export type CreatedCredential = Credential & { secret: unknown }
export type AutomationSettings = Schemas.AutomationSettingsBody
export type Run = Schemas.RunResponse
export type Workflow = Schemas.WorkflowResponse
export type AutomationFolder = Schemas.FolderResponse
export type WorkflowDetail = Schemas.WorkflowDetailResponse

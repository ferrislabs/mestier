import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

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
const WORKFLOWS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows'
const RUNS_PATH = '/api/v1/organizations/{organization_id}/automation/runs'
const RUN_PATH =
	'/api/v1/organizations/{organization_id}/automation/runs/{run_id}'
const RUN_REPLAY_PATH =
	'/api/v1/organizations/{organization_id}/automation/runs/{run_id}/replay'

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

/** Fetched only to resolve `workflow_id` into a name for the run log — no
 * write path needed here, that is the workflow editor's job (#204). */
export function useAutomationWorkflows(organizationId: string) {
	return useQuery(
		window.tanstackApi.get(WORKFLOWS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
	)
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

export type AuthScheme = Schemas.AuthSchemeResponse
export type AuthField = Schemas.FieldResponse
export type EventDescriptor = Schemas.EventDescriptorResponse
export type Credential = Schemas.CredentialResponse
/** Only present in the create/rotate response — never on list/update. */
export type CreatedCredential = Credential & { secret: unknown }
export type AutomationSettings = Schemas.AutomationSettingsBody
export type Run = Schemas.RunResponse

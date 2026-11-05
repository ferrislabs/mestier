import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen } from '@testing-library/react'
import type { UserEvent } from '@testing-library/user-event'
import type { ReactNode } from 'react'
import type { Schemas } from '#/api/api.client'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { PERMISSION_CATALOG } from '#/lib/permission-catalog'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

export const TICKETS_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets'
export const TICKET_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}'
export const ENTRIES_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/entries'
export const STATUS_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/status'
export const ASSIGNEE_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/assignee'
export const BADGES_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/badges'
export const BADGE_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/badges/{label}'
export const SETTINGS_PATH =
	'/api/v1/organizations/{organization_id}/support/settings'
export const ENTRY_PATH =
	'/api/v1/organizations/{organization_id}/support/entries/{entry_id}'
export const ENTRY_SEND_PATH =
	'/api/v1/organizations/{organization_id}/support/entries/{entry_id}/send'
export const ATTACHMENT_URL_PATH =
	'/api/v1/organizations/{organization_id}/support/attachments/{attachment_id}/url'
export const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
export const ALL_PERMISSIONS = PERMISSION_CATALOG.map((entry) => entry.name)

export const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	slug: 'atelier-bois',
	field_clock_enabled: false,
	vat_on_debits: false,
	mcp_enabled: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

export const MEMBERS = [
	{
		id: 'member-1',
		organization_id: 'org-1',
		display_name: 'Alix Martin',
		last_name: 'Martin',
		first_name: 'Alix',
		created_at: '2026-01-01T00:00:00Z',
	},
	{
		id: 'member-2',
		organization_id: 'org-1',
		display_name: 'Jo Durand',
		last_name: 'Durand',
		first_name: 'Jo',
		created_at: '2026-01-01T00:00:00Z',
	},
]

export function ticket(
	overrides: Partial<Schemas.TicketResponse> = {},
): Schemas.TicketResponse {
	return {
		id: 'ticket-1',
		number: 1,
		subject: 'Fuite sous évier',
		body: 'Bonjour',
		status: 'new',
		assignee_member_id: null,
		assigned_at: null,
		custom_fields: {},
		badges: [],
		external_id: null,
		created_at: '2026-10-01T07:00:00Z',
		updated_at: '2026-10-01T07:00:00Z',
		...overrides,
	}
}

export function entry(
	overrides: Partial<Schemas.TicketEntryResponse> = {},
): Schemas.TicketEntryResponse {
	return {
		id: 'entry-1',
		ticket_id: 'ticket-1',
		kind: 'reply',
		body: 'Réponse',
		author_member_id: 'member-1',
		attachments: [],
		created_at: '2026-11-01T09:00:00Z',
		updated_at: '2026-11-01T09:00:00Z',
		...overrides,
	}
}

export function page(tickets: Schemas.TicketResponse[], total?: number) {
	return {
		data: tickets,
		pagination: {
			current_page: 1,
			first_page: 1,
			is_empty: tickets.length === 0,
			per_page: 100,
			total: total ?? tickets.length,
		},
	}
}

type Handler = (params: unknown) => unknown

export interface RecordedCall {
	method: string
	path: string
	params: unknown
}

export function installFakeTanstackApi() {
	const calls: RecordedCall[] = []
	const getHandlers = new Map<string, Handler>()
	const mutationHandlers = new Map<string, Handler>()

	const fakeApi = {
		get(path: string, params: unknown) {
			const request = (params ?? {}) as { path?: unknown; query?: unknown }
			const queryKey = [{ _id: path, path: request.path, query: request.query }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls.push({ method: 'get', path, params })
						const handler = getHandlers.get(path)
						if (!handler) throw new Error(`unmocked GET ${path}`)
						return handler(params)
					},
				},
			}
		},
		mutation(method: string, path: string) {
			return {
				mutationOptions: {
					mutationKey: [{ method, path }],
					mutationFn: async (params: unknown) => {
						calls.push({ method, path, params })
						const handler = mutationHandlers.get(`${method}:${path}`)
						if (!handler) {
							throw new Error(`unmocked ${method.toUpperCase()} ${path}`)
						}
						return handler(params)
					},
				},
			}
		},
	}

	Object.assign(window, { tanstackApi: fakeApi })

	return {
		calls,
		mockGet(path: string, handler: Handler) {
			getHandlers.set(path, handler)
		},
		mockMutation(method: string, path: string, handler: Handler) {
			mutationHandlers.set(`${method}:${path}`, handler)
		},
	}
}

export type FakeApi = ReturnType<typeof installFakeTanstackApi>

export function callsTo(api: FakeApi, method: string, path: string) {
	return api.calls.filter(
		(call) => call.method === method && call.path === path,
	)
}

export function createQueryClient(permissions: string[]): QueryClient {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	seedPermissionsCacheForOrganization(queryClient, ORGANIZATION.id, permissions)
	return queryClient
}

export function Providers({
	queryClient,
	children,
}: {
	queryClient: QueryClient
	children: ReactNode
}) {
	return (
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
					{children}
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>
	)
}

export interface Deferred<T> {
	promise: Promise<T>
	resolve: (value: T) => void
	reject: (reason: unknown) => void
}

export function deferred<T>(): Deferred<T> {
	let resolve: (value: T) => void = () => {}
	let reject: (reason: unknown) => void = () => {}
	const promise = new Promise<T>((resolvePromise, rejectPromise) => {
		resolve = resolvePromise
		reject = rejectPromise
	})
	return { promise, resolve, reject }
}

export function installRadixPolyfills() {
	const prototype = window.HTMLElement.prototype
	if (!prototype.hasPointerCapture) prototype.hasPointerCapture = () => false
	if (!prototype.setPointerCapture) prototype.setPointerCapture = () => {}
	if (!prototype.releasePointerCapture)
		prototype.releasePointerCapture = () => {}
	if (!prototype.scrollIntoView) prototype.scrollIntoView = () => {}
}

export async function chooseOption(
	user: UserEvent,
	trigger: HTMLElement,
	optionName: string | RegExp,
) {
	await user.click(trigger)
	await user.click(await screen.findByRole('option', { name: optionName }))
}

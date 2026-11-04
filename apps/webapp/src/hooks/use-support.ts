import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback } from 'react'
import type { Schemas } from '#/api/api.client'
import { applyStatus } from '#/pages/support/lib/columns'
import { supportFiltersToQuery } from '#/pages/support/lib/filters'
import type { SupportFilters, SupportStatus } from '#/pages/support/types'

const TICKETS_PATH = '/api/v1/organizations/{organization_id}/support/tickets'
const TICKET_STATUS_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/status'
const SETTINGS_PATH = '/api/v1/organizations/{organization_id}/support/settings'
const TICKET_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}'
const TICKET_ENTRIES_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/entries'
const TICKET_ASSIGNEE_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/assignee'
const ENTRY_PATH =
	'/api/v1/organizations/{organization_id}/support/entries/{entry_id}'
const ENTRY_SEND_PATH =
	'/api/v1/organizations/{organization_id}/support/entries/{entry_id}/send'
const ATTACHMENT_URL_PATH =
	'/api/v1/organizations/{organization_id}/support/attachments/{attachment_id}/url'

export const SUPPORT_TICKETS_PER_PAGE = 100

function supportTicketsRequest(
	organizationId: string,
	filters: SupportFilters,
) {
	return {
		path: { organization_id: organizationId },
		query: supportFiltersToQuery(filters, SUPPORT_TICKETS_PER_PAGE),
	}
}

export function useSupportTickets(
	organizationId: string,
	filters: SupportFilters,
) {
	return useQuery({
		...window.tanstackApi.get(
			TICKETS_PATH,
			supportTicketsRequest(organizationId, filters),
		).queryOptions,
		enabled: Boolean(organizationId),
	})
}

interface SupportTicketsPage {
	data: Schemas.TicketResponse[]
	pagination?: Schemas.PaginationMetadata | null
}

interface MoveVariables {
	path: { organization_id: string; ticket_id: string }
	body: { status: SupportStatus }
}

export function useMoveSupportTicket(
	organizationId: string,
	filters: SupportFilters,
) {
	const queryClient = useQueryClient()
	const queryKey = window.tanstackApi.get(
		TICKETS_PATH,
		supportTicketsRequest(organizationId, filters),
	).queryKey

	return useMutation({
		...window.tanstackApi.mutation('patch', TICKET_STATUS_PATH).mutationOptions,
		onMutate: async (variables) => {
			const { path, body } = variables as MoveVariables
			await queryClient.cancelQueries({ queryKey })

			const previous = queryClient.getQueryData<SupportTicketsPage>(queryKey)
			if (previous) {
				queryClient.setQueryData<SupportTicketsPage>(queryKey, {
					...previous,
					data: applyStatus(previous.data, path.ticket_id, body.status),
				})
			}

			return { previous }
		},
		onError: (_error, _variables, context) => {
			const previous = (
				context as { previous?: SupportTicketsPage } | undefined
			)?.previous
			if (previous) queryClient.setQueryData(queryKey, previous)
		},
		onSettled: () => queryClient.invalidateQueries({ queryKey }),
	})
}

export function useSupportSettings(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(SETTINGS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: Boolean(organizationId),
	})
}

interface QueryKeyMeta {
	_id?: unknown
	path?: {
		organization_id?: unknown
		ticket_id?: unknown
	}
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function isTicketQuery(
	queryKey: readonly unknown[],
	organizationId: string,
	ticketId: string,
) {
	const meta = queryKeyMeta(queryKey)
	return (
		meta?._id === TICKET_PATH &&
		meta.path?.organization_id === organizationId &&
		meta.path?.ticket_id === ticketId
	)
}

function isThreadQuery(
	queryKey: readonly unknown[],
	organizationId: string,
	ticketId: string,
) {
	const meta = queryKeyMeta(queryKey)
	return (
		meta?._id === TICKET_ENTRIES_PATH &&
		meta.path?.organization_id === organizationId &&
		meta.path?.ticket_id === ticketId
	)
}

function isTicketListQuery(
	queryKey: readonly unknown[],
	organizationId: string,
) {
	const meta = queryKeyMeta(queryKey)
	return (
		meta?._id === TICKETS_PATH && meta.path?.organization_id === organizationId
	)
}

export function useSupportTicket(organizationId: string, ticketId: string) {
	return useQuery({
		...window.tanstackApi.get(TICKET_PATH, {
			path: { organization_id: organizationId, ticket_id: ticketId },
		}).queryOptions,
		enabled: Boolean(organizationId) && Boolean(ticketId),
	})
}

export function useSupportThread(organizationId: string, ticketId: string) {
	return useQuery({
		...window.tanstackApi.get(TICKET_ENTRIES_PATH, {
			path: { organization_id: organizationId, ticket_id: ticketId },
		}).queryOptions,
		enabled: Boolean(organizationId) && Boolean(ticketId),
	})
}

function useRefreshThreadAndTicket(organizationId: string, ticketId: string) {
	const queryClient = useQueryClient()

	return () =>
		queryClient.invalidateQueries({
			predicate: (query) =>
				isThreadQuery(query.queryKey, organizationId, ticketId) ||
				isTicketQuery(query.queryKey, organizationId, ticketId),
		})
}

function useRefreshTicketAndLists(organizationId: string, ticketId: string) {
	const queryClient = useQueryClient()

	return () =>
		queryClient.invalidateQueries({
			predicate: (query) =>
				isTicketQuery(query.queryKey, organizationId, ticketId) ||
				isTicketListQuery(query.queryKey, organizationId),
		})
}

export function useAddSupportEntry(organizationId: string, ticketId: string) {
	return useMutation({
		...window.tanstackApi.mutation('post', TICKET_ENTRIES_PATH).mutationOptions,
		onSettled: useRefreshThreadAndTicket(organizationId, ticketId),
	})
}

export function useEditSupportDraft(organizationId: string, ticketId: string) {
	return useMutation({
		...window.tanstackApi.mutation('patch', ENTRY_PATH).mutationOptions,
		onSettled: useRefreshThreadAndTicket(organizationId, ticketId),
	})
}

export function useSendSupportDraft(organizationId: string, ticketId: string) {
	return useMutation({
		...window.tanstackApi.mutation('post', ENTRY_SEND_PATH).mutationOptions,
		onSettled: useRefreshThreadAndTicket(organizationId, ticketId),
	})
}

export function useDeleteSupportDraft(
	organizationId: string,
	ticketId: string,
) {
	return useMutation({
		...window.tanstackApi.mutation('delete', ENTRY_PATH).mutationOptions,
		onSettled: useRefreshThreadAndTicket(organizationId, ticketId),
	})
}

export function useTransitionSupportTicket(
	organizationId: string,
	ticketId: string,
) {
	return useMutation({
		...window.tanstackApi.mutation('patch', TICKET_STATUS_PATH).mutationOptions,
		onSettled: useRefreshTicketAndLists(organizationId, ticketId),
	})
}

export function useAssignSupportTicket(
	organizationId: string,
	ticketId: string,
) {
	return useMutation({
		...window.tanstackApi.mutation('put', TICKET_ASSIGNEE_PATH).mutationOptions,
		onSettled: useRefreshTicketAndLists(organizationId, ticketId),
	})
}

export function useOpenSupportAttachment(organizationId: string) {
	const queryClient = useQueryClient()

	return useCallback(
		async (attachmentId: string) => {
			const response = await queryClient.fetchQuery({
				...window.tanstackApi.get(ATTACHMENT_URL_PATH, {
					path: {
						organization_id: organizationId,
						attachment_id: attachmentId,
					},
				}).queryOptions,
				staleTime: 0,
			})
			window.open(response.data.url, '_blank', 'noopener,noreferrer')
		},
		[queryClient, organizationId],
	)
}

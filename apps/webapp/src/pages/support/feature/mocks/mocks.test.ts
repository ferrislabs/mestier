import { act, renderHook, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it } from 'vitest'
import {
	createApiKey,
	revokeApiKey,
	sendTestRequest,
	useApiKeys,
} from '#/pages/support/feature/mocks/api-keys'
import {
	clearDraftReviews,
	commentDraft,
	resetDraftReviews,
	toggleDraftApproval,
	useDraftReviews,
} from '#/pages/support/feature/mocks/draft-reviews'
import {
	resetTicketPriorities,
	setTicketPriority,
	useTicketPriorities,
} from '#/pages/support/feature/mocks/priorities'

describe('priorities mock', () => {
	beforeEach(() => resetTicketPriorities())

	it('starts without priority and stores a chosen one', async () => {
		const { result } = renderHook(() => useTicketPriorities())
		expect(result.current.data).toEqual({})

		await act(() => setTicketPriority('ticket-1', 'urgent'))

		expect(result.current.data).toEqual({ 'ticket-1': 'urgent' })
		expect(result.current.isLoading).toBe(false)
	})
})

describe('draft reviews mock', () => {
	beforeEach(() => resetDraftReviews())

	it('toggles an approval on and off for one member', async () => {
		const { result } = renderHook(() => useDraftReviews())

		await act(() => toggleDraftApproval('entry-1', 'member-1'))
		expect(result.current.data['entry-1']).toHaveLength(1)
		expect(result.current.data['entry-1'][0]).toMatchObject({
			kind: 'approval',
			memberId: 'member-1',
		})

		await act(() => toggleDraftApproval('entry-1', 'member-1'))
		expect(result.current.data['entry-1']).toEqual([])
	})

	it('keeps the approvals of two members apart', async () => {
		const { result } = renderHook(() => useDraftReviews())

		await act(() => toggleDraftApproval('entry-1', 'member-1'))
		await act(() => toggleDraftApproval('entry-1', 'member-2'))
		await act(() => toggleDraftApproval('entry-1', 'member-1'))

		expect(
			result.current.data['entry-1'].map((review) => review.memberId),
		).toEqual(['member-2'])
	})

	it('stores a trimmed comment', async () => {
		const { result } = renderHook(() => useDraftReviews())

		await act(() => commentDraft('entry-1', 'member-1', '  Plus court ?  '))

		expect(result.current.data['entry-1'][0]).toMatchObject({
			kind: 'comment',
			body: 'Plus court ?',
		})
	})

	it('forgets the reviews of a draft that changed', async () => {
		const { result } = renderHook(() => useDraftReviews())
		await act(() => toggleDraftApproval('entry-1', 'member-1'))

		act(() => clearDraftReviews('entry-1'))

		expect(result.current.data['entry-1']).toBeUndefined()
	})
})

describe('api keys mock', () => {
	it('starts with a live key and a revoked one', () => {
		const { result } = renderHook(() => useApiKeys())

		expect(result.current.data.map((key) => key.revokedAt === null)).toEqual([
			true,
			false,
		])
	})

	it('creates a key with the prefix of its secret, first in the list', async () => {
		const { result } = renderHook(() => useApiKeys())

		const key = await act(() =>
			createApiKey(' Mon site ', 'mst_live_abcdEFGH1234'),
		)

		expect(key.name).toBe('Mon site')
		expect(key.prefix).toBe('mst_live_abcd')
		expect(result.current.data[0].id).toBe(key.id)
	})

	it('revokes a key', async () => {
		const { result } = renderHook(() => useApiKeys())

		await act(() => revokeApiKey('mock-key-1'))

		await waitFor(() =>
			expect(
				result.current.data.find((key) => key.id === 'mock-key-1')?.revokedAt,
			).not.toBeNull(),
		)
	})

	it('creates a ticket for a new external id and returns the duplicate for a known one', async () => {
		const first = await sendTestRequest('mock-key-1', 'EC-1')
		const again = await sendTestRequest('mock-key-1', 'EC-1')
		const other = await sendTestRequest('mock-key-1', null)

		expect(first.kind).toBe('created')
		expect(again).toEqual({
			kind: 'duplicate',
			ticketId: first.ticketId,
			ticketNumber: first.ticketNumber,
		})
		expect(other.kind).toBe('created')
		expect(other.ticketNumber).toBe(first.ticketNumber + 1)
	})
})

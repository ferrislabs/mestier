import { describe, expect, it } from 'vitest'
import {
	initialMessagesState,
	type MessagesState,
	messagesReducer,
} from '#/pages/chat/lib/messages-reducer'

function message(overrides: Partial<Record<string, unknown>> = {}) {
	return {
		id: 'm-1',
		organization_id: 'org-1',
		channel_id: 'ch-1',
		author_type: 'USER' as const,
		author_user_id: 'me',
		author_webhook_id: null,
		content: 'hello',
		components: null,
		mention_user_ids: [],
		mention_role_ids: [],
		mention_channel_ids: [],
		mention_everyone: false,
		reactions: [],
		attachments: [],
		edited_at: null,
		created_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

describe('messagesReducer — initial and older pages', () => {
	it('reverses the newest-first server order to oldest-first for display', () => {
		const state = messagesReducer(initialMessagesState, {
			type: 'initial-page-loaded',
			messages: [
				message({ id: 'm-3' }),
				message({ id: 'm-2' }),
				message({ id: 'm-1' }),
			],
			hasMore: true,
		})

		expect(state.messages.map((entry) => entry.message.id)).toEqual([
			'm-1',
			'm-2',
			'm-3',
		])
		expect(state.hasMoreOlder).toBe(true)
	})

	it('prepends an older page, oldest at the very front', () => {
		const afterInitial = messagesReducer(initialMessagesState, {
			type: 'initial-page-loaded',
			messages: [message({ id: 'm-2' }), message({ id: 'm-1' })],
			hasMore: true,
		})

		const afterOlder = messagesReducer(afterInitial, {
			type: 'older-page-loaded',
			messages: [message({ id: 'm-0' })],
			hasMore: false,
		})

		expect(afterOlder.messages.map((entry) => entry.message.id)).toEqual([
			'm-0',
			'm-1',
			'm-2',
		])
		expect(afterOlder.hasMoreOlder).toBe(false)
	})

	it('tracks the loading flag around an older-page request', () => {
		const requested = messagesReducer(initialMessagesState, {
			type: 'older-page-requested',
		})
		expect(requested.isLoadingOlder).toBe(true)

		const loaded = messagesReducer(requested, {
			type: 'older-page-loaded',
			messages: [],
			hasMore: false,
		})
		expect(loaded.isLoadingOlder).toBe(false)
	})
})

describe('messagesReducer — optimistic send and its own echo', () => {
	function withOptimisticSend(): MessagesState {
		return messagesReducer(initialMessagesState, {
			type: 'optimistic-send',
			tempId: 'temp-1',
			message: message({ id: 'temp-1', content: 'hey there' }),
		})
	}

	it('shows the optimistic message immediately as sending', () => {
		const state = withOptimisticSend()
		expect(state.messages).toHaveLength(1)
		expect(state.messages[0]?.status).toBe('sending')
		expect(state.messages[0]?.tempId).toBe('temp-1')
	})

	it('reconciles with the REST response when it arrives first', () => {
		const sent = messagesReducer(withOptimisticSend(), {
			type: 'send-succeeded',
			tempId: 'temp-1',
			message: message({ id: 'm-real', content: 'hey there' }),
		})

		expect(sent.messages).toHaveLength(1)
		expect(sent.messages[0]?.message.id).toBe('m-real')
		expect(sent.messages[0]?.status).toBe('sent')
		expect(sent.messages[0]?.tempId).toBeUndefined()
	})

	it('reconciles with its own gateway echo when that arrives first — no duplicate', () => {
		const state = withOptimisticSend()
		const echoed = messagesReducer(state, {
			type: 'remote-create',
			message: message({
				id: 'm-real',
				author_user_id: 'me',
				content: 'hey there',
			}),
		})

		expect(echoed.messages).toHaveLength(1)
		expect(echoed.messages[0]?.message.id).toBe('m-real')
		expect(echoed.messages[0]?.status).toBe('sent')

		// The REST response for the same send now arrives — must be a no-op,
		// not a second entry.
		const afterLateRestResponse = messagesReducer(echoed, {
			type: 'send-succeeded',
			tempId: 'temp-1',
			message: message({
				id: 'm-real',
				author_user_id: 'me',
				content: 'hey there',
			}),
		})
		expect(afterLateRestResponse.messages).toHaveLength(1)
	})

	it('does not reconcile someone else’s message against my pending send', () => {
		const state = withOptimisticSend()
		const fromSomeoneElse = messagesReducer(state, {
			type: 'remote-create',
			message: message({
				id: 'm-other',
				author_user_id: 'someone-else',
				content: 'hey there',
			}),
		})

		expect(fromSomeoneElse.messages).toHaveLength(2)
		expect(
			fromSomeoneElse.messages.some((entry) => entry.status === 'sending'),
		).toBe(true)
	})

	it('does not reconcile against a pending send with different content', () => {
		const state = withOptimisticSend()
		const differentContent = messagesReducer(state, {
			type: 'remote-create',
			message: message({
				id: 'm-other',
				author_user_id: 'me',
				content: 'unrelated',
			}),
		})

		expect(differentContent.messages).toHaveLength(2)
	})

	it('marks a failed send as failed, keeping it visible and retryable', () => {
		const failed = messagesReducer(withOptimisticSend(), {
			type: 'send-failed',
			tempId: 'temp-1',
		})

		expect(failed.messages).toHaveLength(1)
		expect(failed.messages[0]?.status).toBe('failed')
	})

	it('retry moves a failed message back to sending', () => {
		const failed = messagesReducer(withOptimisticSend(), {
			type: 'send-failed',
			tempId: 'temp-1',
		})
		const retried = messagesReducer(failed, {
			type: 'retry-send',
			tempId: 'temp-1',
		})

		expect(retried.messages[0]?.status).toBe('sending')
	})
})

describe('messagesReducer — remote updates and deletes', () => {
	it('ignores a duplicate remote-create for an id already present', () => {
		const state = messagesReducer(initialMessagesState, {
			type: 'initial-page-loaded',
			messages: [message({ id: 'm-1' })],
			hasMore: false,
		})
		const again = messagesReducer(state, {
			type: 'remote-create',
			message: message({ id: 'm-1' }),
		})
		expect(again.messages).toHaveLength(1)
	})

	it('applies a remote edit in place', () => {
		const state = messagesReducer(initialMessagesState, {
			type: 'initial-page-loaded',
			messages: [message({ id: 'm-1', content: 'before' })],
			hasMore: false,
		})
		const edited = messagesReducer(state, {
			type: 'remote-update',
			message: message({
				id: 'm-1',
				content: 'after',
				edited_at: '2026-01-02T00:00:00Z',
			}),
		})

		expect(edited.messages[0]?.message.content).toBe('after')
		expect(edited.messages[0]?.message.edited_at).toBe('2026-01-02T00:00:00Z')
	})

	it('ignores an edit for a message not currently loaded', () => {
		const edited = messagesReducer(initialMessagesState, {
			type: 'remote-update',
			message: message({ id: 'unknown' }),
		})
		expect(edited.messages).toHaveLength(0)
	})

	it('removes a deleted message', () => {
		const state = messagesReducer(initialMessagesState, {
			type: 'initial-page-loaded',
			messages: [message({ id: 'm-1' }), message({ id: 'm-2' })],
			hasMore: false,
		})
		const deleted = messagesReducer(state, {
			type: 'remote-delete',
			messageId: 'm-1',
		})

		expect(deleted.messages.map((entry) => entry.message.id)).toEqual(['m-2'])
	})
})

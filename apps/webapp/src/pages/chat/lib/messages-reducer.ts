import type { Schemas } from '#/api/api.client'

export type MessageStatus = 'sent' | 'sending' | 'failed'

export interface DisplayMessage {
	message: Schemas.MessageResponse
	status: MessageStatus
	/** Only set while `status` is `sending`/`failed` — the key a later
	 * `send-succeeded`/`send-failed` uses to find this entry again. */
	tempId?: string
}

export interface MessagesState {
	/** Oldest first — the order a chat renders in. */
	messages: DisplayMessage[]
	hasMoreOlder: boolean
	isLoadingOlder: boolean
}

export const initialMessagesState: MessagesState = {
	messages: [],
	hasMoreOlder: true,
	isLoadingOlder: false,
}

export type MessagesAction =
	| {
			type: 'initial-page-loaded'
			messages: Schemas.MessageResponse[]
			hasMore: boolean
	  }
	| {
			type: 'older-page-requested'
	  }
	| {
			type: 'older-page-loaded'
			messages: Schemas.MessageResponse[]
			hasMore: boolean
	  }
	| {
			type: 'optimistic-send'
			tempId: string
			message: Schemas.MessageResponse
	  }
	| { type: 'send-succeeded'; tempId: string; message: Schemas.MessageResponse }
	| { type: 'send-failed'; tempId: string }
	| { type: 'retry-send'; tempId: string }
	| { type: 'remote-create'; message: Schemas.MessageResponse }
	| { type: 'remote-update'; message: Schemas.MessageResponse }
	| { type: 'remote-delete'; messageId: string }
	| { type: 'reaction-add'; messageId: string; emoji: string; userId: string }
	| {
			type: 'reaction-remove'
			messageId: string
			emoji: string
			userId: string
	  }

export function messagesReducer(
	state: MessagesState,
	action: MessagesAction,
): MessagesState {
	switch (action.type) {
		case 'initial-page-loaded': {
			// Server returns newest-first for every cursor direction (see
			// `message/postgres/repository.rs`) — reversed once, here, so
			// everything downstream works in the oldest-first order a chat
			// renders in.
			const messages = [...action.messages]
				.reverse()
				.map((message) => toSent(message))
			return { messages, hasMoreOlder: action.hasMore, isLoadingOlder: false }
		}

		case 'older-page-requested': {
			return { ...state, isLoadingOlder: true }
		}

		case 'older-page-loaded': {
			const older = [...action.messages]
				.reverse()
				.map((message) => toSent(message))
			return {
				messages: dedupeById([...older, ...state.messages]),
				hasMoreOlder: action.hasMore,
				isLoadingOlder: false,
			}
		}

		case 'optimistic-send': {
			return {
				...state,
				messages: [
					...state.messages,
					{ message: action.message, status: 'sending', tempId: action.tempId },
				],
			}
		}

		case 'send-succeeded': {
			const index = state.messages.findIndex(
				(entry) => entry.tempId === action.tempId,
			)
			if (index === -1) {
				// Already reconciled by a `remote-create` echo that beat the REST
				// response back — upsert by id defensively rather than duplicate.
				return {
					...state,
					messages: upsertById(state.messages, action.message),
				}
			}
			const messages = [...state.messages]
			messages[index] = toSent(action.message)
			return { ...state, messages }
		}

		case 'send-failed': {
			const index = state.messages.findIndex(
				(entry) => entry.tempId === action.tempId,
			)
			if (index === -1) return state
			const messages = [...state.messages]
			const entry = messages[index]
			if (!entry) return state
			messages[index] = { ...entry, status: 'failed' }
			return { ...state, messages }
		}

		case 'retry-send': {
			const index = state.messages.findIndex(
				(entry) => entry.tempId === action.tempId,
			)
			if (index === -1) return state
			const messages = [...state.messages]
			const entry = messages[index]
			if (!entry) return state
			messages[index] = { ...entry, status: 'sending' }
			return { ...state, messages }
		}

		case 'remote-create': {
			if (
				state.messages.some((entry) => entry.message.id === action.message.id)
			) {
				return state
			}

			// The fiddly part: a gateway echo of a message this tab itself just
			// sent must resolve the pending optimistic entry, not duplicate it.
			// There is no client-supplied nonce round-tripped by the server, so
			// the match is by (author, content) against the oldest still-pending
			// send — good enough for one tab sending one message at a time, which
			// is the actual case this guards.
			const pendingIndex = state.messages.findIndex(
				(entry) =>
					entry.status === 'sending' &&
					entry.message.author_user_id === action.message.author_user_id &&
					entry.message.content === action.message.content,
			)
			if (pendingIndex !== -1) {
				const messages = [...state.messages]
				messages[pendingIndex] = toSent(action.message)
				return { ...state, messages }
			}

			return { ...state, messages: [...state.messages, toSent(action.message)] }
		}

		case 'remote-update': {
			// Ignore, don't insert, when the message isn't currently loaded (e.g.
			// an edit to a message on a page further back than what's fetched) —
			// it will render correctly once that page is loaded.
			const index = state.messages.findIndex(
				(entry) => entry.message.id === action.message.id,
			)
			if (index === -1) return state
			const messages = [...state.messages]
			messages[index] = toSent(action.message)
			return { ...state, messages }
		}

		case 'remote-delete': {
			return {
				...state,
				messages: state.messages.filter(
					(entry) => entry.message.id !== action.messageId,
				),
			}
		}

		// Reactions are a set-membership toggle, not a created resource with a
		// fresh id — (message, emoji, user) is the whole identity. That makes
		// applying it idempotent: the same action from an optimistic click and
		// from its own gateway echo lands on the same state either way, so
		// there is no id to reconcile and no separate echo-matching path like
		// `remote-create`'s. Reverting a failed optimistic add is just
		// dispatching the opposite action.
		case 'reaction-add': {
			return {
				...state,
				messages: state.messages.map((entry) =>
					entry.message.id === action.messageId
						? { ...entry, message: withReaction(entry.message, action, true) }
						: entry,
				),
			}
		}

		case 'reaction-remove': {
			return {
				...state,
				messages: state.messages.map((entry) =>
					entry.message.id === action.messageId
						? { ...entry, message: withReaction(entry.message, action, false) }
						: entry,
				),
			}
		}

		default:
			return state
	}
}

function withReaction(
	message: Schemas.MessageResponse,
	action: { emoji: string; userId: string },
	add: boolean,
): Schemas.MessageResponse {
	const groups = message.reactions
	const index = groups.findIndex((group) => group.emoji === action.emoji)
	const existing = index === -1 ? null : groups[index]

	const userIds = new Set(existing?.user_ids ?? [])
	if (add) userIds.add(action.userId)
	else userIds.delete(action.userId)

	if (userIds.size === 0) {
		if (index === -1) return message
		return {
			...message,
			reactions: groups.filter((_, i) => i !== index),
		}
	}

	const nextGroup = {
		emoji: action.emoji,
		count: userIds.size,
		user_ids: [...userIds],
	}
	const nextGroups =
		index === -1
			? [...groups, nextGroup]
			: groups.map((group, i) => (i === index ? nextGroup : group))

	return { ...message, reactions: nextGroups }
}

function toSent(message: Schemas.MessageResponse): DisplayMessage {
	return { message, status: 'sent' }
}

function upsertById(
	messages: DisplayMessage[],
	message: Schemas.MessageResponse,
): DisplayMessage[] {
	const index = messages.findIndex((entry) => entry.message.id === message.id)
	if (index === -1) return [...messages, toSent(message)]
	const next = [...messages]
	next[index] = toSent(message)
	return next
}

function dedupeById(messages: DisplayMessage[]): DisplayMessage[] {
	const seen = new Set<string>()
	const result: DisplayMessage[] = []
	for (const entry of messages) {
		if (seen.has(entry.message.id)) continue
		seen.add(entry.message.id)
		result.push(entry)
	}
	return result
}

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useReducer, useRef, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { useGatewayEvent } from '#/hooks/use-gateway'
import {
	initialMessagesState,
	messagesReducer,
} from '#/pages/chat/lib/messages-reducer'

const ORG_CATEGORIES_PATH =
	'/api/v1/chat/organizations/{organization_id}/categories'
const ORG_CHANNELS_PATH =
	'/api/v1/chat/organizations/{organization_id}/channels'
const CHANNEL_PATH = '/api/v1/chat/channels/{channel_id}'
const CHANNEL_MESSAGES_PATH = '/api/v1/chat/channels/{channel_id}/messages'
const MESSAGE_PATH = '/api/v1/chat/messages/{message_id}'
const REACTION_PATH = '/api/v1/chat/messages/{message_id}/reactions/{emoji}'
const CHANNEL_THREADS_PATH = '/api/v1/chat/channels/{channel_id}/threads'

/** Server caps `limit` at 100 (`MessageCursorQuery::effective_limit`); 50
 * matches its own default. Also doubles as the "did that page run out"
 * signal: a page shorter than this is the last one. */
const MESSAGES_PAGE_SIZE = 50

export type Category = Schemas.CategoryResponse
export type Channel = Schemas.ChannelResponse
export type Message = Schemas.MessageResponse
export type MessageAttachment = Schemas.CreateMessageAttachment

function categoriesKey(organizationId: string) {
	return window.tanstackApi.get(ORG_CATEGORIES_PATH, {
		path: { organization_id: organizationId },
	}).queryKey
}

function channelsKey(organizationId: string) {
	return window.tanstackApi.get(ORG_CHANNELS_PATH, {
		path: { organization_id: organizationId },
	}).queryKey
}

export function useCategories(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(ORG_CATEGORIES_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		// Every list/get endpoint on this backend wraps its payload in a
		// `DataEnvelope` (`{ data, pagination }`) — `select` unwraps it once,
		// here, rather than every caller reaching for `.data.data` (the
		// pattern the raw, un-abstracted hooks like `useCustomers` use).
		select: (response) => response.data,
		enabled: Boolean(organizationId),
	})
}

/**
 * TEXT channels only — threads (`channel_type === 'THREAD'`) are fetched per
 * parent channel (see #326), not listed at the organization level. The
 * server already scopes this to `list_by_organization`, which does not
 * return threads; the filter here is a cheap, harmless belt-and-braces.
 */
export function useChannels(organizationId: string) {
	return useQuery({
		...window.tanstackApi.get(ORG_CHANNELS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		select: (response) =>
			response.data.filter((channel) => channel.channel_type === 'TEXT'),
		enabled: Boolean(organizationId),
	})
}

/** A single channel by id — used for the channel header once one is active. */
export function useChannel(channelId: string) {
	return useQuery({
		...window.tanstackApi.get(CHANNEL_PATH, {
			path: { channel_id: channelId },
		}).queryOptions,
		select: (response) => response.data,
		enabled: Boolean(channelId),
	})
}

/**
 * Subscribes the org's category/channel list queries to the gateway so a
 * create, rename or delete from any tab updates every other tab's sidebar
 * without a refetch. Mount once, near the top of the chat page.
 */
export function useChatListGatewaySync(organizationId: string) {
	const queryClient = useQueryClient()

	useGatewayEvent('CATEGORY_CREATE', (event) => {
		if (event.data.organization_id !== organizationId) return
		upsertInList(queryClient, categoriesKey(organizationId), event.data)
	})
	useGatewayEvent('CATEGORY_UPDATE', (event) => {
		if (event.data.organization_id !== organizationId) return
		upsertInList(queryClient, categoriesKey(organizationId), event.data)
	})
	useGatewayEvent('CATEGORY_DELETE', (event) => {
		if (event.data.organization_id !== organizationId) return
		removeFromList(
			queryClient,
			categoriesKey(organizationId),
			event.data.category_id,
		)
	})

	useGatewayEvent('CHANNEL_CREATE', (event) => {
		if (event.data.organization_id !== organizationId) return
		upsertInList(queryClient, channelsKey(organizationId), event.data)
	})
	useGatewayEvent('CHANNEL_UPDATE', (event) => {
		if (event.data.organization_id !== organizationId) return
		upsertInList(queryClient, channelsKey(organizationId), event.data)
	})
	useGatewayEvent('CHANNEL_DELETE', (event) => {
		if (event.data.organization_id !== organizationId) return
		removeFromList(
			queryClient,
			channelsKey(organizationId),
			event.data.channel_id,
		)
	})
}

/** The shape every list query actually caches — the raw `DataEnvelope`, not
 * the `select`-unwrapped array a `useQuery` caller sees (`select` runs on
 * read; the cache itself is never transformed by it). */
interface ListEnvelope<T> {
	data: T[]
	pagination?: unknown
}

function upsertInList<T extends { id: string }>(
	queryClient: ReturnType<typeof useQueryClient>,
	key: readonly unknown[],
	item: T,
) {
	queryClient.setQueryData<ListEnvelope<T>>(key, (old) => {
		if (!old) return { data: [item] }
		const index = old.data.findIndex((existing) => existing.id === item.id)
		if (index === -1) return { ...old, data: [...old.data, item] }
		const next = [...old.data]
		next[index] = item
		return { ...old, data: next }
	})
}

function removeFromList(
	queryClient: ReturnType<typeof useQueryClient>,
	key: readonly unknown[],
	id: string,
) {
	queryClient.setQueryData<ListEnvelope<{ id: string }>>(key, (old) =>
		old ? { ...old, data: old.data.filter((item) => item.id !== id) } : old,
	)
}

// ── Threads ───────────────────────────────────────────────────────────────
// A thread is a `Channel` (`channel_type: 'THREAD'`) with `parent_id` and
// `origin_message_id` set — the server's model, per `response.rs`; the
// frontend reuses the channel/message endpoints wholesale rather than
// inventing a parallel "thread message" concept.

function threadsKey(channelId: string) {
	return window.tanstackApi.get(CHANNEL_THREADS_PATH, {
		path: { channel_id: channelId },
	}).queryKey
}

/** Threads spawned from messages in this channel. */
export function useThreads(channelId: string) {
	return useQuery({
		...window.tanstackApi.get(CHANNEL_THREADS_PATH, {
			path: { channel_id: channelId },
		}).queryOptions,
		select: (response) => response.data,
		enabled: Boolean(channelId),
	})
}

export function useCreateThread() {
	return useMutation({
		...window.tanstackApi.mutation('post', CHANNEL_THREADS_PATH)
			.mutationOptions,
	})
}

/** Mirrors `useChatListGatewaySync`, scoped to one channel's threads. */
export function useThreadsGatewaySync(channelId: string) {
	const queryClient = useQueryClient()

	useGatewayEvent('THREAD_CREATE', (event) => {
		if (event.data.parent_id !== channelId) return
		upsertInList(queryClient, threadsKey(channelId), event.data)
	})
	useGatewayEvent('THREAD_UPDATE', (event) => {
		if (event.data.parent_id !== channelId) return
		upsertInList(queryClient, threadsKey(channelId), event.data)
	})
	useGatewayEvent('THREAD_DELETE', (event) => {
		removeFromList(queryClient, threadsKey(channelId), event.data.channel_id)
	})
}

// ── Messages ──────────────────────────────────────────────────────────────

function createTempId(): string {
	return `temp:${crypto.randomUUID()}`
}

async function fetchMessagesPage(
	channelId: string,
	before?: string,
): Promise<Message[]> {
	// `window.api` (unlike `window.tanstackApi`) has no `select` to unwrap
	// the `DataEnvelope` — every endpoint here wraps its payload in
	// `{ data, pagination }`, so `.data` is read out by hand.
	const response = (await window.api.get(CHANNEL_MESSAGES_PATH, {
		path: { channel_id: channelId },
		query: { limit: MESSAGES_PAGE_SIZE, ...(before ? { before } : {}) },
	} as never)) as { data: Message[] }
	return response.data
}

/**
 * A channel's message thread: paginated history, optimistic sending
 * reconciled against its own gateway echo (see `messages-reducer.ts`, where
 * that reconciliation is unit-tested directly), editing and deleting one's
 * own messages, and live updates from the gateway. One instance per channel
 * — callers should mount it keyed by `channelId` (e.g. `key={channelId}` on
 * the route's channel component) so switching channels starts fresh rather
 * than carrying over another channel's history.
 */
export function useMessages(channelId: string, currentUserId: string | null) {
	const [state, dispatch] = useReducer(messagesReducer, initialMessagesState)
	const [isLoadingInitial, setIsLoadingInitial] = useState(true)
	const [initialError, setInitialError] = useState<string | null>(null)
	const oldestIdRef = useRef<string | null>(null)
	oldestIdRef.current = state.messages[0]?.message.id ?? null

	useEffect(() => {
		let cancelled = false
		setIsLoadingInitial(true)
		setInitialError(null)

		fetchMessagesPage(channelId)
			.then((messages) => {
				if (cancelled) return
				dispatch({
					type: 'initial-page-loaded',
					messages,
					hasMore: messages.length === MESSAGES_PAGE_SIZE,
				})
			})
			.catch(() => {
				if (!cancelled) setInitialError('load-failed')
			})
			.finally(() => {
				if (!cancelled) setIsLoadingInitial(false)
			})

		return () => {
			cancelled = true
		}
	}, [channelId])

	const loadOlder = useCallback(() => {
		const before = oldestIdRef.current
		if (!before || state.isLoadingOlder || !state.hasMoreOlder) return

		dispatch({ type: 'older-page-requested' })
		fetchMessagesPage(channelId, before)
			.then((messages) => {
				dispatch({
					type: 'older-page-loaded',
					messages,
					hasMore: messages.length === MESSAGES_PAGE_SIZE,
				})
			})
			.catch(() => {
				// Stay put rather than silently dropping "hasMoreOlder": the
				// reader can retry by scrolling up again.
				dispatch({ type: 'older-page-loaded', messages: [], hasMore: true })
			})
	}, [channelId, state.isLoadingOlder, state.hasMoreOlder])

	const sendMessage = useCallback(
		(content: string, attachments: MessageAttachment[] = []) => {
			const tempId = createTempId()
			const optimistic: Message = {
				id: tempId,
				organization_id: '',
				channel_id: channelId,
				author_type: 'USER',
				author_user_id: currentUserId ?? '',
				author_webhook_id: null,
				content,
				components: null,
				mention_user_ids: [],
				mention_role_ids: [],
				mention_channel_ids: [],
				mention_everyone: false,
				reactions: [],
				attachments: attachments.map((a) => ({ ...a })),
				edited_at: null,
				created_at: new Date().toISOString(),
			}
			dispatch({ type: 'optimistic-send', tempId, message: optimistic })

			window.api
				.post(CHANNEL_MESSAGES_PATH, {
					path: { channel_id: channelId },
					body: { content, attachments },
				} as never)
				.then((response) => {
					dispatch({
						type: 'send-succeeded',
						tempId,
						message: (response as { data: Message }).data,
					})
				})
				.catch(() => {
					dispatch({ type: 'send-failed', tempId })
				})
		},
		[channelId, currentUserId],
	)

	const retrySend = useCallback(
		(tempId: string) => {
			const entry = state.messages.find((e) => e.tempId === tempId)
			if (!entry) return

			dispatch({ type: 'retry-send', tempId })
			window.api
				.post(CHANNEL_MESSAGES_PATH, {
					path: { channel_id: channelId },
					body: {
						content: entry.message.content,
						attachments: entry.message.attachments.map((a) => ({
							storage_key: a.storage_key,
							filename: a.filename,
							mime_type: a.mime_type,
							size_bytes: a.size_bytes,
						})),
					},
				} as never)
				.then((response) => {
					dispatch({
						type: 'send-succeeded',
						tempId,
						message: (response as { data: Message }).data,
					})
				})
				.catch(() => {
					dispatch({ type: 'send-failed', tempId })
				})
		},
		[channelId, state.messages],
	)

	const editMessage = useCallback((messageId: string, content: string) => {
		return window.api
			.patch(MESSAGE_PATH, {
				path: { message_id: messageId },
				body: { content },
			} as never)
			.then((response) => {
				dispatch({
					type: 'remote-update',
					message: (response as { data: Message }).data,
				})
			})
	}, [])

	const deleteMessage = useCallback((messageId: string) => {
		return window.api
			.delete(MESSAGE_PATH, { path: { message_id: messageId } } as never)
			.then(() => {
				dispatch({ type: 'remote-delete', messageId })
			})
	}, [])

	/**
	 * Reactions are a set-membership toggle keyed by (message, emoji, user) —
	 * there is no id to reconcile against a gateway echo the way a new
	 * message has, so applying the same action twice (the optimistic click,
	 * then its own echo) is simply a no-op. See the reducer's comment on
	 * `reaction-add` for the detail. A failed request reverts by dispatching
	 * the opposite action — cheap, per the issue's own framing.
	 */
	const toggleReaction = useCallback(
		(messageId: string, emoji: string, currentlyReacted: boolean) => {
			if (!currentUserId) return
			const apply = currentlyReacted ? 'reaction-remove' : 'reaction-add'
			const revert = currentlyReacted ? 'reaction-add' : 'reaction-remove'

			dispatch({ type: apply, messageId, emoji, userId: currentUserId })

			const request = currentlyReacted
				? window.api.delete(REACTION_PATH, {
						path: { message_id: messageId, emoji },
					} as never)
				: window.api.put(REACTION_PATH, {
						path: { message_id: messageId, emoji },
					} as never)

			request.catch(() => {
				dispatch({ type: revert, messageId, emoji, userId: currentUserId })
			})
		},
		[currentUserId],
	)

	useGatewayEvent('MESSAGE_CREATE', (event) => {
		if (event.data.channel_id !== channelId) return
		dispatch({ type: 'remote-create', message: event.data })
	})
	useGatewayEvent('MESSAGE_UPDATE', (event) => {
		if (event.data.channel_id !== channelId) return
		dispatch({ type: 'remote-update', message: event.data })
	})
	useGatewayEvent('MESSAGE_DELETE', (event) => {
		if (event.data.channel_id !== channelId) return
		dispatch({ type: 'remote-delete', messageId: event.data.message_id })
	})
	useGatewayEvent('REACTION_ADD', (event) => {
		if (event.data.channel_id !== channelId) return
		dispatch({
			type: 'reaction-add',
			messageId: event.data.message_id,
			emoji: event.data.emoji,
			userId: event.data.user_id,
		})
	})
	useGatewayEvent('REACTION_REMOVE', (event) => {
		if (event.data.channel_id !== channelId) return
		dispatch({
			type: 'reaction-remove',
			messageId: event.data.message_id,
			emoji: event.data.emoji,
			userId: event.data.user_id,
		})
	})

	return {
		messages: state.messages,
		hasMoreOlder: state.hasMoreOlder,
		isLoadingOlder: state.isLoadingOlder,
		isLoadingInitial,
		initialError,
		loadOlder,
		sendMessage,
		retrySend,
		editMessage,
		deleteMessage,
		toggleReaction,
	}
}

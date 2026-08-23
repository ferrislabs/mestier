import { useMutation } from '@tanstack/react-query'
import { useEffect, useRef, useState } from 'react'
import { useGatewayEvent } from '#/hooks/use-gateway'

const CHANNEL_TYPING_PATH = '/api/v1/chat/channels/{channel_id}/typing'

/** Minimum time between two `typing` notifications for the same channel —
 * the debounce the issue asks for: not one call per keystroke. */
const TYPING_SEND_DEBOUNCE_MS = 3_000

/**
 * Returns a function the message composer calls on every keystroke; it only
 * actually notifies the server at most once per debounce window. Built here
 * so #325's composer has something to call — this issue owns the debounce
 * and the receiving side, not the text input itself.
 */
export function useSendTyping(channelId: string) {
	const mutation = useMutation({
		...window.tanstackApi.mutation('post', CHANNEL_TYPING_PATH).mutationOptions,
	})
	const lastSentAtRef = useRef(0)
	const mutateRef = useRef(mutation.mutate)
	mutateRef.current = mutation.mutate

	return function notifyTyping() {
		const now = Date.now()
		if (now - lastSentAtRef.current < TYPING_SEND_DEBOUNCE_MS) return
		lastSentAtRef.current = now
		mutateRef.current({ path: { channel_id: channelId } })
	}
}

/**
 * User ids currently typing in a channel. Ephemeral: driven entirely by
 * `TYPING_START` events, each expiring after its own `ttl_ms` (the server's
 * 10s window) via a local timer — never a query cache entry, so nothing
 * outlives a reload or lingers past the TTL a stale timer would miss.
 */
export function useTypingUsers(channelId: string): ReadonlySet<string> {
	const [typingUserIds, setTypingUserIds] = useState<Set<string>>(new Set())
	const timersRef = useRef(new Map<string, ReturnType<typeof setTimeout>>())

	useGatewayEvent('TYPING_START', (event) => {
		if (event.data.channel_id !== channelId) return
		const userId = event.data.user_id

		setTypingUserIds((previous) => {
			if (previous.has(userId)) return previous
			const next = new Set(previous)
			next.add(userId)
			return next
		})

		const existing = timersRef.current.get(userId)
		if (existing) clearTimeout(existing)
		const timer = setTimeout(() => {
			timersRef.current.delete(userId)
			setTypingUserIds((previous) => {
				if (!previous.has(userId)) return previous
				const next = new Set(previous)
				next.delete(userId)
				return next
			})
		}, event.data.ttl_ms)
		timersRef.current.set(userId, timer)
	})

	// A typing user in the previous channel must not leak into the next one.
	// `channelId` drives the reset by identity alone — nothing in the body
	// reads its value, so it cannot be inferred from usage.
	// biome-ignore lint/correctness/useExhaustiveDependencies: channelId is a reset trigger, not read in the body
	useEffect(() => {
		for (const timer of timersRef.current.values()) clearTimeout(timer)
		timersRef.current.clear()
		setTypingUserIds(new Set())
	}, [channelId])

	useEffect(() => {
		const timers = timersRef.current
		return () => {
			for (const timer of timers.values()) clearTimeout(timer)
		}
	}, [])

	return typingUserIds
}

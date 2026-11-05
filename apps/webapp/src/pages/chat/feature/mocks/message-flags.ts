// MOCK: pinned and saved messages — no backend endpoint yet. Replace with channel pins and per-user saved messages endpoints.
import { useSyncExternalStore } from 'react'

interface MessageFlags {
	pinned: ReadonlySet<string>
	saved: ReadonlySet<string>
}

let flags: MessageFlags = { pinned: new Set(), saved: new Set() }
const listeners = new Set<() => void>()

function subscribe(listener: () => void) {
	listeners.add(listener)
	return () => {
		listeners.delete(listener)
	}
}

function snapshot() {
	return flags
}

function toggle(set: ReadonlySet<string>, id: string): ReadonlySet<string> {
	const next = new Set(set)
	if (next.has(id)) next.delete(id)
	else next.add(id)
	return next
}

function update(next: MessageFlags) {
	flags = next
	for (const listener of listeners) listener()
}

export function useMessageFlags() {
	const data = useSyncExternalStore(subscribe, snapshot, snapshot)

	return { data, isLoading: false }
}

export async function toggleMessagePin(messageId: string): Promise<void> {
	await new Promise((resolve) => setTimeout(resolve, 120))
	update({ ...flags, pinned: toggle(flags.pinned, messageId) })
}

export async function toggleMessageSaved(messageId: string): Promise<void> {
	await new Promise((resolve) => setTimeout(resolve, 120))
	update({ ...flags, saved: toggle(flags.saved, messageId) })
}

export function resetMessageFlags(): void {
	update({ pinned: new Set(), saved: new Set() })
}

// MOCK: credential description, status and usage — no backend field yet. Replace with `description`, `status` and `used_by_workflows` on the credential response.
import {
	type CredentialMeta,
	seededCredentialMeta,
} from '#/pages/automation/lib/credential-meta'

const STORAGE_KEY = 'mestier:automation-credential-meta:v1'

function readPersisted(): Record<string, Partial<CredentialMeta>> {
	try {
		const stored = localStorage.getItem(STORAGE_KEY)
		return stored
			? (JSON.parse(stored) as Record<string, Partial<CredentialMeta>>)
			: {}
	} catch {
		return {}
	}
}

const patches = new Map<string, Partial<CredentialMeta>>(
	Object.entries(readPersisted()),
)
const kinds = new Map<string, string>()
const listeners = new Set<() => void>()

export function credentialMetaOf(id: string): CredentialMeta {
	return {
		...seededCredentialMeta(id, kinds.get(id) ?? ''),
		...patches.get(id),
	}
}

const metaRecord = (): Readonly<Record<string, CredentialMeta>> =>
	new Proxy(
		{},
		{
			get: (_target, key) =>
				typeof key === 'string' ? credentialMetaOf(key) : undefined,
		},
	)

let snapshot = metaRecord()

function persist(): boolean {
	try {
		localStorage.setItem(
			STORAGE_KEY,
			JSON.stringify(Object.fromEntries(patches)),
		)
		return true
	} catch {
		return false
	}
}

function publish() {
	snapshot = metaRecord()
	persist()
	for (const listener of listeners) listener()
}

export function subscribeCredentialMeta(listener: () => void) {
	listeners.add(listener)
	return () => {
		listeners.delete(listener)
	}
}

export function readCredentialMeta(): Readonly<Record<string, CredentialMeta>> {
	return snapshot
}

export function rememberCredentialKinds(
	entries: readonly { id: string; kind: string }[],
) {
	let changed = false
	for (const entry of entries) {
		if (kinds.get(entry.id) === entry.kind) continue
		kinds.set(entry.id, entry.kind)
		changed = true
	}
	if (changed) publish()
}

export function patchCredentialMeta(
	id: string,
	patch: Partial<CredentialMeta>,
) {
	patches.set(id, { ...patches.get(id), ...patch })
	publish()
}

export function forgetCredentialMeta(id: string) {
	patches.delete(id)
	kinds.delete(id)
	publish()
}

import {
	type DocumentContent,
	LOCK_RENEW_MS,
	type LockGrant,
	type LockRefusal,
	SAVE_DELAY_MS,
	type SaveConflictReason,
	type SavedContent,
} from '#/pages/knowledge-documents/lib/document-api-types'

export const SAVE_RETRY_MS = 5_000

export type AcquireOutcome =
	| { kind: 'granted'; grant: LockGrant }
	| { kind: 'refused'; lock: LockRefusal }

export type SaveOutcome =
	| { kind: 'saved'; saved: SavedContent }
	| { kind: 'conflict'; reason: SaveConflictReason }
	| { kind: 'invalid'; message: string }

export interface RequestOptions {
	keepalive: boolean
}

export interface SessionApi {
	acquire: () => Promise<AcquireOutcome>
	release: (options?: RequestOptions) => Promise<void>
	save: (
		content: DocumentContent,
		revision: number,
		options?: RequestOptions,
	) => Promise<SaveOutcome>
}

export type SessionPhase =
	| 'viewing'
	| 'acquiring'
	| 'editing'
	| 'locked'
	| 'conflict'
	| 'failed'

export type SaveState = 'idle' | 'dirty' | 'saving' | 'saved' | 'error'

export type ConflictKind = SaveConflictReason | 'stale'

export interface SessionState {
	phase: SessionPhase
	saveState: SaveState
	holderId: string | null
	conflict: ConflictKind | null
	saveError: string | null
}

export interface DocumentSession {
	start: () => void
	change: (read: () => DocumentContent) => void
	flush: () => Promise<void>
	leave: () => Promise<void>
	pageHide: () => void
}

interface SessionOptions {
	api: SessionApi
	canEdit: boolean
	loadedRevision: number
	onState: (state: SessionState) => void
}

type ContentReader = () => DocumentContent

export function createDocumentSession({
	api,
	canEdit,
	loadedRevision,
	onState,
}: SessionOptions): DocumentSession {
	let state: SessionState = {
		phase: 'viewing',
		saveState: 'idle',
		holderId: null,
		conflict: null,
		saveError: null,
	}
	let revision = loadedRevision
	let pending: ContentReader | null = null
	let inflight: Promise<void> | null = null
	let saveTimer: ReturnType<typeof setTimeout> | null = null
	let renewTimer: ReturnType<typeof setInterval> | null = null
	let holding = false
	let left = false

	const update = (change: Partial<SessionState>) => {
		if (left) return
		state = { ...state, ...change }
		onState(state)
	}

	const clearSaveTimer = () => {
		if (saveTimer !== null) clearTimeout(saveTimer)
		saveTimer = null
	}

	const stopRenewing = () => {
		if (renewTimer !== null) clearInterval(renewTimer)
		renewTimer = null
	}

	const scheduleSave = (delay: number) => {
		clearSaveTimer()
		saveTimer = setTimeout(() => {
			saveTimer = null
			void runSave()
		}, delay)
	}

	const loseEditing = (conflict: ConflictKind) => {
		stopRenewing()
		clearSaveTimer()
		pending = null
		update({ phase: 'conflict', conflict, saveState: 'idle', saveError: null })
	}

	async function runSave(): Promise<void> {
		if (inflight) return
		if (state.phase !== 'editing' || pending === null) return
		const read = pending
		pending = null
		update({ saveState: 'saving', saveError: null })
		inflight = (async () => {
			let outcome: SaveOutcome
			try {
				outcome = await api.save(read(), revision)
			} catch {
				pending ??= read
				update({ saveState: 'error', saveError: 'Enregistrement impossible.' })
				scheduleSave(SAVE_RETRY_MS)
				return
			}
			if (outcome.kind === 'saved') {
				revision = outcome.saved.revision
				if (pending !== null) {
					update({ saveState: 'dirty' })
					scheduleSave(SAVE_DELAY_MS)
				} else {
					update({ saveState: 'saved' })
				}
			} else if (outcome.kind === 'conflict') {
				loseEditing(outcome.reason)
			} else {
				update({ saveState: 'error', saveError: outcome.message })
			}
		})()
		try {
			await inflight
		} finally {
			inflight = null
		}
	}

	async function renew(): Promise<void> {
		let outcome: AcquireOutcome
		try {
			outcome = await api.acquire()
		} catch {
			return
		}
		if (left || state.phase !== 'editing') return
		if (outcome.kind === 'refused') {
			loseEditing('lock_lost')
		} else if (outcome.grant.revision !== revision) {
			loseEditing('revision_conflict')
		}
	}

	async function acquire(): Promise<void> {
		let outcome: AcquireOutcome
		try {
			outcome = await api.acquire()
		} catch {
			update({ phase: 'failed' })
			return
		}
		if (outcome.kind === 'refused') {
			update({ phase: 'locked', holderId: outcome.lock.holder_id })
			return
		}
		holding = true
		if (left) {
			holding = false
			void api.release().catch(() => {})
			return
		}
		if (outcome.grant.revision !== loadedRevision) {
			holding = false
			void api.release().catch(() => {})
			update({ phase: 'conflict', conflict: 'stale' })
			return
		}
		revision = outcome.grant.revision
		update({ phase: 'editing', holderId: outcome.grant.holder_id })
		renewTimer = setInterval(() => {
			void renew()
		}, LOCK_RENEW_MS)
	}

	return {
		start() {
			if (!canEdit) {
				update({ phase: 'viewing' })
				return
			}
			update({ phase: 'acquiring' })
			void acquire()
		},

		change(read) {
			if (state.phase !== 'editing') return
			pending = read
			update({ saveState: 'dirty', saveError: null })
			scheduleSave(SAVE_DELAY_MS)
		},

		async flush() {
			clearSaveTimer()
			if (inflight) await inflight.catch(() => {})
			await runSave()
		},

		async leave() {
			const wasHolding = holding || state.phase === 'conflict'
			const editing = state.phase === 'editing'
			stopRenewing()
			clearSaveTimer()
			if (editing) {
				if (inflight) await inflight.catch(() => {})
				await runSave()
			}
			left = true
			holding = false
			if (wasHolding) await api.release().catch(() => {})
		},

		pageHide() {
			if (state.phase !== 'editing') return
			stopRenewing()
			clearSaveTimer()
			if (pending !== null) {
				const read = pending
				pending = null
				void api
					.save(read(), revision, { keepalive: true })
					.catch(() => undefined)
				return
			}
			void api.release({ keepalive: true }).catch(() => {})
		},
	}
}

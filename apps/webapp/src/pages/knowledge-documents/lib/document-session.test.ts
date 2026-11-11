import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { DocumentContent } from '#/pages/knowledge-documents/lib/document-api-types'
import {
	type AcquireOutcome,
	createDocumentSession,
	type SaveOutcome,
	type SessionApi,
	type SessionState,
} from '#/pages/knowledge-documents/lib/document-session'

const EMPTY: DocumentContent = { version: 1, blocks: [] }
const GRANT = {
	holder_id: 'me',
	expires_at: '2026-11-11T10:00:00Z',
	revision: 3,
}

function fakeApi(overrides: Partial<SessionApi> = {}): SessionApi {
	return {
		acquire: vi.fn(async () => ({ kind: 'granted' as const, grant: GRANT })),
		release: vi.fn(async () => {}),
		save: vi.fn(async (_content, revision) => ({
			kind: 'saved' as const,
			saved: { revision: revision + 1, updated_at: '2026-11-11T10:00:01Z' },
		})),
		...overrides,
	}
}

function setup(
	api: SessionApi,
	options: { canEdit?: boolean; loadedRevision?: number } = {},
) {
	const states: SessionState[] = []
	const session = createDocumentSession({
		api,
		canEdit: options.canEdit ?? true,
		loadedRevision: options.loadedRevision ?? 3,
		onState: (state) => states.push(state),
	})
	const last = () => states[states.length - 1] as SessionState
	return { session, states, last }
}

async function settle() {
	await vi.advanceTimersByTimeAsync(0)
}

beforeEach(() => {
	vi.useFakeTimers()
})

afterEach(() => {
	vi.useRealTimers()
})

describe('taking the lock', () => {
	it('opens editable when nobody else edits', async () => {
		const api = fakeApi()
		const { session, last } = setup(api)
		session.start()
		expect(last().phase).toBe('acquiring')
		await settle()
		expect(last().phase).toBe('editing')
		expect(api.acquire).toHaveBeenCalledTimes(1)
	})

	it('opens read-only with the holder when someone else edits', async () => {
		const api = fakeApi({
			acquire: vi.fn(async () => ({
				kind: 'refused' as const,
				lock: { holder_id: 'someone', expires_at: '2026-11-11T10:01:00Z' },
			})),
		})
		const { session, last } = setup(api)
		session.start()
		await settle()
		expect(last().phase).toBe('locked')
		expect(last().holderId).toBe('someone')
		await vi.advanceTimersByTimeAsync(120_000)
		expect(api.acquire).toHaveBeenCalledTimes(1)
	})

	it('never asks for the lock without the right to edit', async () => {
		const api = fakeApi()
		const { session, last } = setup(api, { canEdit: false })
		session.start()
		await settle()
		expect(last().phase).toBe('viewing')
		expect(api.acquire).not.toHaveBeenCalled()
		await session.leave()
		expect(api.release).not.toHaveBeenCalled()
	})

	it('goes to conflict when the document moved on between reading and locking', async () => {
		const api = fakeApi()
		const { session, last } = setup(api, { loadedRevision: 2 })
		session.start()
		await settle()
		expect(last().phase).toBe('conflict')
		expect(last().conflict).toBe('stale')
		expect(api.release).toHaveBeenCalledTimes(1)
	})

	it('reads only when the lock cannot be asked for at all', async () => {
		const api = fakeApi({
			acquire: vi.fn(async () => {
				throw new Error('offline')
			}),
		})
		const { session, last } = setup(api)
		session.start()
		await settle()
		expect(last().phase).toBe('failed')
	})
})

describe('renewing the lock', () => {
	it('renews every 30 seconds while the page is open', async () => {
		const api = fakeApi()
		const { session } = setup(api)
		session.start()
		await settle()
		expect(api.acquire).toHaveBeenCalledTimes(1)
		await vi.advanceTimersByTimeAsync(29_000)
		expect(api.acquire).toHaveBeenCalledTimes(1)
		await vi.advanceTimersByTimeAsync(1_000)
		expect(api.acquire).toHaveBeenCalledTimes(2)
		await vi.advanceTimersByTimeAsync(60_000)
		expect(api.acquire).toHaveBeenCalledTimes(4)
	})

	it('stops renewing once the page is left', async () => {
		const api = fakeApi()
		const { session } = setup(api)
		session.start()
		await settle()
		await session.leave()
		await vi.advanceTimersByTimeAsync(120_000)
		expect(api.acquire).toHaveBeenCalledTimes(1)
	})

	it('goes read-only when the lock was taken by someone else', async () => {
		let calls = 0
		const api = fakeApi({
			acquire: vi.fn(async () => {
				calls += 1
				return calls === 1
					? { kind: 'granted' as const, grant: GRANT }
					: {
							kind: 'refused' as const,
							lock: { holder_id: 'x', expires_at: '2026-11-11T10:05:00Z' },
						}
			}),
		})
		const { session, last } = setup(api)
		session.start()
		await settle()
		await vi.advanceTimersByTimeAsync(30_000)
		expect(last().phase).toBe('conflict')
		expect(last().conflict).toBe('lock_lost')
	})

	it('keeps editing through a failed renewal and tries again', async () => {
		let calls = 0
		const api = fakeApi({
			acquire: vi.fn(async () => {
				calls += 1
				if (calls === 2) throw new Error('offline')
				return { kind: 'granted' as const, grant: GRANT }
			}),
		})
		const { session, last } = setup(api)
		session.start()
		await settle()
		await vi.advanceTimersByTimeAsync(30_000)
		expect(last().phase).toBe('editing')
		await vi.advanceTimersByTimeAsync(30_000)
		expect(api.acquire).toHaveBeenCalledTimes(3)
		expect(last().phase).toBe('editing')
	})
})

describe('saving', () => {
	it('saves one second after the last change, with the current revision', async () => {
		const api = fakeApi()
		const { session, last } = setup(api)
		session.start()
		await settle()
		session.change(() => EMPTY)
		expect(last().saveState).toBe('dirty')
		await vi.advanceTimersByTimeAsync(999)
		expect(api.save).not.toHaveBeenCalled()
		await vi.advanceTimersByTimeAsync(1)
		expect(api.save).toHaveBeenCalledWith(EMPTY, 3)
		expect(last().saveState).toBe('saved')
	})

	it('waits for the typing to stop and saves once', async () => {
		const api = fakeApi()
		const { session } = setup(api)
		session.start()
		await settle()
		session.change(() => EMPTY)
		await vi.advanceTimersByTimeAsync(800)
		session.change(() => EMPTY)
		await vi.advanceTimersByTimeAsync(800)
		expect(api.save).not.toHaveBeenCalled()
		await vi.advanceTimersByTimeAsync(200)
		expect(api.save).toHaveBeenCalledTimes(1)
	})

	it('sends the next revision with the next save', async () => {
		const api = fakeApi()
		const { session } = setup(api)
		session.start()
		await settle()
		session.change(() => EMPTY)
		await vi.advanceTimersByTimeAsync(1000)
		session.change(() => EMPTY)
		await vi.advanceTimersByTimeAsync(1000)
		expect(vi.mocked(api.save).mock.calls.map((call) => call[1])).toEqual([
			3, 4,
		])
	})

	it('shows saving while the request is out', async () => {
		let release: () => void = () => {}
		const api = fakeApi({
			save: vi.fn(
				() =>
					new Promise<SaveOutcome>((resolve) => {
						release = () =>
							resolve({
								kind: 'saved',
								saved: { revision: 4, updated_at: '' },
							})
					}),
			),
		})
		const { session, last } = setup(api)
		session.start()
		await settle()
		session.change(() => EMPTY)
		await vi.advanceTimersByTimeAsync(1000)
		expect(last().saveState).toBe('saving')
		session.change(() => EMPTY)
		release()
		await settle()
		expect(last().saveState).toBe('dirty')
		await vi.advanceTimersByTimeAsync(1000)
		expect(api.save).toHaveBeenCalledTimes(2)
	})

	it('goes read-only with a conflict when the lock was lost', async () => {
		const api = fakeApi({
			save: vi.fn(async () => ({
				kind: 'conflict' as const,
				reason: 'lock_lost' as const,
			})),
		})
		const { session, last } = setup(api)
		session.start()
		await settle()
		session.change(() => EMPTY)
		await vi.advanceTimersByTimeAsync(1000)
		expect(last().phase).toBe('conflict')
		expect(last().conflict).toBe('lock_lost')
		await vi.advanceTimersByTimeAsync(120_000)
		expect(api.acquire).toHaveBeenCalledTimes(1)
		session.change(() => EMPTY)
		await vi.advanceTimersByTimeAsync(2000)
		expect(api.save).toHaveBeenCalledTimes(1)
	})

	it('goes read-only with a conflict when the revision is stale', async () => {
		const api = fakeApi({
			save: vi.fn(async () => ({
				kind: 'conflict' as const,
				reason: 'revision_conflict' as const,
			})),
		})
		const { session, last } = setup(api)
		session.start()
		await settle()
		session.change(() => EMPTY)
		await vi.advanceTimersByTimeAsync(1000)
		expect(last().conflict).toBe('revision_conflict')
	})

	it('reports a refused content and keeps editing', async () => {
		const api = fakeApi({
			save: vi.fn(async () => ({
				kind: 'invalid' as const,
				message: 'Trop de blocs.',
			})),
		})
		const { session, last } = setup(api)
		session.start()
		await settle()
		session.change(() => EMPTY)
		await vi.advanceTimersByTimeAsync(1000)
		expect(last().phase).toBe('editing')
		expect(last().saveState).toBe('error')
		expect(last().saveError).toBe('Trop de blocs.')
	})

	it('tries again after a network failure', async () => {
		let calls = 0
		const api = fakeApi({
			save: vi.fn(async (_content, revision) => {
				calls += 1
				if (calls === 1) throw new Error('offline')
				return {
					kind: 'saved' as const,
					saved: { revision: revision + 1, updated_at: '' },
				}
			}),
		})
		const { session, last } = setup(api)
		session.start()
		await settle()
		session.change(() => EMPTY)
		await vi.advanceTimersByTimeAsync(1000)
		expect(last().saveState).toBe('error')
		await vi.advanceTimersByTimeAsync(5000)
		expect(api.save).toHaveBeenCalledTimes(2)
		expect(last().saveState).toBe('saved')
	})

	it('ignores changes while read-only', async () => {
		const api = fakeApi({
			acquire: vi.fn(async () => ({
				kind: 'refused' as const,
				lock: { holder_id: 'x', expires_at: '' },
			})),
		})
		const { session } = setup(api)
		session.start()
		await settle()
		session.change(() => EMPTY)
		await vi.advanceTimersByTimeAsync(5000)
		expect(api.save).not.toHaveBeenCalled()
	})
})

describe('leaving', () => {
	it('saves what is pending and then gives the lock back', async () => {
		const order: string[] = []
		const api = fakeApi({
			save: vi.fn(async (_content, revision) => {
				order.push('save')
				return {
					kind: 'saved' as const,
					saved: { revision: revision + 1, updated_at: '' },
				}
			}),
			release: vi.fn(async () => {
				order.push('release')
			}),
		})
		const { session } = setup(api)
		session.start()
		await settle()
		session.change(() => EMPTY)
		await session.leave()
		expect(order).toEqual(['save', 'release'])
		await vi.advanceTimersByTimeAsync(5000)
		expect(api.save).toHaveBeenCalledTimes(1)
	})

	it('gives the lock back when the answer comes after leaving', async () => {
		let grant: () => void = () => {}
		const api = fakeApi({
			acquire: vi.fn(
				() =>
					new Promise<AcquireOutcome>((resolve) => {
						grant = () => resolve({ kind: 'granted', grant: GRANT })
					}),
			),
		})
		const { session } = setup(api)
		session.start()
		await session.leave()
		grant()
		await settle()
		expect(api.release).toHaveBeenCalledTimes(1)
		await vi.advanceTimersByTimeAsync(60_000)
		expect(api.acquire).toHaveBeenCalledTimes(1)
	})

	it('does not give back a lock it never held', async () => {
		const api = fakeApi({
			acquire: vi.fn(async () => ({
				kind: 'refused' as const,
				lock: { holder_id: 'x', expires_at: '' },
			})),
		})
		const { session } = setup(api)
		session.start()
		await settle()
		await session.leave()
		expect(api.release).not.toHaveBeenCalled()
	})

	it('releases with a keepalive request when the page is hidden and saved', async () => {
		const api = fakeApi()
		const { session } = setup(api)
		session.start()
		await settle()
		session.pageHide()
		expect(api.release).toHaveBeenCalledWith({ keepalive: true })
	})

	it('saves with a keepalive request when the page is hidden with changes pending', async () => {
		const api = fakeApi()
		const { session } = setup(api)
		session.start()
		await settle()
		session.change(() => EMPTY)
		session.pageHide()
		expect(api.save).toHaveBeenCalledWith(EMPTY, 3, { keepalive: true })
	})
})

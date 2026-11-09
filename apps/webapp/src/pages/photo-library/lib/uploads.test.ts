import { describe, expect, it } from 'vitest'
import {
	clearFinished,
	hasActive,
	newItem,
	patchItem,
	runPool,
	summarize,
	UPLOAD_CONCURRENCY,
	uploadErrorMessage,
} from '#/pages/photo-library/lib/uploads'

describe('runPool', () => {
	it('never runs more than the limit at once and handles every task', async () => {
		let running = 0
		let peak = 0
		const done: number[] = []
		await runPool([1, 2, 3, 4, 5, 6, 7], UPLOAD_CONCURRENCY, async (task) => {
			running += 1
			peak = Math.max(peak, running)
			await new Promise((resolve) => setTimeout(resolve, 5))
			running -= 1
			done.push(task)
		})
		expect(peak).toBe(3)
		expect(done.sort()).toEqual([1, 2, 3, 4, 5, 6, 7])
	})

	it('does nothing without tasks', async () => {
		await expect(runPool([], 3, async () => {})).resolves.toBeUndefined()
	})
})

describe('upload items', () => {
	const items = [
		newItem('1', 'a.jpg'),
		{ ...newItem('2', 'b.jpg'), status: 'done' as const },
		{ ...newItem('3', 'c.jpg'), status: 'duplicate' as const },
		{ ...newItem('4', 'd.pdf'), status: 'rejected' as const },
	]

	it('patches one item', () => {
		const next = patchItem(items, '1', { status: 'uploading' })
		expect(next[0]?.status).toBe('uploading')
		expect(next[1]).toBe(items[1])
	})

	it('knows when something is still running', () => {
		expect(hasActive(items)).toBe(true)
		expect(hasActive(clearFinished(items.slice(1)))).toBe(false)
	})

	it('keeps only active items when clearing', () => {
		expect(clearFinished(items).map((item) => item.id)).toEqual(['1'])
	})

	it('summarises the settled states', () => {
		expect(summarize(items)).toBe('1 envoyée, 1 en doublon, 1 refusée')
	})
})

describe('uploadErrorMessage', () => {
	it('maps the contract statuses to French messages', () => {
		expect(uploadErrorMessage(413, null)).toBe('Fichier trop volumineux.')
		expect(uploadErrorMessage(422, 'bad')).toMatch(/illisible/)
		expect(uploadErrorMessage(500, 'Oups')).toBe('Oups')
		expect(uploadErrorMessage(500, null)).toMatch(/500/)
	})
})

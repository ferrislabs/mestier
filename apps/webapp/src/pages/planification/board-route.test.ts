import { QueryClient } from '@tanstack/react-query'
import { createMemoryHistory, createRouter } from '@tanstack/react-router'
import { describe, expect, it } from 'vitest'
import { EMPTY_BOARD_FILTERS } from '#/pages/planification/lib/board-filters'
import { routeTree } from '#/routeTree.gen'

const BOARD_PATH = '/o/acme/planification/board'

function buildRouter(href: string) {
	return createRouter({
		routeTree,
		context: { queryClient: new QueryClient() },
		history: createMemoryHistory({ initialEntries: [href] }),
	})
}

/** The board route's validated search — what the feature is handed as props. */
async function boardSearch(href: string): Promise<Record<string, unknown>> {
	const router = buildRouter(href)
	await router.load()

	return (router.state.matches.at(-1)?.search ?? {}) as Record<string, unknown>
}

async function openBoard(href = BOARD_PATH) {
	const router = buildRouter(href)
	await router.load()

	async function go(search: Record<string, unknown>) {
		await router.navigate({
			to: '/o/$organizationSlug/planification/board',
			params: { organizationSlug: 'acme' },
			search,
		})
		await router.load()
	}

	async function step(direction: 'back' | 'forward') {
		if (direction === 'back') router.history.back()
		else router.history.forward()
		await new Promise((resolve) => setTimeout(resolve, 0))
		await router.load()
	}

	return {
		go,
		step,
		searchStr: () => router.state.location.searchStr,
	}
}

/**
 * The filter state of the board lives in the address and nowhere else
 * (#467), so these are the tests of that address: what a link restores, what
 * a rotten link does instead of throwing, and what the URL looks like once
 * every filter is off again.
 */
describe('board route — search params', () => {
	it('restores a filtered board from its link', async () => {
		expect(
			await boardSearch(
				`${BOARD_PATH}?project_id=project-1&assignee_id=member-1&label_id=label-1&q=toiture&unscheduled=true`,
			),
		).toEqual({
			project_id: 'project-1',
			assignee_id: 'member-1',
			label_id: 'label-1',
			q: 'toiture',
			unscheduled: true,
		})
	})

	it('opens the unfiltered board on a bare address', async () => {
		expect(await boardSearch(BOARD_PATH)).toEqual(EMPTY_BOARD_FILTERS)
	})

	/**
	 * `validateSearch` throwing is what turns a stale share link into a
	 * router error page. The schema catches instead, per field: the readable
	 * half of the URL survives the unreadable half.
	 */
	it('falls back on a malformed param and keeps the rest', async () => {
		expect(
			await boardSearch(
				`${BOARD_PATH}?project_id=project-1&unscheduled=peut-etre&q=`,
			),
		).toEqual({ ...EMPTY_BOARD_FILTERS, project_id: 'project-1' })
	})
})

describe('board route — navigation', () => {
	it('puts the filter in the address and takes it back out again', async () => {
		const board = await openBoard()
		expect(board.searchStr()).toBe('')

		await board.go({ project_id: 'project-1' })
		expect(board.searchStr()).toBe('?project_id=project-1')

		// Clearing navigates with every key explicitly undefined, which is
		// what leaves a clean address rather than `?project_id=&q=`.
		await board.go({ ...EMPTY_BOARD_FILTERS })
		expect(board.searchStr()).toBe('')
	})

	it('never writes the off state of the unscheduled toggle', async () => {
		const board = await openBoard()

		await board.go({ ...EMPTY_BOARD_FILTERS, unscheduled: true })
		expect(board.searchStr()).toBe('?unscheduled=true')

		await board.go({ ...EMPTY_BOARD_FILTERS, unscheduled: undefined })
		expect(board.searchStr()).toBe('')
	})

	it('moves back and forward through the filter states in order', async () => {
		const board = await openBoard()

		await board.go({ project_id: 'project-1' })
		await board.go({ project_id: 'project-2' })
		expect(board.searchStr()).toBe('?project_id=project-2')

		await board.step('back')
		expect(board.searchStr()).toBe('?project_id=project-1')

		await board.step('back')
		expect(board.searchStr()).toBe('')

		await board.step('forward')
		expect(board.searchStr()).toBe('?project_id=project-1')
	})
})

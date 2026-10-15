import { QueryClient } from '@tanstack/react-query'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { MaintenanceRecapLine } from '#/hooks/use-maintenance-recap'
import { downloadAuthenticatedFile } from '#/lib/download-file'
import { RecapFeature } from '#/pages/maintenance/feature/recap-feature'
import { wrapWithPermissions } from '#/test/with-permissions'

vi.mock('#/lib/download-file', () => ({
	downloadAuthenticatedFile: vi.fn(async () => undefined),
}))

const RECAP_PATH = '/api/v1/organizations/{organization_id}/maintenance/recap'
const PLAN_PATH =
	'/api/v1/organizations/{organization_id}/maintenance/recap/plan'

type Handler = (params: unknown) => unknown

function recapLine(
	id: string,
	label: string,
	overrides: Partial<MaintenanceRecapLine> = {},
): MaintenanceRecapLine {
	return {
		contract_id: 'contract-1',
		due: { on: '2026-10-19', status: 'THIS_WEEK' },
		label,
		last_done_on: '2026-10-05',
		line_id: id,
		minutes_per_intervention: 90,
		site: {
			customer_name: 'Gaume',
			site_label: 'Siège',
			city: 'Vichy',
		},
		...overrides,
	}
}

/**
 * A stand-in for `window.tanstackApi` whose recap is a piece of server
 * state: planning lines takes them out of it, as the real endpoint does,
 * and the screen has to refetch to notice.
 */
function installFakeApi(initial: MaintenanceRecapLine[]) {
	let lines = [...initial]
	const planned = vi.fn()
	let planHandler: Handler = (params) => {
		const { line_ids } = (params as { body: { line_ids: string[] } }).body
		lines = lines.filter((line) => !line_ids.includes(line.line_id))
		return { data: [], pagination: null }
	}

	function queryKeyFor(path: string, params: unknown) {
		const { path: pathParams, query } = (params ?? {}) as {
			path?: unknown
			query?: unknown
		}
		return [{ _id: path, path: pathParams, query }]
	}
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = {
		get(path: string, params: unknown) {
			const queryKey = queryKeyFor(path, params)
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path !== RECAP_PATH) throw new Error(`unmocked GET ${path}`)
						return {
							data: {
								year: 2026,
								week: 43,
								groups: lines.length ? [{ town: 'Vichy', lines }] : [],
							},
							pagination: null,
						}
					},
				},
			}
		},
		mutation(method: string, path: string) {
			return {
				mutationOptions: {
					mutationKey: [{ method, path }],
					mutationFn: async (params: unknown) => {
						if (method !== 'post' || path !== PLAN_PATH) {
							throw new Error(`unmocked ${method} ${path}`)
						}
						planned(params)
						return planHandler(params)
					},
				},
			}
		},
	}

	return {
		planned,
		failPlanWith(error: Error) {
			planHandler = () => {
				throw error
			}
		},
	}
}

function renderRecap(
	initial: MaintenanceRecapLine[],
	onWeekChange: (week: string) => void = () => {},
) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	const ui = wrapWithPermissions(
		<RecapFeature week="2026-W43" onWeekChange={onWeekChange} />,
		{ queryClient },
	)
	const api = installFakeApi(initial)
	render(ui)
	return api
}

beforeEach(() => {
	vi.mocked(downloadAuthenticatedFile).mockClear()
})

describe('RecapFeature', () => {
	it('lists the lines due, grouped by town', async () => {
		renderRecap([recapLine('l1', 'Tonte'), recapLine('l2', 'Taille de haie')])

		expect(await screen.findByText('Tonte')).toBeTruthy()
		expect(screen.getByText('Taille de haie')).toBeTruthy()
		expect(screen.getByRole('heading', { name: /vichy/i })).toBeTruthy()
	})

	it('planning two lines removes them from the list', async () => {
		const user = userEvent.setup()
		const api = renderRecap([
			recapLine('l1', 'Tonte'),
			recapLine('l2', 'Taille de haie'),
			recapLine('l3', 'Désherbage'),
		])
		await screen.findByText('Tonte')

		await user.click(screen.getByRole('checkbox', { name: /tonte/i }))
		await user.click(screen.getByRole('checkbox', { name: /taille de haie/i }))
		await user.click(screen.getByRole('button', { name: /planifier \(2\)/i }))

		await waitFor(() => expect(screen.queryByText('Tonte')).toBeNull())
		expect(screen.queryByText('Taille de haie')).toBeNull()
		expect(screen.getByText('Désherbage')).toBeTruthy()
		expect(api.planned).toHaveBeenCalledTimes(1)
		expect(api.planned).toHaveBeenCalledWith({
			path: { organization_id: 'org-1' },
			body: { line_ids: ['l1', 'l2'], week: '2026-W43' },
		})
	})

	it('offers nothing to plan until a line is ticked', async () => {
		renderRecap([recapLine('l1', 'Tonte')])
		await screen.findByText('Tonte')

		const plan = screen.getByRole('button', { name: /planifier/i })

		expect((plan as HTMLButtonElement).disabled).toBe(true)
	})

	it('ticks every line at once', async () => {
		const user = userEvent.setup()
		renderRecap([recapLine('l1', 'Tonte'), recapLine('l2', 'Taille de haie')])
		await screen.findByText('Tonte')

		await user.click(
			screen.getByRole('checkbox', { name: /tout sélectionner/i }),
		)

		expect(
			screen.getByRole('button', { name: /planifier \(2\)/i }),
		).toBeTruthy()
	})

	it('tells the manager when a line was planned by someone else', async () => {
		const user = userEvent.setup()
		const api = renderRecap([recapLine('l1', 'Tonte')])
		const conflict = Object.assign(
			new Error('a line is already planned: an open task carries it'),
			{ status: 409 },
		)
		api.failPlanWith(conflict)
		await screen.findByText('Tonte')

		await user.click(screen.getByRole('checkbox', { name: /tonte/i }))
		await user.click(screen.getByRole('button', { name: /planifier \(1\)/i }))

		const alert = await screen.findByRole('alert')
		expect(within(alert).getByText(/déjà planifiée/i)).toBeTruthy()
		expect(screen.getByText('Tonte')).toBeTruthy()
	})

	it('downloads the PDF of the week being read', async () => {
		const user = userEvent.setup()
		renderRecap([recapLine('l1', 'Tonte')])
		await screen.findByText('Tonte')

		await user.click(screen.getByRole('button', { name: /pdf/i }))

		await waitFor(() =>
			expect(downloadAuthenticatedFile).toHaveBeenCalledWith(
				'/api/v1/organizations/org-1/maintenance/recap.pdf?week=2026-W43',
				'recap-entretien-2026-W43.pdf',
			),
		)
	})

	it('moves to the neighbouring weeks', async () => {
		const user = userEvent.setup()
		const onWeekChange = vi.fn()
		renderRecap([recapLine('l1', 'Tonte')], onWeekChange)
		await screen.findByText('Tonte')

		await user.click(screen.getByRole('button', { name: /semaine suivante/i }))
		await user.click(
			screen.getByRole('button', { name: /semaine précédente/i }),
		)

		expect(onWeekChange).toHaveBeenNthCalledWith(1, '2026-W44')
		expect(onWeekChange).toHaveBeenNthCalledWith(2, '2026-W42')
	})

	it('says so when nothing is due', async () => {
		renderRecap([])

		expect(await screen.findByText(/rien à planifier/i)).toBeTruthy()
	})
})

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { ActiveOrganizationProvider } from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import {
	PlanningTeamFeature,
	type PlanningTeamFeatureProps,
} from '#/pages/planning/feature/planning-team-feature'

const PLANNING_PATH = '/api/v1/organizations/{organization_id}/planning'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	owner_id: 'user-1',
	slug: 'atelier-bois',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const PLANNING_RESPONSE = {
	timezone: 'Europe/Paris',
	resources: [
		{
			resource_id: 'employee:employee-1',
			kind: 'employee',
			employee_id: 'employee-1',
			user_id: null,
			display_name: 'Alix Martin',
			hourly_rate_cents: 1500,
			weekly_contract_minutes: 2100,
		},
	],
	entries: [],
	work_time: [],
}

function installFakeTanstackApi() {
	const calls: { method: string; path: string; params: unknown }[] = []

	function queryKeyFor(path: string, params: unknown) {
		const p = (params ?? {}) as { path?: unknown; query?: unknown }
		return [{ _id: path, path: p.path, query: p.query }]
	}

	const fakeApi = {
		get(path: string, params: unknown) {
			const queryKey = queryKeyFor(path, params)
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls.push({ method: 'get', path, params })
						if (path === PLANNING_PATH) {
							return { data: PLANNING_RESPONSE, pagination: null }
						}
						throw new Error(`unmocked GET ${path}`)
					},
				},
			}
		},
		mutation() {
			throw new Error('no mutation expected in W6 — planning is read-only')
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi

	return calls
}

function renderFeature(overrides: Partial<PlanningTeamFeatureProps> = {}) {
	const calls = installFakeTanstackApi()
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>
				<ActiveOrganizationProvider organizations={[ORGANIZATION]}>
					{children}
				</ActiveOrganizationProvider>
			</QueryClientProvider>
		)
	}

	const onViewChange = vi.fn()
	const onDateChange = vi.fn()

	render(
		<Providers>
			<PlanningTeamFeature
				view="week"
				date="2026-08-07"
				onViewChange={onViewChange}
				onDateChange={onDateChange}
				{...overrides}
			/>
		</Providers>,
	)

	return { calls, onViewChange, onDateChange }
}

describe('PlanningTeamFeature', () => {
	it('charge le planning avec la fenêtre from/to dérivée de la vue et de la date', async () => {
		const { calls } = renderFeature()

		expect(await screen.findByText('Alix Martin')).toBeDefined()
		const call = calls.find((c) => c.path === PLANNING_PATH)
		expect(call?.params).toMatchObject({
			path: { organization_id: 'org-1' },
			query: { from: '2026-08-03', to: '2026-08-09' },
		})
	})

	it('recalcule la fenêtre pour la vue mois', async () => {
		const { calls } = renderFeature({ view: 'month' })

		await screen.findByText('Alix Martin')
		const call = calls.find((c) => c.path === PLANNING_PATH)
		expect(call?.params).toMatchObject({
			query: { from: '2026-08-01', to: '2026-08-31' },
		})
	})

	it('reporte le changement de vue au parent plutôt que de le gérer lui-même', async () => {
		const user = userEvent.setup()
		const { onViewChange } = renderFeature()

		await screen.findByText('Alix Martin')
		await user.click(screen.getByRole('tab', { name: 'Jour' }))

		expect(onViewChange).toHaveBeenCalledWith('day')
	})

	it('reporte le changement de date au parent', async () => {
		const user = userEvent.setup()
		const { onDateChange } = renderFeature()

		await screen.findByText('Alix Martin')
		await user.click(screen.getByRole('button', { name: 'Période suivante' }))

		expect(onDateChange).toHaveBeenCalledWith('2026-08-14')
	})
})

import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import {
	DEFAULT_FILTERS,
	type ExecutionRow,
} from '#/pages/automation/lib/execution-log'
import {
	AutomationExecutionsLog,
	type AutomationExecutionsLogProps,
} from '#/pages/automation/ui/automation-executions-log'
import { renderWithRouter } from '#/test/render-with-router'

function row(overrides: Partial<ExecutionRow> = {}): ExecutionRow {
	return {
		key: 'run-1',
		runId: 'run-1',
		shortId: 'abcdef12',
		workflowId: 'wf-1',
		workflowName: 'Relance des devis',
		startedLabel: 'Aujourd’hui à 09:00:00',
		startedMs: 0,
		trigger: 'Manuel',
		status: 'error',
		durationMs: 2000,
		failure: 'SMTP expiré',
		version: 'Version 12345678',
		...overrides,
	}
}

function render(overrides: Partial<AutomationExecutionsLogProps> = {}) {
	const props: AutomationExecutionsLogProps = {
		organizationSlug: 'atelier-bois',
		rows: [row()],
		summary: {
			total: 4,
			failed: 1,
			running: 1,
			successRate: 67,
			medianDurationMs: 3000,
		},
		filters: DEFAULT_FILTERS,
		counts: { all: 4, running: 1, success: 2, error: 1 },
		workflows: [{ id: 'wf-1', name: 'Relance des devis' }],
		triggers: ['Manuel'],
		canReset: false,
		isLoading: false,
		error: null,
		sortKey: 'started',
		direction: 'desc',
		onFiltersChange: vi.fn(),
		onSort: vi.fn(),
		onOpen: vi.fn(),
		onReset: vi.fn(),
		...overrides,
	}
	return {
		props,
		rendered: renderWithRouter(<AutomationExecutionsLog {...props} />),
	}
}

describe('AutomationExecutionsLog', () => {
	it('summarizes the runs', async () => {
		await render().rendered

		expect(screen.getByText('67 %')).toBeDefined()
		expect(screen.getByText('dont 1 en cours')).toBeDefined()
		expect(screen.getByText('3 s')).toBeDefined()
	})

	it('links each run to its detail and opens it on click', async () => {
		const user = userEvent.setup()
		const { props, rendered } = render()
		await rendered

		const link = screen.getByRole('link', {
			name: 'Ouvrir l’exécution abcdef12 de Relance des devis',
		})
		expect(link.getAttribute('href')).toBe(
			'/o/atelier-bois/automatisation/wf-1/executions/run-1',
		)

		await user.click(screen.getByText('SMTP expiré'))
		expect(props.onOpen).toHaveBeenCalledWith(row())
	})

	it('says there is nothing yet, or nothing for these filters, and offers a reset', async () => {
		const user = userEvent.setup()
		const { props, rendered } = render({ rows: [], canReset: true })
		await rendered

		expect(
			screen.getByText('Aucune exécution ne correspond à ces filtres.'),
		).toBeDefined()
		await user.click(
			screen.getAllByRole('button', {
				name: /Réinitialiser/,
			})[0] as HTMLElement,
		)
		expect(props.onReset).toHaveBeenCalled()
	})

	it('shows the empty state of a fresh organization', async () => {
		await render({ rows: [] }).rendered

		expect(
			screen.getByText('Les exécutions de vos workflows apparaîtront ici.'),
		).toBeDefined()
	})

	it('filters by status through the toggle group', async () => {
		const user = userEvent.setup()
		const { props, rendered } = render()
		await rendered

		await user.click(screen.getByRole('radio', { name: /Échecs/ }))

		expect(props.onFiltersChange).toHaveBeenCalledWith({ status: 'error' })
	})

	it('sorts when a sortable header is clicked', async () => {
		const user = userEvent.setup()
		const { props, rendered } = render()
		await rendered

		await user.click(screen.getByRole('button', { name: 'Durée' }))

		expect(props.onSort).toHaveBeenCalledWith('duration')
	})

	it('surfaces a load error', async () => {
		await render({ error: 'boom' }).rendered

		expect(screen.getByText('boom')).toBeDefined()
	})
})

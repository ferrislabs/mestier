import { screen, within } from '@testing-library/react'
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
		number: 1285,
		workflowId: 'wf-1',
		workflowName: 'Relance des devis',
		startedLabel: 'Aujourd’hui à 09:00:00',
		startedMs: 0,
		trigger: 'Test manuel',
		stepsDone: 2,
		stepsTotal: 5,
		status: 'error',
		durationMs: 2000,
		resumeNote: null,
		failure: { nodeLabel: 'Envoyer l’e-mail', message: 'SMTP expiré' },
		version: 'Version du 31 oct. 2026 à 18:20',
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
		triggers: ['Test manuel'],
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
			name: 'Ouvrir l’exécution 1285 de Relance des devis',
		})
		expect(link.textContent).toBe('#1285')
		expect(link.getAttribute('href')).toBe(
			'/o/atelier-bois/automatisation/workflows/wf-1?vue=executions&execution=1285',
		)

		await user.click(screen.getByText(': SMTP expiré'))
		expect(props.onOpen).toHaveBeenCalledWith(row())
	})

	it('names the failed node in red, then the message in grey', async () => {
		await render().rendered

		const node = screen.getByText('Envoyer l’e-mail')
		expect(node.className).toContain('text-state-alert')
		const message = screen.getByText(': SMTP expiré')
		expect(message.className).toContain('text-muted-foreground')
		expect(node.parentElement?.getAttribute('title')).toBe(
			'Envoyer l’e-mail : SMTP expiré',
		)
	})

	it('shows the message alone, in red, while the failed node is not known', async () => {
		await render({
			rows: [row({ failure: { nodeLabel: null, message: 'SMTP expiré' } })],
		}).rendered

		const message = screen.getByText('SMTP expiré')
		expect(message.className).toContain('text-state-alert')
		expect(message.getAttribute('title')).toBe('SMTP expiré')
	})

	it('writes the node counts and the version under the workflow', async () => {
		await render().rendered

		expect(screen.getByText('2 / 5')).toBeDefined()
		expect(screen.getByText('Version du 31 oct. 2026 à 18:20')).toBeDefined()
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

	it('shows the elapsed time of a running run where a finished run shows its duration', async () => {
		await render({
			rows: [row({ status: 'running', durationMs: 2200, failure: null })],
		}).rendered

		const table = within(screen.getByRole('table'))
		expect(table.getByText('En cours')).toBeDefined()
		expect(table.getByText('2,2 s')).toBeDefined()
	})

	it('shows a dash for a run that has no duration yet', async () => {
		await render({
			rows: [row({ status: 'waiting', durationMs: null, failure: null })],
		}).rendered

		const table = within(screen.getByRole('table'))
		expect(table.getByText('En attente')).toBeDefined()
		expect(table.getAllByText('—').length).toBeGreaterThan(0)
	})

	it('writes when a paused run resumes in the place of the failure, with no duration', async () => {
		await render({
			rows: [
				row({
					status: 'paused',
					durationMs: null,
					failure: null,
					resumeNote: 'Reprise le 8 nov. 2026 à 09:30',
				}),
			],
		}).rendered

		const table = within(screen.getByRole('table'))
		expect(table.getByText('En pause')).toBeDefined()
		expect(table.getByText('Reprise le 8 nov. 2026 à 09:30')).toBeDefined()
		expect(table.getAllByText('—')).toHaveLength(1)
	})

	it('surfaces a load error', async () => {
		await render({ error: 'boom' }).rendered

		expect(screen.getByText('boom')).toBeDefined()
	})
})

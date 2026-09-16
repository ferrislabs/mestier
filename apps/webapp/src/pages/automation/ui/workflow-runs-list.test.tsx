import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type {
	RunListRow,
	WorkflowRunsListProps,
} from '#/pages/automation/ui/workflow-runs-list'
import { WorkflowRunsList } from '#/pages/automation/ui/workflow-runs-list'

function row(overrides: Partial<RunListRow> = {}): RunListRow {
	return {
		id: 'run-1',
		status: 'succeeded',
		trigger: 'Manuel',
		startedAt: '2026-08-05T10:00:00Z',
		finishedAt: '2026-08-05T10:00:01Z',
		duration: '1.5s',
		...overrides,
	}
}

function baseProps(
	overrides: Partial<WorkflowRunsListProps> = {},
): WorkflowRunsListProps {
	return {
		workflowName: 'Créer une facture Odoo',
		rows: [],
		isLoading: false,
		error: null,
		onOpenRun: vi.fn(),
		onBackToEditor: vi.fn(),
		...overrides,
	}
}

describe('WorkflowRunsList — loading and empty states', () => {
	it('shows a loading indicator', () => {
		render(<WorkflowRunsList {...baseProps({ isLoading: true })} />)

		expect(screen.getByText('Chargement des exécutions…')).toBeDefined()
	})

	it('says there has never been a run', () => {
		render(<WorkflowRunsList {...baseProps()} />)

		expect(screen.getByText('Aucune exécution')).toBeDefined()
	})

	it('surfaces a load error', () => {
		render(<WorkflowRunsList {...baseProps({ error: 'HTTP 500' })} />)

		expect(screen.getByText('HTTP 500')).toBeDefined()
	})
})

describe('WorkflowRunsList — the table', () => {
	it('shows status, trigger, timings and duration for each run', () => {
		render(
			<WorkflowRunsList
				{...baseProps({
					rows: [
						row({
							status: 'failed',
							trigger: 'Événement',
							startedAt: '2026-08-05T10:00:00Z',
							finishedAt: '2026-08-05T10:00:02Z',
							duration: '2s',
						}),
					],
				})}
			/>,
		)

		expect(screen.getByText('Échoué')).toBeDefined()
		expect(screen.getByText('Événement')).toBeDefined()
		expect(screen.getByText('2026-08-05T10:00:00Z')).toBeDefined()
		expect(screen.getByText('2026-08-05T10:00:02Z')).toBeDefined()
		expect(screen.getByText('2s')).toBeDefined()
	})

	it('opens a run on request', async () => {
		const user = userEvent.setup()
		const onOpenRun = vi.fn()
		const target = row({ id: 'run-42' })
		render(<WorkflowRunsList {...baseProps({ rows: [target], onOpenRun })} />)

		await user.click(screen.getByRole('button', { name: 'Détails' }))

		expect(onOpenRun).toHaveBeenCalledWith(target)
	})

	it('navigates back to the editor', async () => {
		const user = userEvent.setup()
		const onBackToEditor = vi.fn()
		render(<WorkflowRunsList {...baseProps({ onBackToEditor })} />)

		await user.click(screen.getByRole('button', { name: /Retour à l’éditeur/ }))

		expect(onBackToEditor).toHaveBeenCalledTimes(1)
	})
})

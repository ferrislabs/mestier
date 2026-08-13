import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import {
	type RunRow,
	WorkflowRunsUI,
} from '#/pages/automation/ui/workflow-runs-ui'
import { renderWithRouter } from '#/test/render-with-router'

function run(overrides: Partial<RunRow> = {}): RunRow {
	return {
		id: 'run-1',
		status: 'succeeded',
		startedAt: '2026-08-01T10:00:00Z',
		finishedAt: '2026-08-01T10:00:05Z',
		error: null,
		...overrides,
	}
}

function baseProps() {
	return {
		organizationName: 'Atelier Bois & Co',
		organizationSlug: 'atelier-bois',
		workflowId: 'workflow-1',
		workflowName: 'Relance devis' as string | null,
		isLoading: false,
		error: null,
		runs: [run()],
	}
}

describe('WorkflowRunsUI — header', () => {
	it('names the workflow in the title once it is known', async () => {
		await renderWithRouter(<WorkflowRunsUI {...baseProps()} />)

		expect(screen.getByText('Runs — Relance devis')).toBeDefined()
	})

	it('falls back to a generic title while the workflow is not resolved yet', async () => {
		await renderWithRouter(
			<WorkflowRunsUI {...baseProps()} workflowName={null} />,
		)

		expect(screen.getByText('Runs')).toBeDefined()
	})
})

describe('WorkflowRunsUI — empty state', () => {
	it('shows a placeholder when the workflow has never run', async () => {
		await renderWithRouter(<WorkflowRunsUI {...baseProps()} runs={[]} />)

		expect(screen.getByText('Aucune exécution')).toBeDefined()
	})
})

describe('WorkflowRunsUI — rows', () => {
	it("links each run's details to the run inspector route", async () => {
		await renderWithRouter(<WorkflowRunsUI {...baseProps()} />)

		const link = screen.getByRole('link', { name: /Détails/ })
		expect(link.getAttribute('href')).toBe(
			'/o/atelier-bois/automation/workflow-1/runs/run-1',
		)
	})

	it('shows the last error for a failed run', async () => {
		await renderWithRouter(
			<WorkflowRunsUI
				{...baseProps()}
				runs={[run({ status: 'failed', error: 'Connector timed out' })]}
			/>,
		)

		expect(screen.getByText('Échoué')).toBeDefined()
		expect(screen.getByText('Connector timed out')).toBeDefined()
	})
})

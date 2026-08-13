import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import {
	RunInspectorUI,
	type RunSummary,
	type StepBlock,
	type StepRow,
} from '#/pages/automation/ui/run-inspector-ui'
import { renderWithRouter } from '#/test/render-with-router'

function step(overrides: Partial<StepRow> = {}): StepRow {
	return {
		id: 'step-1',
		connectorId: 'send-email',
		status: 'succeeded',
		attempts: 1,
		error: null,
		input: { to: 'client@example.com' },
		output: { messageId: 'abc' },
		startedAt: '2026-08-01T10:00:00Z',
		finishedAt: '2026-08-01T10:00:01Z',
		...overrides,
	}
}

function baseRun(overrides: Partial<RunSummary> = {}): RunSummary {
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
		isLoading: false,
		error: null,
		run: baseRun(),
		blocks: [{ kind: 'step', step: step() }] as StepBlock[],
		canReplayRun: false,
		replayingConnectorId: null as string | null,
		onReplay: vi.fn(),
	}
}

describe('RunInspectorUI — run summary', () => {
	it('shows the run status and its error when present', async () => {
		await renderWithRouter(
			<RunInspectorUI
				{...baseProps()}
				run={baseRun({ status: 'failed', error: 'Timed out' })}
			/>,
		)

		expect(screen.getByText('Échoué')).toBeDefined()
		expect(screen.getByText('Timed out')).toBeDefined()
	})
})

describe('RunInspectorUI — steps in graph order', () => {
	it('renders ungrouped steps in the order given, without resorting', async () => {
		const blocks: StepBlock[] = [
			{ kind: 'step', step: step({ id: 's1', connectorId: 'first' }) },
			{ kind: 'step', step: step({ id: 's2', connectorId: 'second' }) },
		]
		await renderWithRouter(<RunInspectorUI {...baseProps()} blocks={blocks} />)

		const rendered = screen.getAllByText(/first|second/)
		expect(rendered.map((el) => el.textContent)).toEqual(['first', 'second'])
	})

	it('renders input and output as formatted JSON', async () => {
		await renderWithRouter(<RunInspectorUI {...baseProps()} />)

		expect(screen.getByText(/"to": "client@example.com"/)).toBeDefined()
		expect(screen.getByText(/"messageId": "abc"/)).toBeDefined()
	})
})

describe('RunInspectorUI — loop iterations grouped by iteration_path', () => {
	it('renders a collapsible header per iteration, distinct from top-level steps', async () => {
		const blocks: StepBlock[] = [
			{
				kind: 'iteration',
				path: 'loop1[0]',
				label: 'Itération 0',
				steps: [step({ id: 's1', connectorId: 'in-loop' })],
			},
			{ kind: 'step', step: step({ id: 's2', connectorId: 'top-level' }) },
		]
		await renderWithRouter(<RunInspectorUI {...baseProps()} blocks={blocks} />)

		expect(screen.getByText('Itération 0')).toBeDefined()
		expect(screen.getByText('in-loop')).toBeDefined()
		expect(screen.getByText('top-level')).toBeDefined()
	})
})

describe('RunInspectorUI — replay from a step', () => {
	it('hides the replay action when the run cannot be replayed', async () => {
		await renderWithRouter(
			<RunInspectorUI {...baseProps()} canReplayRun={false} />,
		)

		expect(
			screen.queryByRole('button', { name: /Relancer depuis ici/ }),
		).toBeNull()
	})

	it('calls onReplay with the step when clicked', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(<RunInspectorUI {...props} canReplayRun={true} />)

		await user.click(
			screen.getByRole('button', { name: /Relancer depuis ici/ }),
		)

		expect(props.onReplay).toHaveBeenCalledWith(step())
	})
})

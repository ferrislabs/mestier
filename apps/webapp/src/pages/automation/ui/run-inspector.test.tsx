import { render, screen } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { installFlowTestEnvironment } from '#/pages/automation/test/flow-test-env'
import type { RunInspectorProps } from '#/pages/automation/ui/run-inspector'
import { RunInspector } from '#/pages/automation/ui/run-inspector'
import { renderWithPermissions } from '#/test/with-permissions'

beforeEach(() => {
	installFlowTestEnvironment()
})

function run(
	overrides: Partial<Schemas.RunResponse> = {},
): Schemas.RunResponse {
	return {
		id: 'run-1',
		organization_id: 'org-1',
		workflow_id: 'workflow-1',
		workflow_version_id: 'v1',
		status: 'succeeded',
		created_at: '2026-08-01T00:00:00Z',
		started_at: '2026-08-01T00:00:00.000Z',
		finished_at: '2026-08-01T00:00:01.500Z',
		trigger_event_id: null,
		...overrides,
	}
}

function descriptor(
	kind: string,
	label: string,
): Schemas.ConnectorDescriptorResponse {
	return {
		auth: 'None',
		branches: [],
		family: 'test',
		fields: [],
		kind,
		label,
		output_example: null,
		version: 1,
	}
}

function baseProps(
	overrides: Partial<RunInspectorProps> = {},
): RunInspectorProps {
	return {
		run: run(),
		graph: { connectors: [], edges: [], triggers: [] },
		layout: new Map(),
		descriptors: new Map(),
		connectorStatuses: new Map(),
		stepTree: [],
		onReplay: vi.fn(),
		isReplaying: false,
		replayingConnectorId: null,
		replayError: null,
		...overrides,
	}
}

describe('RunInspector — the header', () => {
	it('shows the run status, trigger and duration', () => {
		render(<RunInspector {...baseProps()} />)

		expect(screen.getByText('Réussi')).toBeDefined()
		expect(screen.getByText('Manuel')).toBeDefined()
		expect(screen.getByText('Durée : 1.5s')).toBeDefined()
	})

	it('names an event-triggered run', () => {
		render(
			<RunInspector
				{...baseProps({ run: run({ trigger_event_id: 'evt-1' }) })}
			/>,
		)

		expect(screen.getByText('Événement')).toBeDefined()
	})

	it('surfaces the run-level error', () => {
		render(
			<RunInspector
				{...baseProps({
					run: run({ status: 'failed', error: 'Odoo indisponible' }),
				})}
			/>,
		)

		expect(screen.getByText('Odoo indisponible')).toBeDefined()
	})
})

describe('RunInspector — the graph', () => {
	it('draws the pinned graph when there is one', async () => {
		render(
			<RunInspector
				{...baseProps({
					graph: {
						connectors: [
							{ id: 'c1', kind: 'test.simple', version: 1, config: {} },
						],
						edges: [],
						triggers: [],
					},
					layout: new Map([['c1', { x: 0, y: 0 }]]),
					descriptors: new Map([
						['test.simple', descriptor('test.simple', 'Étape simple')],
					]),
				})}
			/>,
		)

		expect(await screen.findByText('Étape simple')).toBeDefined()
	})

	it('shows that the graph is gone, without drawing anything else in its place', () => {
		render(<RunInspector {...baseProps({ graph: null })} />)

		expect(screen.getByText(/graphe n’est plus disponible/)).toBeDefined()
	})
})

describe('RunInspector — the step list', () => {
	it('renders the given step tree', () => {
		renderWithPermissions(
			<RunInspector
				{...baseProps({
					stepTree: [
						{
							kind: 'step',
							step: {
								id: 's1',
								connector_id: 'c1',
								iteration_path: '',
								attempts: 1,
								status: 'succeeded',
								created_at: '2026-08-01T00:00:00Z',
							},
						},
					],
				})}
			/>,
		)

		expect(screen.getByText('c1')).toBeDefined()
	})
})

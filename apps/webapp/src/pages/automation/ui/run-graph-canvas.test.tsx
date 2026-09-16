import { render, screen, within } from '@testing-library/react'
import { beforeEach, describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { installFlowTestEnvironment } from '#/pages/automation/test/flow-test-env'
import { RunGraphCanvas } from '#/pages/automation/ui/run-graph-canvas'

beforeEach(() => {
	installFlowTestEnvironment()
})

function connector(id: string, kind: string): Schemas.PlacedConnectorDto {
	return { id, kind, version: 1, config: {} }
}

function descriptor(
	kind: string,
	label: string,
	branches: Schemas.BranchDto[] = [],
): Schemas.ConnectorDescriptorResponse {
	return {
		auth: 'None',
		branches,
		family: 'test',
		fields: [],
		kind,
		label,
		output_example: null,
		version: 1,
	}
}

describe('RunGraphCanvas — drawing the pinned graph', () => {
	it('renders each connector under its catalogue label', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', 'test.simple')],
			edges: [],
		}
		render(
			<RunGraphCanvas
				graph={graph}
				layout={new Map([['c1', { x: 0, y: 0 }]])}
				descriptors={
					new Map([['test.simple', descriptor('test.simple', 'Étape simple')]])
				}
				connectorStatuses={new Map()}
			/>,
		)

		expect(await screen.findByText('Étape simple')).toBeDefined()
	})

	it('badges a connector with no step yet as not executed', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', 'test.simple')],
			edges: [],
		}
		render(
			<RunGraphCanvas
				graph={graph}
				layout={new Map([['c1', { x: 0, y: 0 }]])}
				descriptors={
					new Map([['test.simple', descriptor('test.simple', 'Étape simple')]])
				}
				connectorStatuses={new Map()}
			/>,
		)

		const node = await screen.findByTestId('rf__node-c1')
		expect(within(node).getByText('Non exécuté')).toBeDefined()
	})

	it('badges a connector with its aggregate run status', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', 'test.simple')],
			edges: [],
		}
		render(
			<RunGraphCanvas
				graph={graph}
				layout={new Map([['c1', { x: 0, y: 0 }]])}
				descriptors={
					new Map([['test.simple', descriptor('test.simple', 'Étape simple')]])
				}
				connectorStatuses={new Map([['c1', 'failed']])}
			/>,
		)

		const node = await screen.findByTestId('rf__node-c1')
		expect(within(node).getByText('Échoué')).toBeDefined()
	})

	it('draws a branch label for each branch the descriptor names', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', 'test.branching')],
			edges: [],
		}
		render(
			<RunGraphCanvas
				graph={graph}
				layout={new Map([['c1', { x: 0, y: 0 }]])}
				descriptors={
					new Map([
						[
							'test.branching',
							descriptor('test.branching', 'Condition', ['Then', 'Else']),
						],
					])
				}
				connectorStatuses={new Map()}
			/>,
		)

		const node = await screen.findByTestId('rf__node-c1')
		expect(within(node).getByText('Then')).toBeDefined()
		expect(within(node).getByText('Else')).toBeDefined()
	})

	it('renders nothing for a connector kind missing from the catalogue, falling back to its id', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', 'unknown.kind')],
			edges: [],
		}
		render(
			<RunGraphCanvas
				graph={graph}
				layout={new Map([['c1', { x: 0, y: 0 }]])}
				descriptors={new Map()}
				connectorStatuses={new Map()}
			/>,
		)

		expect(await screen.findByText('c1')).toBeDefined()
	})
})

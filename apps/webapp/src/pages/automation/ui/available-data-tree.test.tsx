import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { DataTreeNode } from '#/pages/automation/lib/data-tree'
import { AvailableDataTree } from '#/pages/automation/ui/available-data-tree'

const TRIGGER_BRANCH: DataTreeNode = {
	kind: 'branch',
	key: 'trigger',
	label: 'trigger',
	path: 'trigger',
	children: [
		{
			kind: 'leaf',
			key: 'id',
			label: 'id',
			path: 'trigger.id',
			value: 'q1',
		},
	],
}

const CONNECTOR_BRANCH: DataTreeNode = {
	kind: 'branch',
	key: 'connectors.c1.output',
	label: 'c1 · Créer un client',
	path: 'connectors.c1.output',
	children: [
		{
			kind: 'leaf',
			key: 'id',
			label: 'id',
			path: 'connectors.c1.output.id',
			value: 42,
		},
	],
}

function renderTree(
	overrides: Partial<Parameters<typeof AvailableDataTree>[0]> = {},
) {
	const onInsert = vi.fn()
	const onSourceChange = vi.fn()
	render(
		<AvailableDataTree
			branches={[TRIGGER_BRANCH, CONNECTOR_BRANCH]}
			source="example"
			hasLastRun={false}
			onSourceChange={onSourceChange}
			onInsert={onInsert}
			{...overrides}
		/>,
	)
	return { onInsert, onSourceChange }
}

describe('AvailableDataTree — structure', () => {
	it('starts collapsed and reveals leaves once a branch is expanded', async () => {
		const user = userEvent.setup()
		renderTree()

		expect(screen.queryByText('id')).toBeNull()

		await user.click(screen.getByRole('button', { name: /trigger/ }))

		expect(screen.getByText('id')).toBeDefined()
	})

	it('shows every branch it is given, upstream connectors included', () => {
		renderTree()

		expect(screen.getByRole('button', { name: /trigger/ })).toBeDefined()
		expect(
			screen.getByRole('button', { name: /c1 · Créer un client/ }),
		).toBeDefined()
	})

	it('says so when there is nothing to show', () => {
		renderTree({ branches: [] })

		expect(screen.getByText(/Aucune donnée disponible/)).toBeDefined()
	})
})

describe('AvailableDataTree — insertion', () => {
	it('inserts the leaf path, not its label, when clicked', async () => {
		const user = userEvent.setup()
		const { onInsert } = renderTree()

		await user.click(screen.getByRole('button', { name: /trigger/ }))
		await user.click(
			screen.getByRole('button', { name: /Insérer trigger\.id/ }),
		)

		expect(onInsert).toHaveBeenCalledWith('trigger.id')
	})

	it('carries the leaf path as the drag payload', async () => {
		const user = userEvent.setup()
		renderTree()

		await user.click(screen.getByRole('button', { name: /trigger/ }))
		const leafButton = screen.getByRole('button', {
			name: /Insérer trigger\.id/,
		})

		const dataTransfer = { setData: vi.fn() }
		fireEvent.dragStart(leafButton, { dataTransfer })

		expect(dataTransfer.setData).toHaveBeenCalledWith(
			'text/plain',
			'trigger.id',
		)
	})
})

describe('AvailableDataTree — source toggle', () => {
	it('disables the last-run option until a run exists', () => {
		renderTree({ hasLastRun: false })

		const button = screen.getByRole('button', {
			name: 'Dernière exécution',
		}) as HTMLButtonElement
		expect(button.disabled).toBe(true)
	})

	it('reports the requested source once a run exists', async () => {
		const user = userEvent.setup()
		const { onSourceChange } = renderTree({ hasLastRun: true })

		await user.click(screen.getByRole('button', { name: 'Dernière exécution' }))

		expect(onSourceChange).toHaveBeenCalledWith('last_run')
	})

	it('marks the active source', () => {
		renderTree({ source: 'example' })

		expect(
			screen
				.getByRole('button', { name: 'Exemple' })
				.getAttribute('aria-pressed'),
		).toBe('true')
	})
})

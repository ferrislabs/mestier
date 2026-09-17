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

const TRIGGER_NOTICE: DataTreeNode = {
	kind: 'notice',
	key: 'trigger',
	label: 'trigger',
	path: 'trigger',
	message: 'Aucun événement déclencheur n’est configuré.',
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
	render(
		<AvailableDataTree
			branches={[TRIGGER_BRANCH, CONNECTOR_BRANCH]}
			onInsert={onInsert}
			{...overrides}
		/>,
	)
	return { onInsert }
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

	it('shows the trigger notice as static text instead of an expandable branch', () => {
		renderTree({ branches: [TRIGGER_NOTICE, CONNECTOR_BRANCH] })

		expect(screen.getByText(TRIGGER_NOTICE.message)).toBeDefined()
		expect(screen.queryByRole('button', { name: /^trigger$/ })).toBeNull()
		expect(
			screen.getByRole('button', { name: /c1 · Créer un client/ }),
		).toBeDefined()
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

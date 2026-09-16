import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type {
	AutomationWorkflowsListProps,
	WorkflowRow,
} from '#/pages/automation/ui/automation-workflows-list'
import { AutomationWorkflowsList } from '#/pages/automation/ui/automation-workflows-list'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'

function workflow(overrides: Partial<WorkflowRow> = {}): WorkflowRow {
	return {
		id: 'workflow-1',
		name: 'Créer une facture Odoo',
		description: 'Émet la facture dès la signature du devis.',
		enabled: true,
		triggerMode: 'events',
		lastRun: null,
		...overrides,
	}
}

function baseProps(
	overrides: Partial<AutomationWorkflowsListProps> = {},
): AutomationWorkflowsListProps {
	return {
		organizationName: 'Atelier Bois & Co',
		organizationSlug: 'atelier-bois',
		workflows: [workflow()],
		isLoading: false,
		error: null,
		onCreate: vi.fn(),
		onRename: vi.fn(),
		onToggleEnabled: vi.fn(),
		onDelete: vi.fn(),
		...overrides,
	}
}

function render(
	props: Partial<AutomationWorkflowsListProps> = {},
	permissions: string[] = ['VIEW_AUTOMATION', 'MANAGE_AUTOMATION'],
) {
	return renderWithRouter(
		wrapWithPermissions(<AutomationWorkflowsList {...baseProps(props)} />, {
			permissions,
		}),
	)
}

describe('AutomationWorkflowsList — table content', () => {
	it('renders a workflow with its name, description and enabled state', async () => {
		await render()

		expect(screen.getByText('Créer une facture Odoo')).toBeDefined()
		expect(
			screen.getByText('Émet la facture dès la signature du devis.'),
		).toBeDefined()
		expect(screen.getByText('Activé')).toBeDefined()
	})

	it('shows a disabled workflow as such', async () => {
		await render({ workflows: [workflow({ enabled: false })] })

		expect(screen.getByText('Désactivé')).toBeDefined()
	})

	it('shows the last run status when the workflow has run before', async () => {
		await render({
			workflows: [
				workflow({ lastRun: { status: 'failed', at: '2026-08-01T00:00:00Z' } }),
			],
		})

		expect(screen.getByText('Échoué')).toBeDefined()
	})

	it('says plainly when a workflow has never run', async () => {
		await render({ workflows: [workflow({ lastRun: null })] })

		expect(screen.getByText('Jamais exécuté')).toBeDefined()
	})

	it('tells an events workflow apart from a manual one', async () => {
		await render({
			workflows: [
				workflow({ id: 'a', triggerMode: 'events' }),
				workflow({ id: 'b', triggerMode: 'manual' }),
			],
		})

		expect(screen.getByText('Sur événement')).toBeDefined()
		expect(screen.getByText('Manuel')).toBeDefined()
	})
})

describe('AutomationWorkflowsList — empty state', () => {
	it('explains what a workflow is instead of showing an empty table', async () => {
		await render({ workflows: [] })

		expect(screen.getByText('Aucun workflow')).toBeDefined()
		expect(screen.queryByText('Créer une facture Odoo')).toBeNull()
	})
})

describe('AutomationWorkflowsList — create', () => {
	it('calls onCreate from the header action', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await render(props)

		await user.click(screen.getByRole('button', { name: 'Nouveau workflow' }))

		expect(props.onCreate).toHaveBeenCalled()
	})
})

describe('AutomationWorkflowsList — row actions', () => {
	it('calls onRename with the row', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await render(props)

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: 'Renommer' }))

		expect(props.onRename).toHaveBeenCalledWith(workflow())
	})

	it('offers to disable an enabled workflow, and calls onToggleEnabled with the row', async () => {
		const user = userEvent.setup()
		const props = baseProps({ workflows: [workflow({ enabled: true })] })
		await render(props)

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: 'Désactiver' }))

		expect(props.onToggleEnabled).toHaveBeenCalledWith(
			workflow({ enabled: true }),
		)
	})

	it('offers to enable a disabled workflow', async () => {
		const user = userEvent.setup()
		const props = baseProps({ workflows: [workflow({ enabled: false })] })
		await render(props)

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: 'Activer' }))

		expect(props.onToggleEnabled).toHaveBeenCalledWith(
			workflow({ enabled: false }),
		)
	})

	it('names the workflow in the delete confirmation, and only calls onDelete once confirmed', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await render(props)

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: 'Supprimer' }))

		expect(props.onDelete).not.toHaveBeenCalled()
		expect(
			screen.getByRole('alertdialog', {
				name: /Supprimer Créer une facture Odoo/,
			}),
		).toBeDefined()

		await user.click(screen.getByRole('button', { name: 'Supprimer' }))

		expect(props.onDelete).toHaveBeenCalledWith(workflow())
	})
})

describe('AutomationWorkflowsList — permission gating', () => {
	it('offers no create button to a caller without MANAGE_AUTOMATION', async () => {
		await render({}, ['VIEW_AUTOMATION'])

		expect(
			screen.queryByRole('button', { name: 'Nouveau workflow' }),
		).toBeNull()
	})

	it('offers no row actions menu to a caller without MANAGE_AUTOMATION', async () => {
		await render({}, ['VIEW_AUTOMATION'])

		expect(screen.queryByRole('button', { name: 'Actions' })).toBeNull()
	})
})

describe('AutomationWorkflowsList — reaching the editor', () => {
	it('links a workflow name to its editor', async () => {
		await render()

		const link = screen.getByRole('link', { name: /Créer une facture Odoo/ })

		expect(link.getAttribute('href')).toBe(
			'/o/atelier-bois/automatisation/workflow-1',
		)
	})

	it('offers the execution history from the row menu', async () => {
		await render()

		await userEvent.click(screen.getByRole('button', { name: 'Actions' }))

		const history = screen.getByRole('menuitem', { name: /Historique/ })
		expect(history.getAttribute('href')).toBe(
			'/o/atelier-bois/automatisation/workflow-1/executions',
		)
	})

	it('names the rename action for what it does, not for editing the workflow', async () => {
		await render()

		await userEvent.click(screen.getByRole('button', { name: 'Actions' }))

		expect(screen.getByRole('menuitem', { name: 'Renommer' })).toBeDefined()
		expect(screen.queryByRole('menuitem', { name: 'Modifier' })).toBeNull()
	})
})

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
		lastRun: null,
		updatedAt: '2026-08-01T00:00:00Z',
		tags: [],
		folderLabel: null,
		...overrides,
	}
}

function baseProps(
	overrides: Partial<AutomationWorkflowsListProps> = {},
): AutomationWorkflowsListProps {
	return {
		organizationSlug: 'atelier-bois',
		workflows: [workflow()],
		isLoading: false,
		error: null,
		folder: null,
		folderPath: [],
		folderMissing: false,
		subfolders: [],
		hasWorkflowsInScope: true,
		query: '',
		onQueryChange: vi.fn(),
		tag: 'all',
		tags: [],
		onTagChange: vi.fn(),
		sort: 'updated',
		onSortChange: vi.fn(),
		onResetFilters: vi.fn(),
		onCreate: vi.fn(),
		onRename: vi.fn(),
		onToggleEnabled: vi.fn(),
		onDuplicate: vi.fn(),
		onExport: vi.fn(),
		onArchive: vi.fn(),
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
		expect(
			screen
				.getByRole('switch', { name: /Désactiver le workflow/ })
				.getAttribute('aria-checked'),
		).toBe('true')
	})

	it('shows a disabled workflow as such', async () => {
		await render({ workflows: [workflow({ enabled: false })] })

		expect(
			screen
				.getByRole('switch', { name: /Activer le workflow/ })
				.getAttribute('aria-checked'),
		).toBe('false')
	})

	it('shows the tags and the project of a workflow', async () => {
		await render({
			workflows: [
				workflow({ tags: ['Ventes'], folderLabel: 'Ventes › Relances' }),
			],
		})

		expect(screen.getByText('Ventes')).toBeDefined()
		expect(screen.getByText('Ventes › Relances')).toBeDefined()
	})

	it('shows the last run status when the workflow has run before', async () => {
		await render({
			workflows: [
				workflow({ lastRun: { status: 'failed', at: '2026-08-01T00:00:00Z' } }),
			],
		})

		expect(screen.getByText('Échec')).toBeDefined()
	})

	it('says plainly when a workflow has never run', async () => {
		await render({ workflows: [workflow({ lastRun: null })] })

		expect(screen.getByText('Jamais exécuté')).toBeDefined()
	})
})

describe('AutomationWorkflowsList — empty state', () => {
	it('explains what a workflow is instead of showing an empty table', async () => {
		await render({ workflows: [], hasWorkflowsInScope: false })

		expect(screen.getByText('Aucun workflow')).toBeDefined()
		expect(screen.queryByText('Créer une facture Odoo')).toBeNull()
	})

	it('offers to reset the filters when they hide every workflow', async () => {
		const user = userEvent.setup()
		const props = baseProps({ workflows: [], query: 'zzz' })
		await render(props)

		expect(
			screen.getByText('Aucun workflow ne correspond à « zzz »'),
		).toBeDefined()

		await user.click(
			screen.getByRole('button', { name: 'Réinitialiser les filtres' }),
		)

		expect(props.onResetFilters).toHaveBeenCalled()
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

		await user.click(screen.getByRole('button', { name: /Actions pour/ }))
		await user.click(screen.getByRole('menuitem', { name: 'Renommer' }))

		expect(props.onRename).toHaveBeenCalledWith(workflow())
	})

	it('lists the prototype actions first, in order', async () => {
		const user = userEvent.setup()
		await render()

		await user.click(screen.getByRole('button', { name: /Actions pour/ }))

		const labels = screen
			.getAllByRole('menuitem')
			.map((item) => item.textContent)
		expect(labels.slice(0, 2)).toEqual(['Dupliquer', 'Exporter'])
		expect(labels).toContain('Archiver')
		expect(labels.indexOf('Archiver')).toBeGreaterThan(
			labels.indexOf('Renommer'),
		)
	})

	it('calls onDuplicate, onExport and onArchive with the row', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await render(props)

		for (const [name, handler] of [
			['Dupliquer', props.onDuplicate],
			['Exporter', props.onExport],
			['Archiver', props.onArchive],
		] as const) {
			await user.click(screen.getByRole('button', { name: /Actions pour/ }))
			await user.click(screen.getByRole('menuitem', { name }))
			expect(handler).toHaveBeenCalledWith(workflow())
		}
	})

	it('toggles an enabled workflow from its switch, with the row', async () => {
		const user = userEvent.setup()
		const props = baseProps({ workflows: [workflow({ enabled: true })] })
		await render(props)

		await user.click(screen.getByRole('switch', { name: /Désactiver/ }))

		expect(props.onToggleEnabled).toHaveBeenCalledWith(
			workflow({ enabled: true }),
		)
	})

	it('toggles a disabled workflow from its switch', async () => {
		const user = userEvent.setup()
		const props = baseProps({ workflows: [workflow({ enabled: false })] })
		await render(props)

		await user.click(screen.getByRole('switch', { name: /Activer/ }))

		expect(props.onToggleEnabled).toHaveBeenCalledWith(
			workflow({ enabled: false }),
		)
	})

	it('names the workflow in the delete confirmation, and only calls onDelete once confirmed', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await render(props)

		await user.click(screen.getByRole('button', { name: /Actions pour/ }))
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

		expect(screen.queryByRole('button', { name: /Actions pour/ })).toBeNull()
	})
})

describe('AutomationWorkflowsList — projects', () => {
	it('lists the subfolders with their workflow count', async () => {
		await render({
			subfolders: [
				{
					folder: { id: 'ventes', name: 'Ventes', parentId: null },
					workflowCount: 2,
				},
			],
		})

		const link = screen.getByRole('link', { name: 'Ventes' })
		expect(link.getAttribute('href')).toBe(
			'/o/atelier-bois/automatisation/projets/ventes',
		)
		expect(screen.getByText('2 workflows')).toBeDefined()
	})

	it('titles the page after the project and shows its breadcrumb', async () => {
		const ventes = { id: 'ventes', name: 'Ventes', parentId: null }
		await render({ folder: ventes, folderPath: [ventes] })

		expect(screen.getByRole('heading', { name: 'Ventes' })).toBeDefined()
		expect(
			screen.getByRole('navigation', { name: 'Emplacement du projet' }),
		).toBeDefined()
	})

	it('says so when the project does not exist', async () => {
		await render({ folderMissing: true })

		expect(screen.getByText('Projet introuvable')).toBeDefined()
		expect(
			screen.getByRole('link', { name: 'Voir tous les workflows' }),
		).toBeDefined()
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

		await userEvent.click(screen.getByRole('button', { name: /Actions pour/ }))

		const history = screen.getByRole('menuitem', { name: /Historique/ })
		expect(history.getAttribute('href')).toBe(
			'/o/atelier-bois/automatisation/workflow-1/executions',
		)
	})

	it('names the rename action for what it does, not for editing the workflow', async () => {
		await render()

		await userEvent.click(screen.getByRole('button', { name: /Actions pour/ }))

		expect(screen.getByRole('menuitem', { name: 'Renommer' })).toBeDefined()
		expect(screen.queryByRole('menuitem', { name: 'Modifier' })).toBeNull()
	})
})

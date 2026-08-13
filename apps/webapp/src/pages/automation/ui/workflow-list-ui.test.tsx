import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import {
	WorkflowListUI,
	type WorkflowRow,
} from '#/pages/automation/ui/workflow-list-ui'
import { renderWithRouter } from '#/test/render-with-router'

function workflow(overrides: Partial<WorkflowRow> = {}): WorkflowRow {
	return {
		id: 'workflow-1',
		name: 'Relance devis',
		description: 'Envoie une relance 3 jours après un devis sans réponse',
		enabled: true,
		lastRunStatus: null,
		lastRunAt: null,
		...overrides,
	}
}

function baseProps() {
	return {
		organizationName: 'Atelier Bois & Co',
		organizationSlug: 'atelier-bois',
		isLoading: false,
		error: null,
		workflows: [workflow()],
		createDialogOpen: false,
		onOpenCreateDialog: vi.fn(),
		onCreateDialogOpenChange: vi.fn(),
		createForm: {
			values: { name: '', description: '' },
			isPending: false,
			onChange: vi.fn(),
			onSubmit: vi.fn(),
		},
		togglingId: null as string | null,
		onToggleEnabled: vi.fn(),
		deletingId: null as string | null,
		onDelete: vi.fn(),
	}
}

describe('WorkflowListUI — empty state', () => {
	it('shows a placeholder when there is no workflow yet', async () => {
		await renderWithRouter(<WorkflowListUI {...baseProps()} workflows={[]} />)

		expect(screen.getByText('Aucun workflow pour le moment')).toBeDefined()
	})
})

describe('WorkflowListUI — status column', () => {
	it('shows "Jamais exécuté" when the workflow has no run yet', async () => {
		await renderWithRouter(<WorkflowListUI {...baseProps()} />)

		expect(screen.getByText('Jamais exécuté')).toBeDefined()
	})

	it("shows the last run's status when one exists", async () => {
		await renderWithRouter(
			<WorkflowListUI
				{...baseProps()}
				workflows={[workflow({ lastRunStatus: 'failed' })]}
			/>,
		)

		expect(screen.getByText('Échoué')).toBeDefined()
	})
})

describe('WorkflowListUI — enable/disable toggle', () => {
	it('calls onToggleEnabled with the row when the switch is flipped', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(<WorkflowListUI {...props} />)

		await user.click(screen.getByRole('switch'))

		expect(props.onToggleEnabled).toHaveBeenCalledWith(workflow())
	})
})

describe('WorkflowListUI — row links', () => {
	it('links to the editor and to the runs page for the workflow', async () => {
		const user = userEvent.setup()
		await renderWithRouter(<WorkflowListUI {...baseProps()} />)

		await user.click(screen.getByRole('button', { name: 'Actions' }))

		const editorLink = screen.getByRole('menuitem', { name: /éditeur/ })
		expect(editorLink.getAttribute('href')).toBe(
			'/o/atelier-bois/automation/workflow-1',
		)

		const runsLink = screen.getByRole('menuitem', { name: /Voir les runs/ })
		expect(runsLink.getAttribute('href')).toBe(
			'/o/atelier-bois/automation/workflow-1/runs',
		)
	})
})

describe('WorkflowListUI — deletion goes through a confirmation dialog', () => {
	it('does not call onDelete until the confirmation is accepted', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(<WorkflowListUI {...props} />)

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: /Supprimer/ }))

		expect(props.onDelete).not.toHaveBeenCalled()
		expect(
			screen.getByRole('alertdialog', { name: /Supprimer Relance devis/ }),
		).toBeDefined()

		await user.click(screen.getByRole('button', { name: 'Supprimer' }))

		expect(props.onDelete).toHaveBeenCalledWith(workflow())
	})
})

describe('WorkflowListUI — create dialog', () => {
	it('the "Ajouter" button opens the dialog, never submits directly', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(<WorkflowListUI {...props} />)

		await user.click(screen.getByRole('button', { name: 'Ajouter' }))

		expect(props.onOpenCreateDialog).toHaveBeenCalled()
		expect(props.createForm.onSubmit).not.toHaveBeenCalled()
	})

	it('submits the create form from inside the open dialog', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(
			<WorkflowListUI {...props} createDialogOpen={true} />,
		)

		await user.type(screen.getByLabelText('Nom'), 'Nouveau workflow')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(props.createForm.onSubmit).toHaveBeenCalled()
	})

	it('cancelling closes the dialog without submitting', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(
			<WorkflowListUI {...props} createDialogOpen={true} />,
		)

		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(props.onCreateDialogOpenChange).toHaveBeenCalledWith(false)
		expect(props.createForm.onSubmit).not.toHaveBeenCalled()
	})
})

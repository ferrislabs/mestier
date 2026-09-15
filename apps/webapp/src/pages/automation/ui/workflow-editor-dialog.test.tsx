import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import type { WorkflowEditorDialogProps } from '#/pages/automation/ui/workflow-editor-dialog'
import { WorkflowEditorDialog } from '#/pages/automation/ui/workflow-editor-dialog'
import { renderWithPermissions } from '#/test/with-permissions'

function renderDialog(
	ui: ReactNode,
	permissions: string[] = ['MANAGE_AUTOMATION'],
) {
	return renderWithPermissions(ui, { permissions })
}

function baseProps(
	overrides: Partial<WorkflowEditorDialogProps> = {},
): WorkflowEditorDialogProps {
	return {
		open: true,
		editingName: null,
		values: { name: '', description: '' },
		isPending: false,
		error: null,
		onOpenChange: vi.fn(),
		onValuesChange: vi.fn(),
		onSubmit: vi.fn(),
		...overrides,
	}
}

describe('WorkflowEditorDialog — visibility', () => {
	it('renders nothing when closed', async () => {
		renderDialog(<WorkflowEditorDialog {...baseProps({ open: false })} />)

		expect(screen.queryByLabelText('Nom')).toBeNull()
	})
})

describe('WorkflowEditorDialog — create mode', () => {
	it('titles itself "Nouveau workflow" and offers a "Créer" button', async () => {
		renderDialog(<WorkflowEditorDialog {...baseProps()} />)

		expect(screen.getByText('Nouveau workflow')).toBeDefined()
		expect(screen.getByRole('button', { name: 'Créer' })).toBeDefined()
	})

	it('disables the submit button until a name is entered', async () => {
		renderDialog(
			<WorkflowEditorDialog
				{...baseProps({ values: { name: '', description: '' } })}
			/>,
		)

		expect(
			(screen.getByRole('button', { name: 'Créer' }) as HTMLButtonElement)
				.disabled,
		).toBe(true)
	})

	it('enables the submit button once a name is entered', async () => {
		renderDialog(
			<WorkflowEditorDialog
				{...baseProps({ values: { name: 'Créer une facture Odoo', description: '' } })}
			/>,
		)

		expect(
			(screen.getByRole('button', { name: 'Créer' }) as HTMLButtonElement)
				.disabled,
		).toBe(false)
	})
})

describe('WorkflowEditorDialog — edit mode', () => {
	it('titles itself after the workflow being edited and offers "Enregistrer"', async () => {
		renderDialog(
			<WorkflowEditorDialog
				{...baseProps({
					editingName: 'Créer une facture Odoo',
					values: { name: 'Créer une facture Odoo', description: '' },
				})}
			/>,
		)

		expect(screen.getByText('Modifier Créer une facture Odoo')).toBeDefined()
		expect(screen.getByRole('button', { name: 'Enregistrer' })).toBeDefined()
	})
})

describe('WorkflowEditorDialog — field wiring', () => {
	it('reports a name change through onValuesChange', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		renderDialog(<WorkflowEditorDialog {...props} />)

		await user.type(screen.getByLabelText('Nom'), 'X')

		expect(props.onValuesChange).toHaveBeenCalledWith({ name: 'X' })
	})

	it('reports a description change through onValuesChange', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		renderDialog(<WorkflowEditorDialog {...props} />)

		await user.type(screen.getByLabelText('Description'), 'X')

		expect(props.onValuesChange).toHaveBeenCalledWith({ description: 'X' })
	})

	it('calls onSubmit when the submit button is pressed', async () => {
		const user = userEvent.setup()
		const props = baseProps({ values: { name: 'Facture', description: '' } })
		renderDialog(<WorkflowEditorDialog {...props} />)

		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(props.onSubmit).toHaveBeenCalled()
	})

	it('closes on cancel without submitting', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		renderDialog(<WorkflowEditorDialog {...props} />)

		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(props.onOpenChange).toHaveBeenCalledWith(false)
		expect(props.onSubmit).not.toHaveBeenCalled()
	})

	it('shows the error message when one is given', async () => {
		renderDialog(
			<WorkflowEditorDialog {...baseProps({ error: 'HTTP 409: Conflict' })} />,
		)

		expect(screen.getByText('HTTP 409: Conflict')).toBeDefined()
	})
})

describe('WorkflowEditorDialog — permission gating', () => {
	it('offers no submit button to a caller without MANAGE_AUTOMATION', async () => {
		renderDialog(
			<WorkflowEditorDialog
				{...baseProps({ values: { name: 'Facture', description: '' } })}
			/>,
			[],
		)

		expect(screen.queryByRole('button', { name: 'Créer' })).toBeNull()
		expect(screen.getByRole('button', { name: 'Annuler' })).toBeDefined()
	})
})

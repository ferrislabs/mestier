import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useState } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { EditorTopBar } from '#/pages/automation/ui/editor-top-bar'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'

type Props = Parameters<typeof EditorTopBar>[0]

function Controlled(props: Props) {
	const [name, setName] = useState(props.name)
	return (
		<EditorTopBar
			{...props}
			name={name}
			onNameChange={(next) => {
				setName(next)
				props.onNameChange(next)
			}}
		/>
	)
}

function render(
	overrides: Partial<Props> = {},
	permissions = ['VIEW_AUTOMATION', 'MANAGE_AUTOMATION'],
) {
	const props = {
		organizationSlug: 'atelier-bois',
		name: 'Relance des devis',
		canRename: true,
		onNameChange: vi.fn(),
		onNameBlur: vi.fn(),
		tags: ['Ventes'],
		view: 'editor' as const,
		onViewChange: vi.fn(),
		enabled: true,
		onEnabledChange: vi.fn(),
		save: { dirty: true, isSaving: false, onSave: vi.fn() },
		...overrides,
	}
	return {
		props,
		rendered: renderWithRouter(
			wrapWithPermissions(<Controlled {...props} />, { permissions }),
		),
	}
}

describe('EditorTopBar', () => {
	it('links back to the workflows and shows the tags', async () => {
		await render().rendered

		expect(
			screen.getByRole('link', { name: 'Workflows' }).getAttribute('href'),
		).toBe('/o/atelier-bois/automatisation/workflows')
		expect(screen.getByText('Ventes')).toBeDefined()
	})

	it('reports the name at every keystroke and again when the field loses focus', async () => {
		const user = userEvent.setup()
		const { props, rendered } = render()
		await rendered
		const input = screen.getByLabelText('Nom du workflow') as HTMLInputElement

		await user.clear(input)
		await user.type(input, 'Relance 2')

		expect(props.onNameChange).toHaveBeenLastCalledWith('Relance 2')
		expect(vi.mocked(props.onNameChange).mock.calls.length).toBeGreaterThan(2)
		expect(props.onNameBlur).not.toHaveBeenCalled()

		await user.tab()

		expect(props.onNameBlur).toHaveBeenCalledTimes(1)
	})

	it('lets Enter leave the field', async () => {
		const user = userEvent.setup()
		const { props, rendered } = render()
		await rendered

		await user.type(screen.getByLabelText('Nom du workflow'), '{Enter}')

		expect(props.onNameBlur).toHaveBeenCalledTimes(1)
	})

	it('shows the name it is given, not a copy of its own', async () => {
		const { rendered } = render({ name: 'Mon workflow' })
		await rendered

		expect(
			(screen.getByLabelText('Nom du workflow') as HTMLInputElement).value,
		).toBe('Mon workflow')
	})

	it('keeps the name read-only without the right to rename', async () => {
		await render({ canRename: false }).rendered

		expect(
			(screen.getByLabelText('Nom du workflow') as HTMLInputElement).readOnly,
		).toBe(true)
	})

	it('disables the switch when nobody can answer it', async () => {
		await render({ onEnabledChange: undefined, save: undefined }).rendered

		expect(
			(screen.getByRole('switch', { name: 'Actif' }) as HTMLButtonElement)
				.disabled,
		).toBe(true)
		expect(screen.queryByRole('button', { name: 'Enregistrer' })).toBeNull()
	})

	it('switches between the editor and the executions', async () => {
		const user = userEvent.setup()
		const { props, rendered } = render()
		await rendered

		await user.click(screen.getByRole('tab', { name: 'Exécutions' }))

		expect(props.onViewChange).toHaveBeenCalledWith('executions')
	})

	it('toggles the workflow and saves it', async () => {
		const user = userEvent.setup()
		const { props, rendered } = render()
		await rendered

		await user.click(screen.getByRole('switch', { name: 'Actif' }))
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(props.onEnabledChange).toHaveBeenCalledWith(false)
		expect(props.save.onSave).toHaveBeenCalled()
	})

	it('hides the write controls from a caller without MANAGE_AUTOMATION', async () => {
		await render({}, ['VIEW_AUTOMATION']).rendered

		expect(screen.queryByRole('switch', { name: 'Actif' })).toBeNull()
		expect(screen.queryByRole('button', { name: 'Enregistrer' })).toBeNull()
	})
})

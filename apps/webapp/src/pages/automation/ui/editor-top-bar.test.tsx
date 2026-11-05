import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { EditorTopBar } from '#/pages/automation/ui/editor-top-bar'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'

function render(
	overrides: Partial<Parameters<typeof EditorTopBar>[0]> = {},
	permissions = ['VIEW_AUTOMATION', 'MANAGE_AUTOMATION'],
) {
	const onNameCommit = vi.fn()
	const props = {
		organizationSlug: 'atelier-bois',
		name: 'Relance des devis',
		canRename: true,
		onNameCommit,
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
			wrapWithPermissions(<EditorTopBar {...props} />, { permissions }),
		),
	}
}

describe('EditorTopBar', () => {
	it('links back to the workflows and shows the tags', async () => {
		await render().rendered

		expect(
			screen.getByRole('link', { name: 'Workflows' }).getAttribute('href'),
		).toBe('/o/atelier-bois/automatisation')
		expect(screen.getByText('Ventes')).toBeDefined()
	})

	it('commits a renamed workflow on blur, and ignores an unchanged or empty name', async () => {
		const user = userEvent.setup()
		const { props, rendered } = render()
		await rendered
		const input = screen.getByLabelText('Nom du workflow')

		await user.clear(input)
		await user.type(input, 'Relance 2')
		await user.tab()
		expect(props.onNameCommit).toHaveBeenCalledWith('Relance 2')

		vi.mocked(props.onNameCommit).mockClear()
		await user.clear(input)
		await user.tab()
		expect(props.onNameCommit).not.toHaveBeenCalled()
		expect((input as HTMLInputElement).value).toBe('Relance des devis')
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

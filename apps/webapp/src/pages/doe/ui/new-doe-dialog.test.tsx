import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import {
	NewDoeDialog,
	type NewDoeDialogProps,
} from '#/pages/doe/ui/new-doe-dialog'
import { renderWithPermissions } from '#/test/with-permissions'

class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
// biome-ignore lint/suspicious/noExplicitAny: test-only global polyfill
const globalAny = globalThis as any
globalAny.ResizeObserver ??= ResizeObserverStub
Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false

const PROPS: NewDoeDialogProps = {
	open: true,
	mode: 'pick-model',
	projects: [{ id: 'p1', name: 'Jardin Duval' }],
	models: [{ id: 'm1', name: 'Paysagiste' }],
	selectedProjectId: '',
	selectedModelId: '',
	isLoading: false,
	isPending: false,
	error: null,
	onOpenChange: () => {},
	onSelectProject: () => {},
	onSelectModel: () => {},
	onCreate: () => {},
	onCreateStarter: () => {},
}

describe('NewDoeDialog', () => {
	it('disables Créer until a project and a model are chosen', () => {
		const { rerender } = renderWithPermissions(<NewDoeDialog {...PROPS} />)
		const button = () =>
			screen.getByRole('button', { name: 'Créer' }) as HTMLButtonElement
		expect(button().disabled).toBe(true)

		rerender(<NewDoeDialog {...PROPS} selectedProjectId="p1" />)
		expect(button().disabled).toBe(true)

		rerender(
			<NewDoeDialog {...PROPS} selectedProjectId="p1" selectedModelId="m1" />,
		)
		expect(button().disabled).toBe(false)
	}, 20000)

	it('shows the preselected model and creates on click', async () => {
		const user = userEvent.setup()
		const onCreate = vi.fn()
		renderWithPermissions(
			<NewDoeDialog
				{...PROPS}
				selectedProjectId="p1"
				selectedModelId="m1"
				onCreate={onCreate}
			/>,
		)
		expect(
			screen.getByRole('combobox', { name: 'Modèle de DOE' }).textContent,
		).toContain('Paysagiste')
		await user.click(screen.getByRole('button', { name: 'Créer' }))
		expect(onCreate).toHaveBeenCalledTimes(1)
	})

	it('offers the landscaping starter when there is no model', () => {
		renderWithPermissions(
			<NewDoeDialog {...PROPS} mode="from-starter" models={[]} />,
		)
		expect(
			screen.getByRole('button', { name: 'Créer le modèle Paysagiste' }),
		).toBeDefined()
	})
})

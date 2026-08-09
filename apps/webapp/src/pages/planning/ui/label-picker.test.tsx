import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { LabelPicker } from '#/pages/planning/ui/label-picker'

class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
// biome-ignore lint/suspicious/noExplicitAny: test-only global polyfill
;(globalThis as any).ResizeObserver ??= ResizeObserverStub

const LABELS = [
	{ id: 'l1', name: 'Réunion', color: '#2563EB' },
	{ id: 'l2', name: 'Déplacement', color: '#16A34A' },
	{ id: 'l3', name: 'Formation', color: '#F59E0B' },
]

function baseProps() {
	return {
		labels: LABELS,
		selectedIds: [] as string[],
		isCreating: false,
		onToggle: vi.fn(),
		onCreate: vi.fn(),
	}
}

describe('LabelPicker', () => {
	it('shows a placeholder when nothing is selected', () => {
		render(<LabelPicker {...baseProps()} />)
		expect(screen.getByText('Aucun label')).toBeDefined()
	})

	it('renders a pastille per selected label on the trigger', () => {
		render(<LabelPicker {...baseProps()} selectedIds={['l1', 'l2']} />)
		expect(screen.getByText('Réunion')).toBeDefined()
		expect(screen.getByText('Déplacement')).toBeDefined()
		expect(screen.queryByText('Formation')).toBeNull()
	})

	it('lists every organization label in the popover', async () => {
		const user = userEvent.setup()
		render(<LabelPicker {...baseProps()} />)

		await user.click(screen.getByRole('button', { name: /Aucun label/ }))

		expect(screen.getByRole('option', { name: /Réunion/ })).toBeDefined()
		expect(screen.getByRole('option', { name: /Déplacement/ })).toBeDefined()
		expect(screen.getByRole('option', { name: /Formation/ })).toBeDefined()
	})

	it('calls onToggle when an existing label is clicked', async () => {
		const user = userEvent.setup()
		const onToggle = vi.fn()
		render(<LabelPicker {...baseProps()} onToggle={onToggle} />)

		await user.click(screen.getByRole('button', { name: /Aucun label/ }))
		await user.click(screen.getByRole('option', { name: /Réunion/ }))

		expect(onToggle).toHaveBeenCalledWith('l1')
	})

	it('marks a selected label as checked in the list', async () => {
		const user = userEvent.setup()
		render(<LabelPicker {...baseProps()} selectedIds={['l2']} />)

		await user.click(screen.getByRole('button', { name: /Déplacement/ }))

		expect(
			screen
				.getByRole('option', { name: /Déplacement/ })
				.getAttribute('aria-selected'),
		).toBe('true')
	})

	it('offers to create a label when the typed name matches nothing', async () => {
		const user = userEvent.setup()
		render(<LabelPicker {...baseProps()} />)

		await user.click(screen.getByRole('button', { name: /Aucun label/ }))
		await user.type(
			screen.getByPlaceholderText('Rechercher ou créer…'),
			'Urgent',
		)

		expect(
			screen.getByRole('button', { name: /Créer « Urgent »/ }),
		).toBeDefined()
	})

	it('does not offer to create a label that already exists (case-insensitive)', async () => {
		const user = userEvent.setup()
		render(<LabelPicker {...baseProps()} />)

		await user.click(screen.getByRole('button', { name: /Aucun label/ }))
		await user.type(
			screen.getByPlaceholderText('Rechercher ou créer…'),
			'réunion',
		)

		expect(screen.queryByRole('button', { name: /Créer/ })).toBeNull()
	})

	it('calls onCreate with the typed name when confirmed', async () => {
		const user = userEvent.setup()
		const onCreate = vi.fn()
		render(<LabelPicker {...baseProps()} onCreate={onCreate} />)

		await user.click(screen.getByRole('button', { name: /Aucun label/ }))
		await user.type(
			screen.getByPlaceholderText('Rechercher ou créer…'),
			'Urgent',
		)
		await user.click(screen.getByRole('button', { name: /Créer « Urgent »/ }))

		expect(onCreate).toHaveBeenCalledWith('Urgent')
	})

	it('filters the visible list as the user types', async () => {
		const user = userEvent.setup()
		render(<LabelPicker {...baseProps()} />)

		await user.click(screen.getByRole('button', { name: /Aucun label/ }))
		await user.type(screen.getByPlaceholderText('Rechercher ou créer…'), 'form')

		expect(screen.getByRole('option', { name: /Formation/ })).toBeDefined()
		expect(screen.queryByRole('option', { name: /Réunion/ })).toBeNull()
	})
})

describe('LabelPicker — no network call', () => {
	let fetchSpy: ReturnType<typeof createFetchSpy>

	function createFetchSpy() {
		return vi.spyOn(global, 'fetch').mockImplementation(() => {
			throw new Error('le composant ui/ ne doit jamais appeler fetch')
		})
	}

	beforeEach(() => {
		fetchSpy = createFetchSpy()
	})

	afterEach(() => {
		fetchSpy.mockRestore()
	})

	it('never triggers fetch, including after opening and typing', async () => {
		const user = userEvent.setup()
		render(<LabelPicker {...baseProps()} />)

		await user.click(screen.getByRole('button', { name: /Aucun label/ }))
		await user.type(
			within(document.body).getByPlaceholderText('Rechercher ou créer…'),
			'Urgent',
		)

		expect(fetchSpy).not.toHaveBeenCalled()
	})
})

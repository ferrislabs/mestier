import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { AssigneePicker } from '#/pages/planning/ui/assignee-picker'

class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
// biome-ignore lint/suspicious/noExplicitAny: test-only global polyfill
;(globalThis as any).ResizeObserver ??= ResizeObserverStub

const OPTIONS = [
	{ resourceId: 'member:m1', displayName: 'Martin Alix' },
	{ resourceId: 'member:m2', displayName: 'Leroy Marie' },
	{ resourceId: 'member:m3', displayName: 'Petit Sacha' },
]

function baseProps() {
	return {
		options: OPTIONS,
		selectedResourceIds: [] as string[],
		onToggle: vi.fn(),
	}
}

describe('AssigneePicker', () => {
	it('shows a placeholder when nobody is assigned', () => {
		render(<AssigneePicker {...baseProps()} />)
		expect(screen.getByText('Personne assigné')).toBeDefined()
	})

	it('lists the assigned people on the trigger', () => {
		render(
			<AssigneePicker {...baseProps()} selectedResourceIds={['member:m1']} />,
		)
		expect(screen.getByText('Martin Alix')).toBeDefined()
	})

	it('lists every option in the popover', async () => {
		const user = userEvent.setup()
		render(<AssigneePicker {...baseProps()} />)

		await user.click(screen.getByRole('button', { name: /Personne assigné/ }))

		expect(screen.getByRole('option', { name: /Martin Alix/ })).toBeDefined()
		expect(screen.getByRole('option', { name: /Leroy Marie/ })).toBeDefined()
		expect(screen.getByRole('option', { name: /Petit Sacha/ })).toBeDefined()
	})

	it('calls onToggle with the resource id when an option is clicked', async () => {
		const user = userEvent.setup()
		const onToggle = vi.fn()
		render(<AssigneePicker {...baseProps()} onToggle={onToggle} />)

		await user.click(screen.getByRole('button', { name: /Personne assigné/ }))
		await user.click(screen.getByRole('option', { name: /Leroy Marie/ }))

		expect(onToggle).toHaveBeenCalledWith('member:m2')
	})

	it('marks a selected option as checked', async () => {
		const user = userEvent.setup()
		render(
			<AssigneePicker {...baseProps()} selectedResourceIds={['member:m3']} />,
		)

		await user.click(screen.getByRole('button', { name: /Petit Sacha/ }))

		expect(
			screen
				.getByRole('option', { name: /Petit Sacha/ })
				.getAttribute('aria-selected'),
		).toBe('true')
	})
})

describe('AssigneePicker — no network call', () => {
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

	it('never triggers fetch', async () => {
		const user = userEvent.setup()
		render(<AssigneePicker {...baseProps()} />)
		await user.click(screen.getByRole('button', { name: /Personne assigné/ }))

		expect(fetchSpy).not.toHaveBeenCalled()
	})
})

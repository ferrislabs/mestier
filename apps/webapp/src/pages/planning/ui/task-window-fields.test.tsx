import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { TaskWindowFields } from '#/pages/planning/ui/task-window-fields'

class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
// biome-ignore lint/suspicious/noExplicitAny: test-only global polyfill
;(globalThis as any).ResizeObserver ??= ResizeObserverStub
Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false
Element.prototype.releasePointerCapture ??= () => {}

function baseValues(
	overrides: Partial<{
		startDate: string
		endDate: string
		startTime: string
		endTime: string
		allDay: boolean
	}> = {},
) {
	return {
		startDate: '2026-08-10',
		endDate: '2026-08-10',
		startTime: '09:00',
		endTime: '10:00',
		allDay: false,
		...overrides,
	}
}

describe('TaskWindowFields — range trigger', () => {
	it('shows the formatted single-day range as the trigger label', () => {
		render(
			<TaskWindowFields
				values={baseValues()}
				isSubtask={false}
				windowPlaceholder={null}
				onChange={vi.fn()}
			/>,
		)
		expect(screen.getByRole('button', { name: '10/08/2026' })).toBeDefined()
	})

	it('shows a "from – to" label for a multi-day range', () => {
		render(
			<TaskWindowFields
				values={baseValues({ endDate: '2026-08-14' })}
				isSubtask={false}
				windowPlaceholder={null}
				onChange={vi.fn()}
			/>,
		)
		expect(
			screen.getByRole('button', { name: '10/08/2026 – 14/08/2026' }),
		).toBeDefined()
	})

	it('hides the time fields when all-day is on', () => {
		render(
			<TaskWindowFields
				values={baseValues({ allDay: true })}
				isSubtask={false}
				windowPlaceholder={null}
				onChange={vi.fn()}
			/>,
		)
		expect(screen.queryByLabelText('Heure de début')).toBeNull()
	})
})

describe('TaskWindowFields — subtask inheritance', () => {
	it('shows the inherited-window placeholder when the subtask has no dates', () => {
		render(
			<TaskWindowFields
				values={baseValues({ startDate: '', endDate: '' })}
				isSubtask={true}
				windowPlaceholder="Hérite du parent : 10/08/2026 09:00 – 11:00"
				onChange={vi.fn()}
			/>,
		)
		expect(
			screen.getByRole('button', {
				name: 'Hérite du parent : 10/08/2026 09:00 – 11:00',
			}),
		).toBeDefined()
		expect(
			screen.queryByRole('button', { name: 'Hériter du parent' }),
		).toBeNull()
	})

	it('offers a way back to inheriting once the subtask has its own dates', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<TaskWindowFields
				values={baseValues()}
				isSubtask={true}
				windowPlaceholder="Hérite du parent : 10/08/2026 09:00 – 11:00"
				onChange={onChange}
			/>,
		)

		await user.click(screen.getByRole('button', { name: 'Hériter du parent' }))
		expect(onChange).toHaveBeenCalledWith({ startDate: '', endDate: '' })
	})

	it('never offers "hériter du parent" for a root task', () => {
		render(
			<TaskWindowFields
				values={baseValues()}
				isSubtask={false}
				windowPlaceholder={null}
				onChange={vi.fn()}
			/>,
		)
		expect(
			screen.queryByRole('button', { name: 'Hériter du parent' }),
		).toBeNull()
	})
})

describe('TaskWindowFields — time pickers', () => {
	it('shifts the end time to preserve the duration when the start time changes', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<TaskWindowFields
				values={baseValues()}
				isSubtask={false}
				windowPlaceholder={null}
				onChange={onChange}
			/>,
		)

		await user.click(screen.getByLabelText('Heure de début'))
		await user.click(screen.getByRole('option', { name: '11:00' }))

		expect(onChange).toHaveBeenCalledWith({
			startTime: '11:00',
			endTime: '12:00',
		})
	})

	it('lists the current value even when it falls off the half-hour grid', async () => {
		const user = userEvent.setup()
		render(
			<TaskWindowFields
				values={baseValues({ startTime: '09:07' })}
				isSubtask={false}
				windowPlaceholder={null}
				onChange={vi.fn()}
			/>,
		)

		expect(
			screen.getByRole('combobox', { name: 'Heure de début' }).textContent,
		).toBe('09:07')
		await user.click(screen.getByLabelText('Heure de début'))
		expect(screen.getByRole('option', { name: '09:07' })).toBeDefined()
	})

	it('changing the end time alone does not touch the start time', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<TaskWindowFields
				values={baseValues()}
				isSubtask={false}
				windowPlaceholder={null}
				onChange={onChange}
			/>,
		)

		await user.click(screen.getByLabelText('Heure de fin'))
		await user.click(screen.getByRole('option', { name: '13:00' }))

		expect(onChange).toHaveBeenCalledWith({ endTime: '13:00' })
	})
})

describe('TaskWindowFields — no network call', () => {
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

	it('never triggers fetch at render', () => {
		render(
			<TaskWindowFields
				values={baseValues()}
				isSubtask={false}
				windowPlaceholder={null}
				onChange={vi.fn()}
			/>,
		)
		expect(fetchSpy).not.toHaveBeenCalled()
	})
})

import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { Warning } from '#/pages/planning/lib/warnings'
import { PlanningWarningDialog } from '#/pages/planning/ui/planning-warning-dialog'

const ABSENCE_WARNING: Warning = {
	kind: 'absence',
	reason: 'LEAVE',
	note: 'De retour lundi',
	startsAt: '2026-08-10T00:00:00+02:00',
	endsAt: '2026-08-11T00:00:00+02:00',
}

const OVERLAPPING_WARNING: Warning = {
	kind: 'overlapping_task',
	taskId: 'wo-1',
	startsAt: '2026-08-10T08:00:00+02:00',
	endsAt: '2026-08-10T10:00:00+02:00',
}

describe('PlanningWarningDialog', () => {
	it('renders nothing when open is false', () => {
		render(
			<PlanningWarningDialog
				open={false}
				warnings={[ABSENCE_WARNING]}
				isPending={false}
				onConfirm={vi.fn()}
				onCancel={vi.fn()}
			/>,
		)

		expect(screen.queryByRole('alertdialog')).toBeNull()
	})

	it('shows every warning in the list, in a single dialog', () => {
		render(
			<PlanningWarningDialog
				open={true}
				warnings={[ABSENCE_WARNING, OVERLAPPING_WARNING]}
				isPending={false}
				onConfirm={vi.fn()}
				onCancel={vi.fn()}
			/>,
		)

		expect(screen.getAllByRole('alertdialog')).toHaveLength(1)
		expect(screen.getByText(/Absence : Congé/)).toBeDefined()
		expect(screen.getByText('De retour lundi')).toBeDefined()
		expect(screen.getByText(/Déjà affecté à un autre projet/)).toBeDefined()
	})

	it('confirming calls onConfirm and not onCancel', async () => {
		const user = userEvent.setup()
		const onConfirm = vi.fn()
		const onCancel = vi.fn()

		render(
			<PlanningWarningDialog
				open={true}
				warnings={[ABSENCE_WARNING]}
				isPending={false}
				onConfirm={onConfirm}
				onCancel={onCancel}
			/>,
		)

		await user.click(screen.getByRole('button', { name: /Confirmer/ }))

		expect(onConfirm).toHaveBeenCalledTimes(1)
		expect(onCancel).not.toHaveBeenCalled()
	})

	it('cancelling calls onCancel and not onConfirm', async () => {
		const user = userEvent.setup()
		const onConfirm = vi.fn()
		const onCancel = vi.fn()

		render(
			<PlanningWarningDialog
				open={true}
				warnings={[ABSENCE_WARNING]}
				isPending={false}
				onConfirm={onConfirm}
				onCancel={onCancel}
			/>,
		)

		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(onCancel).toHaveBeenCalledTimes(1)
		expect(onConfirm).not.toHaveBeenCalled()
	})

	it('disables the actions and changes the label while confirming', () => {
		render(
			<PlanningWarningDialog
				open={true}
				warnings={[ABSENCE_WARNING]}
				isPending={true}
				onConfirm={vi.fn()}
				onCancel={vi.fn()}
			/>,
		)

		const action = screen.getByRole('button', {
			name: /Confirmation…/,
		}) as HTMLButtonElement
		const cancel = screen.getByRole('button', {
			name: 'Annuler',
		}) as HTMLButtonElement
		expect(action.disabled).toBe(true)
		expect(cancel.disabled).toBe(true)
	})

	it('shows the error of a previously failed confirmation without closing the dialog', () => {
		render(
			<PlanningWarningDialog
				open={true}
				warnings={[ABSENCE_WARNING]}
				isPending={false}
				error="HTTP 500: Internal Server Error"
				onConfirm={vi.fn()}
				onCancel={vi.fn()}
			/>,
		)

		expect(screen.getByRole('alertdialog')).toBeDefined()
		expect(screen.getByText('HTTP 500: Internal Server Error')).toBeDefined()
	})
})

describe('PlanningWarningDialog — no network call', () => {
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

	it('fires no fetch on render', () => {
		render(
			<PlanningWarningDialog
				open={true}
				warnings={[ABSENCE_WARNING, OVERLAPPING_WARNING]}
				isPending={false}
				onConfirm={vi.fn()}
				onCancel={vi.fn()}
			/>,
		)

		expect(fetchSpy).not.toHaveBeenCalled()
	})
})

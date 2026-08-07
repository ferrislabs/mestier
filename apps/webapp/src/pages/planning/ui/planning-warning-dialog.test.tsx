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

const MISSING_RECORD_WARNING: Warning = { kind: 'missing_employee_record' }

describe('PlanningWarningDialog', () => {
	it("n'affiche rien quand open est faux", () => {
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

	it('affiche chaque avertissement de la liste, dans un seul dialogue', () => {
		render(
			<PlanningWarningDialog
				open={true}
				warnings={[ABSENCE_WARNING, MISSING_RECORD_WARNING]}
				isPending={false}
				onConfirm={vi.fn()}
				onCancel={vi.fn()}
			/>,
		)

		expect(screen.getAllByRole('alertdialog')).toHaveLength(1)
		expect(screen.getByText(/Absence : Congé/)).toBeDefined()
		expect(screen.getByText('De retour lundi')).toBeDefined()
		expect(screen.getByText(/Aucune fiche employé/)).toBeDefined()
		expect(screen.getByText(/taux horaire/)).toBeDefined()
	})

	it('confirmer appelle onConfirm et pas onCancel', async () => {
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

	it('annuler appelle onCancel et pas onConfirm', async () => {
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

	it('désactive les actions et change le libellé pendant la confirmation', () => {
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
})

describe('PlanningWarningDialog — pas d’appel réseau', () => {
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

	it('ne déclenche aucun fetch au rendu', () => {
		render(
			<PlanningWarningDialog
				open={true}
				warnings={[ABSENCE_WARNING, MISSING_RECORD_WARNING]}
				isPending={false}
				onConfirm={vi.fn()}
				onCancel={vi.fn()}
			/>,
		)

		expect(fetchSpy).not.toHaveBeenCalled()
	})
})

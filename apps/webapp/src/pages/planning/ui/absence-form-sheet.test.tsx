import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { emptyAbsenceDraft } from '#/pages/planning/lib/absences'
import { AbsenceFormSheet } from '#/pages/planning/ui/absence-form-sheet'

// jsdom doesn't implement ResizeObserver; Radix's `Select` measures its
// trigger with it. Stubbed locally rather than in the shared
// `vitest.setup.ts`, which this workstream doesn't own.
class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
// biome-ignore lint/suspicious/noExplicitAny: test-only global polyfill
;(globalThis as any).ResizeObserver ??= ResizeObserverStub

const EMPLOYEES = [
	{ employeeId: 'emp-1', displayName: 'Alix Martin' },
	{ employeeId: 'emp-2', displayName: 'Marie Leroy' },
]

function baseProps() {
	return {
		open: true,
		mode: 'create' as const,
		values: emptyAbsenceDraft('', '2026-08-10'),
		employees: EMPLOYEES,
		errors: [],
		isSaving: false,
		isDeleting: false,
		saveError: null,
		onChange: vi.fn(),
		onSubmit: vi.fn(),
		onDelete: vi.fn(),
		onOpenChange: vi.fn(),
	}
}

describe('AbsenceFormSheet — création', () => {
	it("n'affiche rien quand open est faux", () => {
		render(<AbsenceFormSheet {...baseProps()} open={false} />)
		expect(screen.queryByRole('dialog')).toBeNull()
	})

	it('propose le sélecteur employé et pas de bouton supprimer', () => {
		render(<AbsenceFormSheet {...baseProps()} />)

		expect(screen.getByText('Nouvelle absence')).toBeDefined()
		expect(screen.getByRole('combobox', { name: 'Employé' })).toBeDefined()
		expect(screen.queryByRole('button', { name: /Supprimer/ })).toBeNull()
	})

	it('masque les champs horaires quand journée entière est actif', () => {
		render(<AbsenceFormSheet {...baseProps()} />)
		expect(screen.queryByLabelText('Heure de début')).toBeNull()
	})

	it('affiche les champs horaires quand journée entière est désactivé', () => {
		const props = baseProps()
		render(
			<AbsenceFormSheet
				{...props}
				values={{ ...props.values, allDay: false }}
			/>,
		)
		expect(screen.getByLabelText('Heure de début')).toBeDefined()
		expect(screen.getByLabelText('Heure de fin')).toBeDefined()
	})

	it('affiche les erreurs de validation et désactive la soumission', () => {
		render(<AbsenceFormSheet {...baseProps()} errors={['Employé requis']} />)

		expect(screen.getByText('Employé requis')).toBeDefined()
		const submit = screen.getByRole('button', {
			name: /Créer l’absence/,
		}) as HTMLButtonElement
		expect(submit.disabled).toBe(true)
	})

	it('appelle onSubmit au clic sur enregistrer quand valide', async () => {
		const user = userEvent.setup()
		const onSubmit = vi.fn()
		render(<AbsenceFormSheet {...baseProps()} onSubmit={onSubmit} />)

		await user.click(screen.getByRole('button', { name: /Créer l’absence/ }))
		expect(onSubmit).toHaveBeenCalledTimes(1)
	})

	it('reporte les changements de champ via onChange', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(<AbsenceFormSheet {...baseProps()} onChange={onChange} />)

		await user.type(screen.getByLabelText('Note'), 'x')
		expect(onChange).toHaveBeenCalled()
	})
})

describe('AbsenceFormSheet — édition', () => {
	function editProps() {
		return {
			...baseProps(),
			mode: 'edit' as const,
			values: { ...emptyAbsenceDraft('emp-1', '2026-08-10') },
		}
	}

	it("verrouille l'employé plutôt que de proposer un sélecteur", () => {
		render(<AbsenceFormSheet {...editProps()} />)

		expect(screen.getByText('Modifier l’absence')).toBeDefined()
		expect(screen.queryByRole('combobox', { name: 'Employé' })).toBeNull()
		expect(
			within(screen.getByRole('dialog')).getByText('Alix Martin'),
		).toBeDefined()
	})

	it('affiche le bouton supprimer et le déclenche', async () => {
		const user = userEvent.setup()
		const onDelete = vi.fn()
		render(<AbsenceFormSheet {...editProps()} onDelete={onDelete} />)

		await user.click(screen.getByRole('button', { name: /Supprimer/ }))
		expect(onDelete).toHaveBeenCalledTimes(1)
	})

	it("affiche l'erreur de sauvegarde renvoyée par la mutation", () => {
		render(<AbsenceFormSheet {...editProps()} saveError="HTTP 409: conflit" />)
		expect(screen.getByText('HTTP 409: conflit')).toBeDefined()
	})
})

describe('AbsenceFormSheet — pas d’appel réseau', () => {
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
		render(<AbsenceFormSheet {...baseProps()} />)
		expect(fetchSpy).not.toHaveBeenCalled()
	})
})

import { screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { emptyAbsenceDraft } from '#/pages/hr/lib/absences'
import { AbsenceFormSheet } from '#/pages/hr/ui/absence-form-sheet'
import { renderWithPermissions } from '#/test/with-permissions'

// jsdom doesn't implement ResizeObserver, and Radix `Select`'s listbox needs
// `scrollIntoView`/pointer-capture methods it also doesn't implement.
// Stubbed locally rather than in the shared `vitest.setup.ts`, which this
// workstream doesn't own.
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

const MEMBERS = [
	{ memberId: 'member-1', displayName: 'Martin Alix' },
	{ memberId: 'member-2', displayName: 'Leroy Marie' },
]

function baseProps() {
	return {
		open: true,
		mode: 'create' as const,
		values: emptyAbsenceDraft('', '2026-08-10'),
		members: MEMBERS,
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

describe('AbsenceFormSheet — creation', () => {
	it('renders nothing when open is false', () => {
		renderWithPermissions(<AbsenceFormSheet {...baseProps()} open={false} />)
		expect(screen.queryByRole('dialog')).toBeNull()
	})

	it('offers the member picker and no delete button', () => {
		renderWithPermissions(<AbsenceFormSheet {...baseProps()} />)

		expect(screen.getByText('Nouvelle absence')).toBeDefined()
		expect(screen.getByRole('combobox', { name: 'Personne' })).toBeDefined()
		expect(screen.queryByRole('button', { name: /Supprimer/ })).toBeNull()
	})

	it('shows « {nom} {prénom} » in the member picker options', async () => {
		const user = userEvent.setup()
		renderWithPermissions(<AbsenceFormSheet {...baseProps()} />)

		await user.click(screen.getByRole('combobox', { name: 'Personne' }))
		expect(
			await screen.findByRole('option', { name: 'Martin Alix' }),
		).toBeDefined()
		expect(screen.getByRole('option', { name: 'Leroy Marie' })).toBeDefined()
	})

	it('shows the surname alone in the picker when the given name is missing', async () => {
		const user = userEvent.setup()
		renderWithPermissions(
			<AbsenceFormSheet
				{...baseProps()}
				members={[{ memberId: 'member-3', displayName: 'Petit' }]}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Personne' }))
		expect(await screen.findByRole('option', { name: 'Petit' })).toBeDefined()
	})

	it('hides the time fields when full day is on', () => {
		renderWithPermissions(<AbsenceFormSheet {...baseProps()} />)
		expect(screen.queryByLabelText('Heure de début')).toBeNull()
	})

	it('shows the time fields when full day is off', () => {
		const props = baseProps()
		renderWithPermissions(
			<AbsenceFormSheet
				{...props}
				values={{ ...props.values, allDay: false }}
			/>,
		)
		expect(screen.getByLabelText('Heure de début')).toBeDefined()
		expect(screen.getByLabelText('Heure de fin')).toBeDefined()
	})

	it('shows validation errors and disables submission', async () => {
		await renderWithPermissions(
			<AbsenceFormSheet {...baseProps()} errors={['Personne requise']} />,
		)

		expect(screen.getByText('Personne requise')).toBeDefined()
		const submit = screen.getByRole('button', {
			name: /Créer l’absence/,
		}) as HTMLButtonElement
		expect(submit.disabled).toBe(true)
	})

	it('calls onSubmit on save click when valid', async () => {
		const user = userEvent.setup()
		const onSubmit = vi.fn()
		await renderWithPermissions(
			<AbsenceFormSheet {...baseProps()} onSubmit={onSubmit} />,
		)

		await user.click(screen.getByRole('button', { name: /Créer l’absence/ }))
		expect(onSubmit).toHaveBeenCalledTimes(1)
	})

	it('reports field changes through onChange', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		renderWithPermissions(
			<AbsenceFormSheet {...baseProps()} onChange={onChange} />,
		)

		await user.type(screen.getByLabelText('Note'), 'x')
		expect(onChange).toHaveBeenCalled()
	})
})

describe('AbsenceFormSheet — champ de plage', () => {
	it('shows a single day when the range is one day long', () => {
		renderWithPermissions(<AbsenceFormSheet {...baseProps()} />)

		expect(screen.getByTestId('absence-range-trigger').textContent).toBe(
			'10/08/2026',
		)
	})

	it('shows both bounds separated by a dash when the range spans several days', () => {
		const props = baseProps()
		renderWithPermissions(
			<AbsenceFormSheet
				{...props}
				values={{
					...props.values,
					range: { from: '2026-08-10', to: '2026-08-12' },
				}}
			/>,
		)

		expect(screen.getByTestId('absence-range-trigger').textContent).toBe(
			'10/08/2026 – 12/08/2026',
		)
	})

	it('opens a calendar when the period trigger is clicked', async () => {
		const user = userEvent.setup()
		renderWithPermissions(<AbsenceFormSheet {...baseProps()} />)

		await user.click(screen.getByTestId('absence-range-trigger'))
		expect(screen.getByRole('grid')).toBeDefined()
	})
})

describe('AbsenceFormSheet — editing', () => {
	function editProps() {
		return {
			...baseProps(),
			mode: 'edit' as const,
			values: { ...emptyAbsenceDraft('member-1', '2026-08-10') },
		}
	}

	it('locks the member instead of offering a picker', () => {
		renderWithPermissions(<AbsenceFormSheet {...editProps()} />)

		expect(screen.getByText('Modifier l’absence')).toBeDefined()
		expect(screen.queryByRole('combobox', { name: 'Personne' })).toBeNull()
		expect(
			within(screen.getByRole('dialog')).getByText('Martin Alix'),
		).toBeDefined()
	})

	it('shows the delete button and fires it', async () => {
		const user = userEvent.setup()
		const onDelete = vi.fn()
		await renderWithPermissions(
			<AbsenceFormSheet {...editProps()} onDelete={onDelete} />,
		)

		await user.click(screen.getByRole('button', { name: /Supprimer/ }))
		expect(onDelete).toHaveBeenCalledTimes(1)
	})

	it('shows the save error returned by the mutation', () => {
		renderWithPermissions(
			<AbsenceFormSheet {...editProps()} saveError="HTTP 409: conflit" />,
		)
		expect(screen.getByText('HTTP 409: conflit')).toBeDefined()
	})
})

describe('AbsenceFormSheet — no network call', () => {
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
		renderWithPermissions(<AbsenceFormSheet {...baseProps()} />)
		expect(fetchSpy).not.toHaveBeenCalled()
	})
})

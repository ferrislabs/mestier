import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { Member } from '#/hooks/use-reference-catalog'
import type { Rhythm } from '#/hooks/use-work-time'
import { emptyAbsenceDraft } from '#/pages/hr/lib/absences'
import {
	EmployeeWorkTimeUI,
	type EmployeeWorkTimeUIProps,
} from '#/pages/hr/ui/employee-work-time-ui'

// jsdom doesn't implement ResizeObserver; the absence sheet's `Select` (via
// Radix) measures its trigger with it. Stubbed locally, see
// `absence-form-sheet.test.tsx` for the same workaround.
class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
// biome-ignore lint/suspicious/noExplicitAny: test-only global polyfill
;(globalThis as any).ResizeObserver ??= ResizeObserverStub

function member(overrides: Partial<Member> = {}): Member {
	return {
		id: 'member-1',
		organization_id: 'org-1',
		last_name: 'Martin',
		first_name: 'Alix',
		display_name: 'Martin Alix',
		account: null,
		joined_at: null,
		created_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

function baseProps(
	overrides: Partial<EmployeeWorkTimeUIProps> = {},
): EmployeeWorkTimeUIProps {
	return {
		member: member(),
		hourlyRateCents: 1500,
		isSalaried: false,
		monthlyCostCents: null,
		effectiveHourlyRateCents: 1500,
		openCostBasisEffectiveFrom: '2026-01-01',
		weeklyGap: {
			plannedMinutes: 1920,
			contractMinutes: 2100,
			deltaMinutes: -180,
		},
		contractForm: {
			value: '35h00',
			isPending: false,
			error: null,
			onChange: vi.fn(),
			onSubmit: vi.fn(),
		},
		costHistorySection: {
			history: [],
			isLoading: false,
			form: {
				values: {
					effectiveFrom: '2026-08-01',
					isSalaried: false,
					hourlyRate: '15,00',
					monthlyCost: '',
				},
				errors: [],
				isSaving: false,
				saveError: null,
				onChange: vi.fn(),
				onSubmit: vi.fn(),
			},
		},
		rhythmSection: {
			values: {
				effectiveFrom: '2026-01-01',
				effectiveTo: '',
				slots: [
					{ key: 'slot-1', weekday: 1, startTime: '08:00', endTime: '16:00' },
				],
			},
			otherRhythms: [],
			openRhythmEffectiveFrom: '2026-01-01',
			errors: [],
			isLoading: false,
			isSaving: false,
			saveError: null,
			onEffectiveFromChange: vi.fn(),
			onEffectiveToChange: vi.fn(),
			onSlotChange: vi.fn(),
			onAddSlot: vi.fn(),
			onRemoveSlot: vi.fn(),
			onSubmit: vi.fn(),
		},
		workSlotsSection: {
			values: {
				from: '2026-08-03',
				to: '2026-08-10',
				slots: [
					{
						key: 'work-slot-1',
						workDate: '2026-08-05',
						startTime: '08:00',
						endTime: '12:00',
					},
				],
			},
			errors: [],
			isLoading: false,
			isSaving: false,
			saveError: null,
			onFromChange: vi.fn(),
			onToChange: vi.fn(),
			onSlotChange: vi.fn(),
			onAddSlot: vi.fn(),
			onRemoveSlot: vi.fn(),
			onSubmit: vi.fn(),
		},
		absencesSection: {
			absences: [],
			isLoading: false,
			onCreate: vi.fn(),
			onSelect: vi.fn(),
		},
		absenceSheet: {
			open: false,
			mode: 'create',
			values: emptyAbsenceDraft('member-1', '2026-08-10'),
			members: [],
			errors: [],
			isSaving: false,
			isDeleting: false,
			saveError: null,
			onChange: vi.fn(),
			onSubmit: vi.fn(),
			onOpenChange: vi.fn(),
		},
		...overrides,
	}
}

describe('EmployeeWorkTimeUI — name rendering', () => {
	it('shows « {nom} {prénom} » in the title', () => {
		render(<EmployeeWorkTimeUI {...baseProps()} />)

		expect(screen.getByText('Martin Alix')).toBeDefined()
	})

	it('shows the surname alone, with no stray space, when the given name is missing', () => {
		render(
			<EmployeeWorkTimeUI
				{...baseProps({
					member: member({ first_name: null, display_name: 'Martin' }),
				})}
			/>,
		)

		expect(screen.getByText('Martin')).toBeDefined()
	})
})

describe('EmployeeWorkTimeUI', () => {
	it('shows the hourly rate when it is filled in', () => {
		render(<EmployeeWorkTimeUI {...baseProps()} />)

		expect(screen.getByText(/15,00\s*€/)).toBeDefined()
	})

	it('shows « Non renseigné » when the hourly rate is null, never an amount of 0', () => {
		render(<EmployeeWorkTimeUI {...baseProps({ hourlyRateCents: null })} />)

		expect(screen.getByText('Non renseigné')).toBeDefined()
		expect(screen.queryByText(/0,00\s*€/)).toBeNull()
	})

	it('shows the gap between planned hours and the contractual baseline', () => {
		render(<EmployeeWorkTimeUI {...baseProps()} />)

		expect(screen.getByText(/32h00/)).toBeDefined()
		expect(screen.getByText(/35h00/)).toBeDefined()
		expect(screen.getByText(/-3h00/)).toBeDefined()
	})

	it('shows a surplus with a positive sign', () => {
		render(
			<EmployeeWorkTimeUI
				{...baseProps({
					weeklyGap: {
						plannedMinutes: 2400,
						contractMinutes: 2100,
						deltaMinutes: 300,
					},
				})}
			/>,
		)

		expect(screen.getByText(/\+5h00/)).toBeDefined()
	})

	it('allows editing the contractual baseline and calls onSubmit', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<EmployeeWorkTimeUI {...props} />)

		await user.click(
			screen.getByRole('button', { name: 'Enregistrer la base contractuelle' }),
		)

		expect(props.contractForm.onSubmit).toHaveBeenCalledTimes(1)
	})

	it('calls onAddSlot with the right weekday', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<EmployeeWorkTimeUI {...props} />)

		const mardiSection = screen.getByRole('region', { name: 'Mardi' })
		await user.click(
			within(mardiSection).getByRole('button', { name: 'Ajouter un créneau' }),
		)

		expect(props.rhythmSection.onAddSlot).toHaveBeenCalledWith(2)
	})

	it("calls onSlotChange when a slot's start time is edited", async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<EmployeeWorkTimeUI {...props} />)

		const lundiSection = screen.getByRole('region', { name: 'Lundi' })
		const startInput = within(lundiSection).getByLabelText('Début')
		await user.clear(startInput)
		await user.type(startInput, '09:00')

		expect(props.rhythmSection.onSlotChange).toHaveBeenCalledWith(
			'slot-1',
			expect.objectContaining({ startTime: expect.any(String) }),
		)
	})

	it("calls onRemoveSlot when the slot's delete button is clicked", async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<EmployeeWorkTimeUI {...props} />)

		const lundiSection = screen.getByRole('region', { name: 'Lundi' })
		await user.click(
			within(lundiSection).getByRole('button', {
				name: 'Supprimer le créneau',
			}),
		)

		expect(props.rhythmSection.onRemoveSlot).toHaveBeenCalledWith('slot-1')
	})

	it("calls the rhythm's onSubmit when Enregistrer le rythme is clicked", async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<EmployeeWorkTimeUI {...props} />)

		await user.click(
			screen.getByRole('button', { name: 'Enregistrer le rythme' }),
		)

		expect(props.rhythmSection.onSubmit).toHaveBeenCalledTimes(1)
	})

	it('disables saving the rhythm when there are validation errors', () => {
		const props = baseProps({
			rhythmSection: {
				...baseProps().rhythmSection,
				errors: [{ key: 'slot-1', message: 'La fin doit être après le début' }],
			},
		})
		render(<EmployeeWorkTimeUI {...props} />)

		const submitButton = screen.getByRole('button', {
			name: 'Enregistrer le rythme',
		}) as HTMLButtonElement
		expect(submitButton.disabled).toBe(true)
		expect(screen.getByText('La fin doit être après le début')).toBeDefined()
	})

	it('shows a clear conflict message (409) rather than a raw error', () => {
		const props = baseProps({
			rhythmSection: {
				...baseProps().rhythmSection,
				saveError:
					'Impossible de démarrer cette version avant celle en cours (01/01/2026).',
			},
		})
		render(<EmployeeWorkTimeUI {...props} />)

		expect(
			screen.getByText(
				/Impossible de démarrer cette version avant celle en cours/,
			),
		).toBeDefined()
	})

	it('shows the existing work ranges', () => {
		render(<EmployeeWorkTimeUI {...baseProps()} />)

		const dateInput = screen.getByLabelText(
			'Date (plage 1)',
		) as HTMLInputElement
		expect(dateInput.value).toBe('2026-08-05')
	})

	it("calls the work ranges' onAddSlot when Ajouter une plage is clicked", async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<EmployeeWorkTimeUI {...props} />)

		await user.click(screen.getByRole('button', { name: 'Ajouter une plage' }))

		expect(props.workSlotsSection.onAddSlot).toHaveBeenCalledTimes(1)
	})

	it("calls the ranges' onSubmit when Enregistrer les plages is clicked", async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<EmployeeWorkTimeUI {...props} />)

		await user.click(
			screen.getByRole('button', { name: 'Enregistrer les plages' }),
		)

		expect(props.workSlotsSection.onSubmit).toHaveBeenCalledTimes(1)
	})

	it('shows the history of previous rhythm versions', () => {
		const otherRhythms: Rhythm[] = [
			{
				id: 'rhythm-old',
				organization_id: 'org-1',
				employee_id: 'employee-1',
				effective_from: '2025-01-01',
				effective_to: '2026-01-01',
				slots: [],
				created_at: '2025-01-01T00:00:00Z',
				updated_at: '2025-01-01T00:00:00Z',
			},
		]
		render(
			<EmployeeWorkTimeUI
				{...baseProps({
					rhythmSection: { ...baseProps().rhythmSection, otherRhythms },
				})}
			/>,
		)

		expect(screen.getByText(/01\/01\/2025/)).toBeDefined()
	})

	it('shows a loading state for the rhythm without crashing', () => {
		render(
			<EmployeeWorkTimeUI
				{...baseProps({
					rhythmSection: { ...baseProps().rhythmSection, isLoading: true },
				})}
			/>,
		)

		expect(screen.getByText(/Chargement/)).toBeDefined()
	})
})

describe('EmployeeWorkTimeUI — absences', () => {
	it("renders the employee's absences section", () => {
		render(
			<EmployeeWorkTimeUI
				{...baseProps({
					absencesSection: {
						absences: [
							{
								id: 'ab-1',
								memberId: 'member-1',
								kind: 'LEAVE',
								allDay: true,
								range: { from: '2026-08-10', to: '2026-08-10' },
								startTime: '08:00',
								endTime: '18:00',
								note: '',
							},
						],
						isLoading: false,
						onCreate: vi.fn(),
						onSelect: vi.fn(),
					},
				})}
			/>,
		)

		expect(screen.getByText(/Congé — 10\/08\/2026/)).toBeDefined()
	})

	it('does not show the absence form when the sheet is closed', () => {
		render(<EmployeeWorkTimeUI {...baseProps()} />)
		expect(screen.queryByRole('dialog')).toBeNull()
	})

	it('shows the absence form when absenceSheet.open is true', () => {
		render(
			<EmployeeWorkTimeUI
				{...baseProps({
					absenceSheet: { ...baseProps().absenceSheet, open: true },
				})}
			/>,
		)

		expect(screen.getByRole('dialog')).toBeDefined()
	})
})

describe('EmployeeWorkTimeUI — no network call', () => {
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

	it('fires no fetch on render nor during interactions', async () => {
		const user = userEvent.setup()
		render(<EmployeeWorkTimeUI {...baseProps()} />)

		await user.click(
			screen.getByRole('button', { name: 'Enregistrer la base contractuelle' }),
		)
		await user.click(
			screen.getByRole('button', { name: 'Enregistrer le rythme' }),
		)
		await user.click(
			screen.getByRole('button', { name: 'Enregistrer les plages' }),
		)

		expect(fetchSpy).not.toHaveBeenCalled()
	})
})

describe('EmployeeWorkTimeUI — coût employeur', () => {
	/**
	 * This page is where the team list sends somebody whose salary cannot be
	 * divided yet. It used to greet them with "Taux horaire : Non renseigné",
	 * repeating the misreading they came here to fix.
	 */
	it('shows a salaried employee their monthly cost, not a missing hourly rate', async () => {
		render(
			<EmployeeWorkTimeUI
				{...baseProps()}
				hourlyRateCents={null}
				isSalaried
				monthlyCostCents={230_000}
				effectiveHourlyRateCents={null}
			/>,
		)

		expect(screen.getByText('Coût employeur')).toBeDefined()
		expect(screen.getByText(/2 300,00/)).toBeDefined()
		expect(screen.queryByText('Taux horaire')).toBeNull()
	})

	it('shows the hourly equivalent once the contract can divide the salary', async () => {
		render(
			<EmployeeWorkTimeUI
				{...baseProps()}
				hourlyRateCents={null}
				isSalaried
				monthlyCostCents={230_000}
				effectiveHourlyRateCents={1_517}
			/>,
		)

		expect(screen.getByText(/soit/)).toBeDefined()
		expect(screen.getByText(/15,17/)).toBeDefined()
	})
})

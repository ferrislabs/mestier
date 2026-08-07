import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { Employee } from '#/hooks/use-reference-catalog'
import type { Rhythm } from '#/hooks/use-work-time'
import {
	EmployeeWorkTimeUI,
	type EmployeeWorkTimeUIProps,
} from '#/pages/hr/ui/employee-work-time-ui'

function employee(overrides: Partial<Employee> = {}): Employee {
	return {
		id: 'employee-1',
		organization_id: 'org-1',
		name: 'Alix Martin',
		hourly_rate_cents: 1500,
		user_id: null,
		weekly_contract_minutes: 2100,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

function baseProps(
	overrides: Partial<EmployeeWorkTimeUIProps> = {},
): EmployeeWorkTimeUIProps {
	return {
		organizationName: 'Atelier Bois & Co',
		employee: employee(),
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
		...overrides,
	}
}

describe('EmployeeWorkTimeUI', () => {
	it('affiche le taux horaire quand il est renseigné', () => {
		render(<EmployeeWorkTimeUI {...baseProps()} />)

		expect(screen.getByText(/15,00\s*€/)).toBeDefined()
	})

	it('affiche « Non renseigné » quand le taux horaire est null, jamais un montant à 0', () => {
		render(
			<EmployeeWorkTimeUI
				{...baseProps({ employee: employee({ hourly_rate_cents: null }) })}
			/>,
		)

		expect(screen.getByText('Non renseigné')).toBeDefined()
		expect(screen.queryByText(/0,00\s*€/)).toBeNull()
	})

	it("affiche l'écart entre heures planifiées et base contractuelle", () => {
		render(<EmployeeWorkTimeUI {...baseProps()} />)

		expect(screen.getByText(/32h00/)).toBeDefined()
		expect(screen.getByText(/35h00/)).toBeDefined()
		expect(screen.getByText(/-3h00/)).toBeDefined()
	})

	it('affiche un surplus avec un signe positif', () => {
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

	it('permet de modifier la base contractuelle et appelle onSubmit', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<EmployeeWorkTimeUI {...props} />)

		await user.click(
			screen.getByRole('button', { name: 'Enregistrer la base contractuelle' }),
		)

		expect(props.contractForm.onSubmit).toHaveBeenCalledTimes(1)
	})

	it('appelle onAddSlot avec le bon jour de semaine', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<EmployeeWorkTimeUI {...props} />)

		const mardiSection = screen.getByRole('region', { name: 'Mardi' })
		await user.click(
			within(mardiSection).getByRole('button', { name: 'Ajouter un créneau' }),
		)

		expect(props.rhythmSection.onAddSlot).toHaveBeenCalledWith(2)
	})

	it("appelle onSlotChange quand on modifie l'heure de début d'un créneau", async () => {
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

	it('appelle onRemoveSlot au clic sur le bouton de suppression du créneau', async () => {
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

	it('appelle onSubmit du rythme au clic sur Enregistrer le rythme', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<EmployeeWorkTimeUI {...props} />)

		await user.click(
			screen.getByRole('button', { name: 'Enregistrer le rythme' }),
		)

		expect(props.rhythmSection.onSubmit).toHaveBeenCalledTimes(1)
	})

	it('désactive l’enregistrement du rythme quand il y a des erreurs de validation', () => {
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

	it("affiche un message de conflit clair (409) plutôt qu'une erreur brute", () => {
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

	it('affiche les plages de travail existantes', () => {
		render(<EmployeeWorkTimeUI {...baseProps()} />)

		const dateInput = screen.getByLabelText(
			'Date (plage 1)',
		) as HTMLInputElement
		expect(dateInput.value).toBe('2026-08-05')
	})

	it('appelle onAddSlot des plages de travail au clic sur Ajouter une plage', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<EmployeeWorkTimeUI {...props} />)

		await user.click(screen.getByRole('button', { name: 'Ajouter une plage' }))

		expect(props.workSlotsSection.onAddSlot).toHaveBeenCalledTimes(1)
	})

	it('appelle onSubmit des plages au clic sur Enregistrer les plages', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<EmployeeWorkTimeUI {...props} />)

		await user.click(
			screen.getByRole('button', { name: 'Enregistrer les plages' }),
		)

		expect(props.workSlotsSection.onSubmit).toHaveBeenCalledTimes(1)
	})

	it('affiche l’historique des versions de rythme précédentes', () => {
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

	it('affiche un état de chargement pour le rythme sans planter', () => {
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

describe('EmployeeWorkTimeUI — aucun appel réseau', () => {
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

	it('ne déclenche aucun fetch au rendu ni pendant les interactions', async () => {
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

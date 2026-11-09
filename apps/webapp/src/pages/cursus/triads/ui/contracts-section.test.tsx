import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Contract } from '#/pages/cursus/triads/feature/use-triads'
import { ContractsSection } from '#/pages/cursus/triads/ui/contracts-section'

function contract(overrides: Partial<Contract> = {}): Contract {
	return {
		id: 'contract-1',
		organization_id: 'org-1',
		triad_id: 'triad-1',
		host_company_id: 'host-1',
		host_company_name: 'Atelier Bois',
		kind: 'APPRENTICESHIP',
		number: 'C-42',
		start_date: '2026-09-01',
		end_date: '2028-08-31',
		duration_months: 24,
		comment: 'Visite **annuelle**',
		has_mission_sheet_file: true,
		tutors: [
			{
				company_tutor_id: 'tutor-1',
				participant_id: 'p-1',
				name: 'Jo Durand',
				is_main: true,
			},
			{
				company_tutor_id: 'tutor-2',
				participant_id: 'p-2',
				name: 'Léa Martin',
				is_main: false,
			},
		],
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

function props(contracts: Contract[], canManage: boolean) {
	return {
		contracts,
		canManage,
		canAdd: true,
		isUploading: false,
		fileError: null,
		onAdd: vi.fn(),
		onEdit: vi.fn(),
		onTutors: vi.fn(),
		onClose: vi.fn(),
		onOpenFile: vi.fn(),
		onUploadFile: vi.fn(),
	}
}

describe('ContractsSection', () => {
	it('shows company, kind, number, dates and tutor of a contract', () => {
		render(<ContractsSection {...props([contract()], true)} />)

		expect(screen.getByText('Atelier Bois')).toBeDefined()
		expect(screen.getByText('Apprentissage')).toBeDefined()
		expect(screen.getByText(/n° C-42/)).toBeDefined()
		expect(screen.getByText(/01\/09\/2026 → 31\/08\/2028/)).toBeDefined()
		expect(screen.getByText('Jo Durand')).toBeDefined()
		expect(screen.getByText('Léa Martin')).toBeDefined()
		expect(screen.getAllByText('Principal')).toHaveLength(1)
		expect(screen.getByText('24 mois')).toBeDefined()
		expect(screen.getByText('annuelle')).toBeDefined()
	})

	it('offers edit, tutors, close and add on an open contract with the bit', async () => {
		const user = userEvent.setup()
		const p = props([contract()], true)
		render(<ContractsSection {...p} />)

		await user.click(screen.getByRole('button', { name: /Ajouter un contrat/ }))
		await user.click(screen.getByRole('button', { name: /Modifier/ }))
		await user.click(screen.getByRole('button', { name: /Tuteurs/ }))
		await user.click(
			screen.getByRole('button', { name: /Clôturer le contrat/ }),
		)

		expect(p.onAdd).toHaveBeenCalledOnce()
		expect(p.onEdit).toHaveBeenCalledOnce()
		expect(p.onTutors).toHaveBeenCalledOnce()
		expect(p.onClose).toHaveBeenCalledOnce()
	})

	it('opens an attached file', async () => {
		const user = userEvent.setup()
		const p = props([contract()], false)
		render(<ContractsSection {...p} />)

		await user.click(
			screen.getByRole('button', { name: 'Ouvrir Fiche mission' }),
		)

		expect(p.onOpenFile).toHaveBeenCalledWith(
			expect.anything(),
			'mission_sheet',
		)
	})

	it('hides every write button without the triads bit', () => {
		render(<ContractsSection {...props([contract()], false)} />)

		expect(
			screen.queryByRole('button', { name: /Ajouter un contrat/ }),
		).toBeNull()
		expect(screen.queryByRole('button', { name: /Modifier/ })).toBeNull()
		expect(screen.queryByRole('button', { name: /Tuteurs/ })).toBeNull()
		expect(screen.queryByRole('button', { name: /Clôturer/ })).toBeNull()
		expect(screen.queryByRole('button', { name: /Téléverser/ })).toBeNull()
	})

	it('shows closure info and no edit button on a closed contract', () => {
		render(
			<ContractsSection
				{...props(
					[
						contract({
							closed_at: '2027-03-01',
							closure_reason_label: 'Rupture',
							closure_note: 'Départ de l’entreprise',
						}),
					],
					true,
				)}
			/>,
		)

		expect(screen.getByText(/Clôturé le 01\/03\/2027 · Rupture/)).toBeDefined()
		expect(screen.getByText('Départ de l’entreprise')).toBeDefined()
		expect(screen.queryByRole('button', { name: /Modifier/ })).toBeNull()
		expect(screen.queryByRole('button', { name: /Tuteurs/ })).toBeNull()
		expect(
			screen.queryByRole('button', { name: /Clôturer le contrat/ }),
		).toBeNull()
		expect(screen.queryByRole('button', { name: /Remplacer/ })).toBeNull()
	})
})

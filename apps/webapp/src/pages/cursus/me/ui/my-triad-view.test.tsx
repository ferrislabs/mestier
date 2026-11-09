import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type { MyTriadDetail } from '#/pages/cursus/me/feature/use-my-cursus'
import { MyTriadView } from '#/pages/cursus/me/ui/my-triad-view'

const TRIAD: MyTriadDetail = {
	triad_id: 'triad-1',
	access: { kind: 'active' },
	role: 'apprentice',
	apprentice_name: 'Lea Martin',
	apprentice_email: 'lea@example.fr',
	promotion_id: 'promo-1',
	promotion_label: 'CAP 2026',
	promotion_start_year: 2026,
	promotion_end_year: 2028,
	training_name: 'CAP Menuisier',
	pedagogic_tutor_name: 'Alix Martin',
	contracts: [
		{
			id: 'contract-1',
			host_company_name: 'Atelier Bois',
			kind: 'PROFESSIONALIZATION',
			start_date: '2026-09-01',
			end_date: '2027-08-31',
			duration_months: 12,
			comment: 'Port des **EPI**',
			closed_at: '2027-02-01',
			closure_reason_label: 'Rupture',
			closure_note: 'Accord des parties',
			tutors: [
				{ name: 'Jo Durand', is_main: true },
				{ name: 'Mia Roche', is_main: false },
			],
		},
	],
}

describe('MyTriadView', () => {
	it('shows promotion, pedagogic tutor, contract and its closure', () => {
		render(<MyTriadView triad={TRIAD} onBack={vi.fn()} />)

		expect(screen.getByText('Lea Martin')).toBeDefined()
		expect(screen.getByText('Alix Martin')).toBeDefined()
		expect(screen.getByText('Atelier Bois')).toBeDefined()
		expect(screen.getByText('Professionnalisation')).toBeDefined()
		expect(screen.getByText('Jo Durand')).toBeDefined()
		expect(screen.getByText('Mia Roche')).toBeDefined()
		expect(screen.getByText('Principal')).toBeDefined()
		expect(screen.getByText('12 mois')).toBeDefined()
		expect(screen.getByText('EPI')).toBeDefined()
		expect(screen.getByText('Accord des parties')).toBeDefined()
		expect(screen.getByText(/Rupture/)).toBeDefined()
	})
})

import { describe, expect, it } from 'vitest'
import {
	type ContractDraft,
	contractIssues,
	contractToDraft,
	duplicateLabel,
	emptyContractDraft,
	hasIssues,
	toContractBody,
	toCreateRequest,
} from '#/pages/maintenance/lib/contract-form'
import type { MaintenanceContract } from '#/pages/maintenance/types'

function filled(overrides: Partial<ContractDraft> = {}): ContractDraft {
	const draft = emptyContractDraft(2026)
	draft.lines[0].label = 'Tonte avec ramassage'

	return {
		...draft,
		customerId: 'customer-1',
		customerContextId: 'context-1',
		...overrides,
	}
}

describe('emptyContractDraft', () => {
	it('opens on the paper sheet season, with one line to fill', () => {
		const draft = emptyContractDraft(2026)

		expect([draft.year, draft.firstWeek, draft.lastWeek]).toEqual([
			'2026',
			'12',
			'48',
		])
		expect(draft.lines).toHaveLength(1)
		expect(draft.status).toBe('ACTIVE')
	})
})

describe('contractIssues', () => {
	it('has nothing to say about a complete draft', () => {
		expect(hasIssues(contractIssues(filled()))).toBe(false)
	})

	it('asks for the customer, then the site, when creating', () => {
		expect(contractIssues(filled({ customerId: '' })).customer).toBe(
			'Choisissez un client.',
		)
		expect(contractIssues(filled({ customerContextId: '' })).site).toBe(
			'Choisissez un site.',
		)
	})

	it('does not check the customer and the site of an existing contract', () => {
		const issues = contractIssues(
			filled({ id: 'contract-1', customerId: '', customerContextId: '' }),
		)

		expect(hasIssues(issues)).toBe(false)
	})

	it('refuses a last week before the first, and a week 53 the year lacks', () => {
		expect(
			contractIssues(filled({ firstWeek: '30', lastWeek: '20' })).season,
		).toBe('La dernière semaine ne peut pas précéder la première.')
		expect(
			contractIssues(filled({ year: '2027', lastWeek: '53' })).season,
		).toBe("2027 n'a que 52 semaines.")
		expect(
			contractIssues(filled({ year: '2026', lastWeek: '53' })).season,
		).toBeUndefined()
	})

	it('names the line whose cadence is wrong', () => {
		const draft = filled()
		draft.lines[0].cadence = {
			kind: 'WINDOWS',
			everyDays: '14',
			windows: [{ from: '5', to: '8' }],
		}

		const issues = contractIssues(draft)

		expect(issues.lines[draft.lines[0].key]).toBe(
			'La fenêtre S5 à S8 sort de la saison (S12 à S48).',
		)
	})

	it('checks the windows against the season typed on the form', () => {
		const draft = filled({ firstWeek: '5' })
		draft.lines[0].cadence = {
			kind: 'WINDOWS',
			everyDays: '14',
			windows: [{ from: '5', to: '8' }],
		}

		expect(hasIssues(contractIssues(draft))).toBe(false)
	})

	it('wants a name for every line and whole minutes', () => {
		const draft = filled()
		draft.lines[0].label = '  '
		expect(contractIssues(draft).lines[draft.lines[0].key]).toBe(
			'Donnez un nom à la prestation.',
		)

		draft.lines[0].label = 'Taille'
		draft.lines[0].minutes = '1h30'
		expect(contractIssues(draft).lines[draft.lines[0].key]).toBe(
			'La durée est un nombre de minutes.',
		)
	})

	it('refuses an amount that is not money', () => {
		expect(contractIssues(filled({ amount: '12 euros' })).amount).toBe(
			'Un montant en euros, par exemple 1 250,00.',
		)
		expect(
			contractIssues(filled({ amount: '1 250,50' })).amount,
		).toBeUndefined()
	})
})

describe('toCreateRequest', () => {
	it('builds the body the API expects', () => {
		const draft = filled({
			reference: ' C25029 ',
			managerMemberId: 'member-1',
			amount: '1 250,50',
			notes: '',
		})
		draft.lines[0].minutes = '90'

		expect(toCreateRequest(draft)).toEqual({
			customer_id: 'customer-1',
			customer_context_id: 'context-1',
			reference: 'C25029',
			season: { year: 2026, first_week: 12, last_week: 48 },
			manager_member_id: 'member-1',
			amount_cents: 125050,
			quote_id: null,
			notes: null,
			status: 'ACTIVE',
			lines: [
				{
					id: null,
					label: 'Tonte avec ramassage',
					cadence: { kind: 'ROLLING', every_days: 14 },
					minutes_per_intervention: 90,
				},
			],
		})
	})
})

describe('contractToDraft and toContractBody', () => {
	const contract: MaintenanceContract = {
		id: 'contract-1',
		customer_id: 'customer-1',
		customer_context_id: 'context-1',
		project_id: 'project-1',
		reference: 'C25029',
		season: { year: 2026, first_week: 12, last_week: 48 },
		manager_member_id: 'member-1',
		amount_cents: 125050,
		quote_id: 'quote-1',
		notes: 'Portail à gauche',
		status: 'DRAFT',
		lines: [
			{
				id: 'line-1',
				label: 'Taille',
				cadence: { kind: 'WINDOWS', windows: [{ from_week: 14, to_week: 17 }] },
				minutes_per_intervention: 120,
			},
		],
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
	}

	it('sends back what the form does not show, since a PUT replaces it all', () => {
		const body = toContractBody(contractToDraft(contract))

		expect(body.quote_id).toBe('quote-1')
		expect(body.status).toBe('DRAFT')
		expect(body.lines[0].id).toBe('line-1')
		expect(body.amount_cents).toBe(125050)
		expect(body.lines[0].cadence).toEqual({
			kind: 'WINDOWS',
			windows: [{ from_week: 14, to_week: 17 }],
		})
	})
})

describe('duplicateLabel', () => {
	it('names the next year', () => {
		expect(
			duplicateLabel({ season: { year: 2026, first_week: 1, last_week: 2 } }),
		).toBe('Dupliquer pour 2027')
	})
})

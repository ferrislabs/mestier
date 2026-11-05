import { describe, expect, it } from 'vitest'
import {
	canSaveFullForm,
	customerIdOfSelect,
	customerSelectValue,
	emptyFullForm,
	extrasOfForm,
	parseNewSearch,
	projectFormErrors,
	quoteIdOfSelect,
	quoteSelectValue,
} from '#/pages/projects/lib/project-form'

describe('project form', () => {
	it('asks for a name and a coherent period', () => {
		const form = {
			...emptyFullForm(null, ''),
			startDate: '2026-03-01',
			endDate: '2026-02-01',
		}
		const errors = projectFormErrors(form)

		expect(errors.byField.name).toBe('Le nom du projet est obligatoire.')
		expect(errors.byField.endDate).toBe('La fin ne peut pas précéder le début.')
	})

	it('trims the extras and drops empty dates', () => {
		const form = {
			...emptyFullForm('m1', ''),
			place: ' Vichy ',
			startDate: '2026-03-01',
		}

		expect(extrasOfForm(form)).toEqual({
			place: 'Vichy',
			description: '',
			startDate: '2026-03-01',
			endDate: null,
			ownerId: 'm1',
		})
	})

	it('maps the select sentinels to and from the real ids', () => {
		expect(customerSelectValue('')).toBe('interne')
		expect(customerIdOfSelect('interne')).toBe('')
		expect(quoteSelectValue('')).toBe('aucun')
		expect(quoteIdOfSelect('aucun')).toBe('')
		expect(quoteIdOfSelect('q1')).toBe('q1')
	})

	it('only saves a changed, valid form', () => {
		const stored = { ...emptyFullForm(null, ''), name: 'Jardin' }

		expect(canSaveFullForm(stored, stored)).toBe(false)
		expect(canSaveFullForm({ ...stored, name: 'Parc' }, stored)).toBe(true)
		expect(canSaveFullForm({ ...stored, name: ' ' }, stored)).toBe(false)
	})

	it('reads the customer to preselect from the search', () => {
		expect(parseNewSearch({ client: ' c1 ' })).toEqual({ client: 'c1' })
		expect(parseNewSearch({ client: ' ' })).toEqual({})
	})
})

import { describe, expect, it } from 'vitest'
import {
	buildTemplateTaskShapeRequest,
	dayOffsetLabel,
	emptyTemplateTaskDraft,
	type TemplateTaskDraft,
	templateTaskToDraft,
	validateTemplateTaskDraft,
	validateTemplateTaskHierarchy,
} from '#/pages/project-templates/lib/template-task-form'

function draft(overrides: Partial<TemplateTaskDraft> = {}): TemplateTaskDraft {
	return {
		...emptyTemplateTaskDraft(),
		title: 'Préparer le chantier',
		...overrides,
	}
}

describe('validateTemplateTaskDraft', () => {
	it('requires a title', () => {
		expect(validateTemplateTaskDraft(draft({ title: '  ' }))).toContain(
			'Titre requis',
		)
	})

	it('accepts an all-day shape with no times', () => {
		expect(validateTemplateTaskDraft(draft({ allDay: true }))).toEqual([])
	})

	it('refuses an end time before the start time', () => {
		const errors = validateTemplateTaskDraft(
			draft({ startTime: '10:00', endTime: '09:00' }),
		)
		expect(errors).toContain('La fin doit être après le début')
	})

	it('refuses an expense with no label', () => {
		const errors = validateTemplateTaskDraft(
			draft({ expensesEuros: '45', expensesLabel: '' }),
		)
		expect(errors).toContain('Un montant de frais doit être justifié')
	})

	it('accepts a zero-expense draft with no label', () => {
		expect(validateTemplateTaskDraft(draft({ expensesEuros: '' }))).toEqual([])
	})
})

describe('validateTemplateTaskHierarchy', () => {
	it('accepts a subtask pointing at a root', () => {
		const errors = validateTemplateTaskHierarchy([
			draft({ parentIndex: null }),
			draft({ parentIndex: 0 }),
		])
		expect(errors).toEqual([])
	})

	it('refuses a shape naming itself as its own parent', () => {
		const errors = validateTemplateTaskHierarchy([draft({ parentIndex: 0 })])
		expect(errors.length).toBeGreaterThan(0)
	})

	it('refuses a three-level hierarchy', () => {
		const errors = validateTemplateTaskHierarchy([
			draft({ parentIndex: null }),
			draft({ parentIndex: 0 }),
			draft({ parentIndex: 1 }),
		])
		expect(errors.length).toBeGreaterThan(0)
	})

	it('refuses a parentIndex naming nothing', () => {
		const errors = validateTemplateTaskHierarchy([draft({ parentIndex: 5 })])
		expect(errors.length).toBeGreaterThan(0)
	})
})

describe('buildTemplateTaskShapeRequest', () => {
	it('converts hours and minutes to a minute-of-day integer', () => {
		const shape = buildTemplateTaskShapeRequest(
			draft({ startTime: '08:00', endTime: '17:30' }),
		)
		expect(shape.starts_minute).toBe(480)
		expect(shape.ends_minute).toBe(1050)
	})

	it('carries no minutes for an all-day shape', () => {
		const shape = buildTemplateTaskShapeRequest(draft({ allDay: true }))
		expect(shape.starts_minute).toBeNull()
		expect(shape.ends_minute).toBeNull()
	})

	it('clears the expense label when the amount is zero', () => {
		const shape = buildTemplateTaskShapeRequest(
			draft({ expensesEuros: '', expensesLabel: 'stale label' }),
		)
		expect(shape.expenses_cents).toBe(0)
		expect(shape.expenses_label).toBeNull()
	})

	it('converts euros to cents', () => {
		const shape = buildTemplateTaskShapeRequest(
			draft({ expensesEuros: '45,50', expensesLabel: 'Location compacteur' }),
		)
		expect(shape.expenses_cents).toBe(4550)
	})
})

describe('templateTaskToDraft', () => {
	it('round-trips a persisted all-day shape back into a draft', () => {
		const seeded = templateTaskToDraft({
			id: 'shape-1',
			organization_id: 'org-1',
			template_id: 'template-1',
			title: 'Livraison matériel',
			description: null,
			day_offset: 2,
			starts_minute: null,
			ends_minute: null,
			all_day: true,
			blocks_availability: false,
			expenses_cents: 0,
			expenses_label: null,
			parent_index: 0,
			position: 1,
		})

		expect(seeded.title).toBe('Livraison matériel')
		expect(seeded.dayOffset).toBe(2)
		expect(seeded.allDay).toBe(true)
		expect(seeded.parentIndex).toBe(0)
	})
})

describe('dayOffsetLabel', () => {
	it('labels day zero as such', () => {
		expect(dayOffsetLabel(0)).toBe('Jour 0')
	})

	it('signs a positive offset', () => {
		expect(dayOffsetLabel(3)).toBe('Jour +3')
	})

	it('signs a negative offset', () => {
		expect(dayOffsetLabel(-1)).toBe('Jour -1')
	})
})

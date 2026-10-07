import { describe, expect, it } from 'vitest'
import {
	buildModelRequest,
	canSubmitModel,
	emptyModelDraft,
	modelToDraft,
	newModelSectionDraft,
} from '#/pages/doe-models/lib/model-form'

describe('model draft round trip', () => {
	it('keeps the settings of every kind', () => {
		const draft = modelToDraft(
			{
				id: 'm1',
				name: 'Paysagiste',
				description: null,
				sections: [
					{
						title: 'Végétaux',
						enabled: true,
						kind: { kind: 'supplies', maker_label: 'nursery' },
					},
					{
						title: 'Photos',
						enabled: false,
						kind: { kind: 'photos', per_page: 'four' },
					},
					{ title: 'Plans', enabled: true, kind: { kind: 'plans' } },
					{
						title: 'Notes',
						enabled: true,
						kind: { kind: 'blank_pages', count: 3 },
					},
				],
			},
			(index) => `k${index}`,
		)
		const request = buildModelRequest(draft)
		expect(request.description).toBeNull()
		expect(request.sections.map((section) => section.kind)).toEqual([
			{ kind: 'supplies', maker_label: 'nursery' },
			{ kind: 'photos', per_page: 'four' },
			{ kind: 'plans' },
			{ kind: 'blank_pages', count: 3 },
		])
		expect(request.sections[1]?.enabled).toBe(false)
	})
})

describe('buildModelRequest', () => {
	it('trims and clamps the blank page count', () => {
		const request = buildModelRequest({
			...emptyModelDraft(),
			name: '  Mon modèle ',
			description: ' texte ',
			sections: [
				{ ...newModelSectionDraft('a', ' Notes ', 'blank_pages'), count: 99 },
			],
		})
		expect(request.name).toBe('Mon modèle')
		expect(request.description).toBe('texte')
		expect(request.sections[0]?.title).toBe('Notes')
		expect(request.sections[0]?.kind).toEqual({
			kind: 'blank_pages',
			count: 20,
		})
	})
})

describe('canSubmitModel', () => {
	it('needs a name and titled sections', () => {
		expect(canSubmitModel(emptyModelDraft())).toBe(false)
		const named = { ...emptyModelDraft(), name: 'M' }
		expect(canSubmitModel(named)).toBe(true)
		expect(
			canSubmitModel({
				...named,
				sections: [newModelSectionDraft('a', ' ', 'plans')],
			}),
		).toBe(false)
	})
})

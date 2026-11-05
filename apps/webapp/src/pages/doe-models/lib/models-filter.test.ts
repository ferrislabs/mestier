import { describe, expect, it } from 'vitest'
import type { DoeModel } from '#/hooks/use-doe-models'
import {
	filterModels,
	modelCounts,
	parseModelsSearch,
	themeCounts,
	themesLabel,
} from '#/pages/doe-models/lib/models-filter'

function model(name: string, archived: boolean, enabled: boolean[]): DoeModel {
	return {
		id: name,
		organization_id: 'org',
		name,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		archived_at: archived ? '2026-02-01T00:00:00Z' : null,
		sections: enabled.map((flag, index) => ({
			title: `T${index}`,
			enabled: flag,
			kind: { kind: 'plans' },
		})),
	}
}

describe('models filter', () => {
	const models = [
		model('Zèbre', false, [true]),
		model('Alpha', false, [true, false]),
		model('Vieux', true, []),
	]

	it('keeps the requested state, sorted by name', () => {
		expect(filterModels(models, 'actifs').map((item) => item.name)).toEqual([
			'Alpha',
			'Zèbre',
		])
		expect(filterModels(models, 'archives').map((item) => item.name)).toEqual([
			'Vieux',
		])
	})

	it('counts active and archived models', () => {
		expect(modelCounts(models)).toEqual({ active: 2, archived: 1 })
	})

	it('words the themes of a model', () => {
		expect(themesLabel(themeCounts(models[1] as DoeModel))).toBe(
			'2 thèmes, 1 activé',
		)
		expect(themesLabel(themeCounts(models[2] as DoeModel))).toBe('Aucun thème')
	})

	it('only keeps the archived status in the search', () => {
		expect(parseModelsSearch({ statut: 'archives' })).toEqual({
			statut: 'archives',
		})
		expect(parseModelsSearch({ statut: 'x' })).toEqual({})
	})
})

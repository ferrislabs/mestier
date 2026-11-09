import { describe, expect, it } from 'vitest'
import {
	buildPlacements,
	failedResults,
	failureLabel,
	mergePhotos,
	placedIds,
	placementSummary,
} from '#/pages/doe/lib/library-placement'

describe('buildPlacements', () => {
	it('targets a section by its id', () => {
		expect(
			buildPlacements(['p1', 'p2'], { kind: 'section', sectionId: 's1' }),
		).toEqual([
			{ photo_id: 'p1', target: 'section', section_id: 's1' },
			{ photo_id: 'p2', target: 'section', section_id: 's1' },
		])
	})

	it('targets a supply by its id', () => {
		expect(buildPlacements(['p1'], { kind: 'supply', supplyId: 'u1' })).toEqual(
			[{ photo_id: 'p1', target: 'supply', supply_id: 'u1' }],
		)
	})
})

describe('placement results', () => {
	const results = [
		{ photo_id: 'p1', status: 'placed' as const, key: 'k1' },
		{ photo_id: 'p2', status: 'failed' as const, error: 'Section introuvable' },
		{ photo_id: 'p3', status: 'failed' as const },
	]

	it('splits placed and failed', () => {
		expect([...placedIds(results)]).toEqual(['p1'])
		expect(failedResults(results).map((r) => r.photo_id)).toEqual(['p2', 'p3'])
	})

	it('always has a message for a failure', () => {
		expect(failureLabel(results[1] as (typeof results)[number])).toBe(
			'Section introuvable',
		)
		expect(failureLabel(results[2] as (typeof results)[number])).toMatch(
			/pas pu être placée/,
		)
	})

	it('summarises in French', () => {
		expect(placementSummary(2, 1)).toBe('2 photos placées, 1 en échec')
		expect(placementSummary(1, 0)).toBe('1 photo placée')
	})
})

describe('mergePhotos', () => {
	it('keeps the local photos and appends the saved ones it lacks', () => {
		expect(
			mergePhotos(
				[{ key: 'a', caption: 'x' }],
				[
					{ key: 'a', caption: null },
					{ key: 'b', caption: null },
				],
			),
		).toEqual([
			{ key: 'a', caption: 'x' },
			{ key: 'b', caption: null },
		])
	})
})

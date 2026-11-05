import { beforeEach, describe, expect, it } from 'vitest'
import {
	archiveWorkflowMock,
	placementOf,
	readArchived,
	readPlacements,
	rememberWorkflowNames,
	resetPlacementsMock,
	seededPlacement,
	setPlacementMock,
} from '#/pages/automation/feature/mocks/workflow-placements'

describe('workflow placements mock', () => {
	beforeEach(() => {
		localStorage.clear()
		resetPlacementsMock()
	})

	it('places the prototype workflows by name, ignoring accents', () => {
		expect(seededPlacement('any-id', 'Relance des devis')).toEqual({
			folderId: 'ventes-relances',
			tags: ['Ventes'],
		})
		expect(
			seededPlacement('any-id', 'Devis accepté → création du projet'),
		).toEqual({ folderId: 'ventes', tags: ['Ventes', 'Projets'] })
	})

	it('spreads unknown workflows deterministically over the folders', () => {
		expect(seededPlacement('id-1', 'Autre')).toEqual(
			seededPlacement('id-1', 'Encore autre'),
		)
	})

	it('uses the remembered name to seed a placement', () => {
		rememberWorkflowNames([{ id: 'wf-1', name: 'Relance des devis' }])
		expect(placementOf('wf-1').folderId).toBe('ventes-relances')
		expect(readPlacements()['wf-1']?.tags).toEqual(['Ventes'])
	})

	it('lets an explicit placement win over the seed', () => {
		setPlacementMock('wf-1', { folderId: 'support', tags: ['Support'] })
		expect(placementOf('wf-1')).toEqual({
			folderId: 'support',
			tags: ['Support'],
		})
	})

	it('archives a workflow and keeps it archived across reloads', () => {
		archiveWorkflowMock('wf-1')
		expect(readArchived().has('wf-1')).toBe(true)
		expect(
			localStorage.getItem('mestier:automation-workflow-placements:v1'),
		).toContain('wf-1')
	})
})

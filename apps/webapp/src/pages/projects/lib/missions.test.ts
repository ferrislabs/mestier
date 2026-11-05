import { describe, expect, it } from 'vitest'
import {
	missionStats,
	movedMissions,
	validateMissionDraft,
} from '#/pages/projects/lib/missions'
import type { Mission } from '#/pages/projects/lib/model'

function mission(
	id: string,
	position: number,
	status: Mission['status'],
): Mission {
	return {
		id,
		projectId: 'p',
		title: id,
		description: '',
		status,
		startDate: null,
		endDate: null,
		responsibleId: null,
		position,
		createdAt: '2026-01-01',
	}
}

describe('missions', () => {
	it('computes the progress from the done missions', () => {
		const stats = missionStats([
			mission('a', 0, 'done'),
			mission('b', 1, 'in_progress'),
			mission('c', 2, 'todo'),
			mission('d', 3, 'done'),
		])

		expect(stats).toEqual({
			total: 4,
			todo: 1,
			inProgress: 1,
			done: 2,
			progress: 50,
		})
		expect(missionStats([]).progress).toBe(0)
	})

	it('moves a mission one step and renumbers the positions', () => {
		const moved = movedMissions(
			[
				mission('a', 0, 'todo'),
				mission('b', 1, 'todo'),
				mission('c', 2, 'todo'),
			],
			'c',
			-1,
		)

		expect(
			[...moved].sort((x, y) => x.position - y.position).map((m) => m.id),
		).toEqual(['a', 'c', 'b'])
	})

	it('does not move past the ends', () => {
		const list = [mission('a', 0, 'todo'), mission('b', 1, 'todo')]

		expect(movedMissions(list, 'a', -1)).toEqual(list)
	})

	it('requires a title and a coherent period', () => {
		const draft = {
			title: ' ',
			description: '',
			status: 'todo' as const,
			startDate: '2026-02-01',
			endDate: '2026-01-01',
			responsibleId: null,
		}

		expect(validateMissionDraft(draft)).toEqual([
			'Le titre de la mission est obligatoire.',
			'La fin ne peut pas précéder le début.',
		])
	})
})

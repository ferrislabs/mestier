import { describe, expect, it } from 'vitest'
import { MOCK_HOME_ACTIVITY } from '#/pages/home/feature/mocks/activity'

describe('MOCK_HOME_ACTIVITY', () => {
	it('points the automation items at the workflow list and the identifications', () => {
		const targets = Object.fromEntries(
			MOCK_HOME_ACTIVITY.filter(
				(item) => item.moduleId === 'automatisation',
			).map((item) => [item.id, item.to]),
		)

		expect(targets).toEqual({
			'auto-relance': '/automatisation/workflows',
			'auto-smtp': '/automatisation/identifications',
		})
	})
})

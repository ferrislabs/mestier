import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	featureStateLine,
	historySentence,
} from '#/pages/admin-organizations/lib/history'

const driveAi: Schemas.AdminOrganizationFeatureResponse = {
	key: 'drive_ai',
	label: 'Drive IA',
	description: 'Assistant du drive',
	enabled: true,
	enabled_at: '2026-11-10T09:00:00Z',
	enabled_by_name: 'Nathael',
}

describe('historySentence', () => {
	it('names the actor, the verb and the flag label', () => {
		expect(
			historySentence(
				{
					flag: 'drive_ai',
					enabled: true,
					actor_name: 'Nathael',
					occurred_at: '2026-11-10T09:00:00Z',
				},
				[driveAi],
			),
		).toBe('Nathael a activé Drive IA')
		expect(
			historySentence(
				{
					flag: 'drive_ai',
					enabled: false,
					actor_name: 'Nathael',
					occurred_at: '2026-11-11T09:00:00Z',
				},
				[driveAi],
			),
		).toBe('Nathael a désactivé Drive IA')
	})

	it('falls back on the key and a generic actor when unknown', () => {
		expect(
			historySentence(
				{
					flag: 'ancien',
					enabled: true,
					actor_name: null,
					occurred_at: '2026-11-10T09:00:00Z',
				},
				[driveAi],
			),
		).toBe('Un administrateur a activé ancien')
	})
})

describe('featureStateLine', () => {
	it('says when and by whom an enabled flag was switched on', () => {
		expect(featureStateLine(driveAi, () => '10 nov. 2026')).toBe(
			'Activé le 10 nov. 2026 par Nathael',
		)
	})

	it('is empty for a disabled flag', () => {
		expect(
			featureStateLine({ ...driveAi, enabled: false }, () => 'x'),
		).toBeNull()
	})

	it('omits the actor when it is unknown', () => {
		expect(
			featureStateLine({ ...driveAi, enabled_by_name: null }, () => '10 nov.'),
		).toBe('Activé le 10 nov.')
	})
})

import { describe, expect, it } from 'vitest'
import {
	settingsOf,
	settingsPatch,
	toggleCampaign,
} from '#/pages/survey/lib/survey-settings'
import { survey } from '#/pages/survey/test/survey-harness'

describe('survey settings campaigns', () => {
	it('sends the full campaign list only when the set changed', () => {
		const saved = settingsOf(survey({ campaign_ids: ['a', 'b', 'c'] }))

		expect(settingsPatch(saved, saved, false)).toEqual({})
		expect(
			'campaign_ids' in
				settingsPatch({ ...saved, campaignIds: ['c', 'b', 'a'] }, saved, false),
		).toBe(false)
		expect(
			settingsPatch({ ...saved, campaignIds: ['a', 'b'] }, saved, false),
		).toEqual({ campaign_ids: ['a', 'b'] })
		expect(
			settingsPatch(
				{ ...saved, campaignIds: ['a', 'b', 'c', 'd'] },
				saved,
				false,
			),
		).toEqual({ campaign_ids: ['a', 'b', 'c', 'd'] })
	})

	it('attaches and detaches a campaign without touching the others', () => {
		const settings = settingsOf(survey({ campaign_ids: ['a', 'b'] }))

		expect(toggleCampaign(settings, 'c', true)).toEqual(['a', 'b', 'c'])
		expect(toggleCampaign(settings, 'a', false)).toEqual(['b'])
		expect(toggleCampaign(settings, 'b', true)).toEqual(['a', 'b'])
	})

	it('sends an empty list when the last campaign is detached', () => {
		const saved = settingsOf(survey({ campaign_ids: ['a'] }))

		expect(settingsPatch({ ...saved, campaignIds: [] }, saved, true)).toEqual({
			campaign_ids: [],
		})
	})

	it('sends only the changed fields and omits the visibility once opened', () => {
		const saved = settingsOf(survey({ description: 'Avant' }))

		expect(
			settingsPatch({ ...saved, title: 'Nouveau titre' }, saved, true),
		).toEqual({ title: 'Nouveau titre' })
		expect(settingsPatch({ ...saved, description: '  ' }, saved, true)).toEqual(
			{ description: null },
		)
		expect(
			settingsPatch({ ...saved, visibility: 'MEMBERS' }, saved, true),
		).toEqual({ visibility: 'MEMBERS' })
		expect(
			settingsPatch({ ...saved, visibility: 'MEMBERS' }, saved, false),
		).toEqual({})
	})
})

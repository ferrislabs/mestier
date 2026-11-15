import { describe, expect, it } from 'vitest'
import {
	settingsOf,
	settingsPatch,
	toggleCampaign,
} from '#/pages/survey/lib/survey-settings'
import { survey } from '#/pages/survey/test/survey-harness'

describe('survey settings campaigns', () => {
	it('keeps every attached campaign in order', () => {
		const settings = settingsOf(survey({ campaign_ids: ['a', 'b', 'c'] }))

		expect(settings.campaignIds).toEqual(['a', 'b', 'c'])
		expect(settingsPatch(settings, false).campaign_ids).toEqual(['a', 'b', 'c'])
	})

	it('attaches and detaches a campaign without touching the others', () => {
		const settings = settingsOf(survey({ campaign_ids: ['a', 'b'] }))

		expect(toggleCampaign(settings, 'c', true)).toEqual(['a', 'b', 'c'])
		expect(toggleCampaign(settings, 'a', false)).toEqual(['b'])
		expect(toggleCampaign(settings, 'b', true)).toEqual(['a', 'b'])
	})

	it('sends an empty list for an independent survey and omits the visibility once opened', () => {
		const settings = settingsOf(survey())

		expect(settingsPatch(settings, true).campaign_ids).toEqual([])
		expect(settingsPatch(settings, true).visibility).toBe('PUBLIC')
		expect('visibility' in settingsPatch(settings, false)).toBe(false)
	})
})

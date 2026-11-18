import { describe, expect, it } from 'vitest'
import { settingsOf, settingsPatch } from '#/pages/survey/lib/survey-settings'
import { survey } from '#/pages/survey/test/survey-harness'

describe('survey settings campaigns', () => {
	it('reads the first campaign into the select and keeps the others', () => {
		const settings = settingsOf(survey({ campaign_ids: ['a', 'b', 'c'] }))

		expect(settings.campaignId).toBe('a')
		expect(settingsPatch(settings, false).campaign_ids).toEqual(['a', 'b', 'c'])
	})

	it('replaces the first campaign without losing the others', () => {
		const settings = settingsOf(survey({ campaign_ids: ['a', 'b'] }))

		expect(
			settingsPatch({ ...settings, campaignId: 'b' }, false).campaign_ids,
		).toEqual(['b'])
		expect(
			settingsPatch({ ...settings, campaignId: 'c' }, false).campaign_ids,
		).toEqual(['c', 'b'])
		expect(
			settingsPatch({ ...settings, campaignId: '' }, false).campaign_ids,
		).toEqual(['b'])
	})

	it('sends an empty list for an independent survey', () => {
		const settings = settingsOf(survey())

		expect(settings.campaignId).toBe('')
		expect(settingsPatch(settings, true).campaign_ids).toEqual([])
	})
})

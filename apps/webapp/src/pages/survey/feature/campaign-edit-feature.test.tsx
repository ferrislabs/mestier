import { screen, within } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { CampaignEditFeature } from '#/pages/survey/feature/campaign-edit-feature'
import {
	campaign,
	installSurveyApi,
	PATHS,
	renderSurveyPage,
} from '#/pages/survey/test/survey-harness'

describe('CampaignEditFeature', () => {
	it('lists the surveys attached to the campaign', async () => {
		const api = installSurveyApi({ permissions: ['VIEW_SURVEYS'] })
		api.mockGet(`${PATHS.campaigns}/{campaign_id}`, () => ({
			data: campaign(),
			pagination: null,
		}))
		api.mockGet(`${PATHS.campaigns}/{campaign_id}/surveys`, () => ({
			data: [
				{ id: 's-1', title: 'Bilan de session', status: 'OPENED' },
				{ id: 's-2', title: 'Retour formateur', status: 'DRAFT' },
			],
			pagination: null,
		}))
		await renderSurveyPage(<CampaignEditFeature campaignId="campaign-1" />)

		const section = (
			await screen.findByRole('heading', { name: 'Questionnaires rattachés' })
		).closest('section')
		if (!section) throw new Error('no section')
		expect(
			await within(section).findByRole('button', { name: /Bilan de session/ }),
		).toBeTruthy()
		expect(
			within(section).getByRole('button', { name: /Retour formateur/ }),
		).toBeTruthy()
	})
})

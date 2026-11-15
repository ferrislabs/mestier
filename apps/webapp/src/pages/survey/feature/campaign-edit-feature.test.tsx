import { fireEvent, screen, within } from '@testing-library/react'
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

	it('pages through 120 attached surveys with « Voir plus »', async () => {
		const api = installSurveyApi({ permissions: ['VIEW_SURVEYS'] })
		api.mockGet(`${PATHS.campaigns}/{campaign_id}`, () => ({
			data: campaign(),
			pagination: null,
		}))
		const all = Array.from({ length: 120 }, (_, index) => ({
			id: `s-${index + 1}`,
			title: `Questionnaire ${index + 1}`,
			status: 'DRAFT',
		}))
		const requested: unknown[] = []
		api.mockGet(`${PATHS.campaigns}/{campaign_id}/surveys`, (params) => {
			const query = (params as { query: { page: number; per_page: number } })
				.query
			requested.push(query)
			const size = 50
			const last = Math.ceil(all.length / size)
			return {
				data: all.slice((query.page - 1) * size, query.page * size),
				pagination: {
					current_page: query.page,
					first_page: 1,
					is_empty: false,
					last_page: last,
					next_page: query.page < last ? query.page + 1 : null,
					per_page: size,
					total: all.length,
				},
			}
		})
		await renderSurveyPage(<CampaignEditFeature campaignId="campaign-1" />)

		expect(await screen.findByText('50 sur 120')).toBeTruthy()
		expect(requested[0]).toEqual({ page: 1, per_page: 200 })
		fireEvent.click(screen.getByRole('button', { name: 'Voir plus' }))
		expect(await screen.findByText('100 sur 120')).toBeTruthy()
		fireEvent.click(screen.getByRole('button', { name: 'Voir plus' }))
		expect(await screen.findByText('120 sur 120')).toBeTruthy()
		expect(screen.queryByRole('button', { name: 'Voir plus' })).toBe(null)
		expect(
			screen.getByRole('button', { name: /Questionnaire 120/ }),
		).toBeTruthy()
	})
})

import { fireEvent, screen, waitFor, within } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { SurveyResponsesFeature } from '#/pages/survey/feature/survey-responses-feature'
import {
	installSurveyApi,
	PATHS,
	renderSurveyPage,
	survey,
} from '#/pages/survey/test/survey-harness'
import type { SurveyResponse } from '#/pages/survey/types'

const RESPONSES_PATH = `${PATHS.survey}/responses`
const RESPONSE_PATH =
	'/api/v1/organizations/{organization_id}/survey-responses/{response_id}'
const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'

function response(id: string): SurveyResponse {
	return {
		id,
		survey_id: 'survey-1',
		answers: {},
		status: 'COMPLETED',
		started_at: '2026-05-02T08:00:00Z',
		completed_at: '2026-05-02T08:05:00Z',
		member_id: null,
	}
}

describe('SurveyResponsesFeature', () => {
	it('steps back one page after deleting the last response of a later page', async () => {
		const pages: Record<number, SurveyResponse[]> = {
			1: [response('r1'), response('r2')],
			2: [response('r3')],
		}
		const api = installSurveyApi({
			permissions: ['VIEW_SURVEYS', 'MANAGE_SURVEYS'],
			surveys: [survey()],
		})
		api.mockGet(MEMBERS_PATH, () => ({ data: [], pagination: null }))
		api.mockGet(RESPONSES_PATH, (params) => {
			const page = (params as { query: { page: number } }).query.page
			const lastPage = pages[2].length > 0 ? 2 : 1
			return {
				data: pages[page] ?? [],
				pagination: {
					current_page: page,
					first_page: 1,
					is_empty: false,
					last_page: lastPage,
					per_page: 2,
					total: pages[1].length + pages[2].length,
				},
			}
		})
		api.mockMutation('delete', RESPONSE_PATH, () => {
			pages[2] = []
			return null
		})
		await renderSurveyPage(<SurveyResponsesFeature surveyId="survey-1" />)

		const pagination = await screen.findByRole('navigation', {
			name: 'Pagination des réponses',
		})
		fireEvent.click(
			within(pagination).getByRole('button', { name: /Suivante/ }),
		)
		await screen.findByText('Page 2 sur 2')
		const rows = screen.getAllByRole('row')
		fireEvent.click(rows[rows.length - 1])
		fireEvent.click(
			await screen.findByRole('button', { name: /Supprimer la réponse/ }),
		)
		const dialog = await screen.findByRole('dialog', {
			name: 'Supprimer cette réponse ?',
		})
		fireEvent.click(within(dialog).getByRole('button', { name: 'Supprimer' }))

		await waitFor(() => expect(screen.getByText('Page 1 sur 1')).toBeTruthy())
		expect(screen.getAllByRole('row').length).toBeGreaterThan(2)
	})
})

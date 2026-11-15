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

	it('reads an older response against the graph of its own version', async () => {
		const graphOf = (label: string) => ({
			start: 'q1',
			nodes: [
				{
					type: 'QUESTION' as const,
					id: 'q1',
					kind: 'TEXT' as const,
					label,
					required: false,
				},
				{ type: 'END' as const, id: 'e1', message: null },
			],
			edges: [{ from: 'q1', to: 'e1', branch: 'NEXT' as const }],
		})
		const api = installSurveyApi({
			permissions: ['VIEW_SURVEYS'],
			surveys: [survey({ graph: graphOf('Votre ville'), graph_version: 2 })],
		})
		api.mockGet(MEMBERS_PATH, () => ({ data: [], pagination: null }))
		api.mockGet(RESPONSES_PATH, () => ({
			data: [
				{
					...response('r1'),
					graph_version: 1,
					answers: { q1: { Text: 'Lyon' } },
				},
			],
			pagination: null,
		}))
		api.mockGet(`${PATHS.survey}/graph-versions/{version}`, (params) => {
			expect((params as { path: { version: number } }).path.version).toBe(1)
			return {
				data: { version: 1, graph: graphOf('Votre commune') },
				pagination: null,
			}
		})
		await renderSurveyPage(<SurveyResponsesFeature surveyId="survey-1" />)

		const rows = await screen.findAllByRole('row')
		expect(within(rows[1]).getByText('Version 1')).toBeTruthy()
		fireEvent.click(rows[1])
		expect(await screen.findByText('Votre commune')).toBeTruthy()
		expect(screen.getByText(/aujourd’hui en version 2\./)).toBeTruthy()
	})

	it('reads a response on the current version when its own version is missing', async () => {
		const api = installSurveyApi({
			permissions: ['VIEW_SURVEYS'],
			surveys: [
				survey({
					graph: {
						start: 'q1',
						nodes: [
							{
								type: 'QUESTION',
								id: 'q1',
								kind: 'TEXT',
								label: 'Votre ville',
								required: false,
							},
							{ type: 'END', id: 'e1', message: null },
						],
						edges: [{ from: 'q1', to: 'e1', branch: 'NEXT' }],
					},
					graph_version: 3,
				}),
			],
		})
		api.mockGet(MEMBERS_PATH, () => ({ data: [], pagination: null }))
		api.mockGet(RESPONSES_PATH, () => ({
			data: [
				{
					...response('r1'),
					graph_version: 2,
					answers: { q1: { Text: 'Lyon' } },
				},
			],
			pagination: null,
		}))
		api.mockGet(`${PATHS.survey}/graph-versions/{version}`, () => {
			throw Object.assign(new Error('Not found'), {
				status: 404,
				data: {
					code: 'E_NOT_FOUND',
					details: { reason: 'graph_version_not_found' },
				},
			})
		})
		await renderSurveyPage(<SurveyResponsesFeature surveyId="survey-1" />)

		const rows = await screen.findAllByRole('row')
		fireEvent.click(rows[1])
		expect(
			await screen.findByText(
				'Version 2 introuvable : affichage sur la version actuelle.',
			),
		).toBeTruthy()
		expect(screen.getByText('Votre ville')).toBeTruthy()
	})
})

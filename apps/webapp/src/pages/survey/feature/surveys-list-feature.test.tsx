import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
import { SurveysListFeature } from '#/pages/survey/feature/surveys-list-feature'
import {
	campaign,
	installSurveyApi,
	PATHS,
	renderSurveyPage,
	survey,
} from '#/pages/survey/test/survey-harness'

for (const method of [
	'hasPointerCapture',
	'setPointerCapture',
	'releasePointerCapture',
] as const) {
	if (typeof Element.prototype[method] !== 'function') {
		Element.prototype[method] = (() => false) as never
	}
}

const SURVEYS = [
	survey({ id: 's-draft', title: 'Brouillon de test', status: 'DRAFT' }),
	survey({
		id: 's-open',
		title: 'QCM ouvert',
		status: 'OPENED',
		public_slug: 'abcd1234',
		campaign_id: 'campaign-1',
		opened_at: '2026-05-02T08:00:00Z',
	}),
	survey({
		id: 's-closed',
		title: 'Ancien questionnaire',
		status: 'CLOSED',
		visibility: 'MEMBERS',
	}),
]

async function rowOf(title: string) {
	const cell = await screen.findByRole('button', { name: title })
	const row = cell.closest('tr')
	if (!row) throw new Error(`no row for ${title}`)
	return row as HTMLElement
}

describe('SurveysListFeature', () => {
	it('shows each survey with its status, visibility and campaign', async () => {
		installSurveyApi({
			permissions: ['VIEW_SURVEYS', 'MANAGE_SURVEYS'],
			surveys: SURVEYS,
			campaigns: [campaign()],
		})
		await renderSurveyPage(<SurveysListFeature />)

		const draft = await rowOf('Brouillon de test')
		expect(within(draft).getByText('Brouillon')).toBeTruthy()
		expect(within(draft).getByText('Indépendant')).toBeTruthy()
		const opened = await rowOf('QCM ouvert')
		expect(within(opened).getByText('Ouvert')).toBeTruthy()
		expect(within(opened).getByText('Public')).toBeTruthy()
		await waitFor(() =>
			expect(within(opened).getByText('Session de mai')).toBeTruthy(),
		)
		const closed = await rowOf('Ancien questionnaire')
		expect(within(closed).getByText('Fermé')).toBeTruthy()
		expect(within(closed).getByText('Membres')).toBeTruthy()
	})

	it('offers the manage actions to a manager', async () => {
		installSurveyApi({
			permissions: ['VIEW_SURVEYS', 'MANAGE_SURVEYS'],
			surveys: SURVEYS,
		})
		await renderSurveyPage(<SurveysListFeature />)

		expect(
			await screen.findByRole('button', { name: 'Nouveau questionnaire' }),
		).toBeTruthy()
		await userEvent.click(
			within(await rowOf('QCM ouvert')).getByRole('button', {
				name: 'Actions sur « QCM ouvert »',
			}),
		)
		const menu = await screen.findByRole('menu')
		const items = within(menu)
			.getAllByRole('menuitem')
			.map((item) => item.textContent)
		expect(items).toEqual([
			'Ouvrir l’éditeur',
			'Voir les réponses',
			'Copier le lien public',
			'Dupliquer',
			'Fermer…',
			'Supprimer…',
		])
	})

	it('hides every manage action from a reader', async () => {
		installSurveyApi({ permissions: ['VIEW_SURVEYS'], surveys: SURVEYS })
		await renderSurveyPage(<SurveysListFeature />)

		const draft = await rowOf('Brouillon de test')
		expect(
			screen.queryByRole('button', { name: 'Nouveau questionnaire' }),
		).toBe(null)
		await userEvent.click(
			within(draft).getByRole('button', {
				name: 'Actions sur « Brouillon de test »',
			}),
		)
		const menu = await screen.findByRole('menu')
		expect(
			within(menu)
				.getAllByRole('menuitem')
				.map((item) => item.textContent),
		).toEqual(['Ouvrir l’éditeur', 'Voir les réponses'])
	})

	it('disables the duplicate action while a duplicate is running', async () => {
		const api = installSurveyApi({
			permissions: ['VIEW_SURVEYS', 'MANAGE_SURVEYS'],
			surveys: SURVEYS,
		})
		api.mockMutation(
			'post',
			`${PATHS.survey}/duplicate`,
			() => new Promise(() => {}),
		)
		await renderSurveyPage(<SurveysListFeature />)

		const openMenu = async () => {
			await userEvent.click(
				within(await rowOf('QCM ouvert')).getByRole('button', {
					name: 'Actions sur « QCM ouvert »',
				}),
			)
			return within(await screen.findByRole('menu')).getByRole('menuitem', {
				name: 'Dupliquer',
			})
		}
		await userEvent.click(await openMenu())
		await waitFor(() =>
			expect(api.calls.filter((call) => call.method === 'post')).toHaveLength(
				1,
			),
		)
		const again = await openMenu()
		expect(again.getAttribute('aria-disabled')).toBe('true')
		await userEvent.click(again)
		expect(api.calls.filter((call) => call.method === 'post')).toHaveLength(1)
	})
})

import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { PublicSurveyFeature } from '#/pages/survey-respond/feature/public-survey-feature'
import { resumeKey } from '#/pages/survey-respond/lib/resume-store'
import {
	question,
	respondentSurvey,
} from '#/pages/survey-respond/test/respond-fixtures'
import {
	data,
	installServer,
	type RecordedCall,
	refusal,
	renderWithQuery,
} from '#/pages/survey-respond/test/respond-server'
import { clearAuth, setAuth } from '#/store/auth.store'

const BASE = '/api/v1/public/surveys/lien-bilan'
const TOKEN = 'resume-token-123456'

function response(
	answers: Record<string, unknown> = {},
	status = 'IN_PROGRESS',
) {
	return { id: 'response-1', answers, status }
}

function happyRoutes(
	extra: Record<string, (call: RecordedCall) => ReturnType<typeof data>> = {},
) {
	return {
		[`GET ${BASE}`]: () => data(respondentSurvey()),
		[`POST ${BASE}/responses`]: () =>
			data({ ...response(), resume_token: TOKEN }, 201),
		[`PUT ${BASE}/responses/${TOKEN}`]: (call: RecordedCall) =>
			data(
				response((call.body as { answers: Record<string, unknown> }).answers),
			),
		[`POST ${BASE}/responses/${TOKEN}/complete`]: () =>
			data(response({}, 'COMPLETED')),
		...extra,
	}
}

const IDENTIFIER_GRAPH = {
	start: 'q-email',
	nodes: [
		question('q-email', 'EMAIL', {
			label: 'Votre e-mail',
			required: true,
			identifier: true,
		}),
		question('q-note', 'TEXT', { label: 'Un commentaire' }),
		{ type: 'END' as const, id: 'end', message: 'Merci !' },
	],
	edges: [
		{ from: 'q-email', to: 'q-note', branch: 'NEXT' as const },
		{ from: 'q-note', to: 'end', branch: 'NEXT' as const },
	],
}

async function answerToEmail() {
	await heading()
	await userEvent.click(screen.getByRole('radio', { name: 'Élève' }))
	await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
	await userEvent.type(
		await screen.findByRole('textbox', { name: 'Votre classe' }),
		'2nde',
	)
	await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
	await screen.findByRole('textbox', { name: 'Votre e-mail' })
}

function heading() {
	return screen.findByRole('heading', { level: 2 })
}

beforeEach(() => {
	window.localStorage.clear()
})

afterEach(() => {
	vi.unstubAllGlobals()
	clearAuth()
})

describe('PublicSurveyFeature', () => {
	it('follows the branch of the answer, keeps answers on the way back and starts nothing before the first step is submitted', async () => {
		const { calls } = installServer(happyRoutes())
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)

		expect((await heading()).textContent).toBe('Vous êtes')
		expect(screen.getByText('Question 1')).toBeTruthy()
		await userEvent.click(screen.getByRole('radio', { name: 'Professeur' }))
		expect(calls.map((call) => call.method)).toEqual(['GET'])

		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		const subject = await screen.findByRole('heading', {
			name: 'Quelle matière enseignez-vous ?',
		})
		expect(document.activeElement).toBe(subject)
		expect(screen.getByText('Question 2 sur 3')).toBeTruthy()
		await waitFor(() =>
			expect(calls.map((call) => `${call.method} ${call.url}`)).toEqual([
				`GET ${BASE}`,
				`POST ${BASE}/responses`,
				`PUT ${BASE}/responses/${TOKEN}`,
			]),
		)
		expect(calls[2]?.body).toEqual({
			answers: { 'q-role': { Choice: 'Professeur' } },
		})
		expect(window.localStorage.getItem(resumeKey('lien-bilan'))).toBe(TOKEN)

		await userEvent.type(
			screen.getByRole('textbox', { name: 'Quelle matière enseignez-vous ?' }),
			'Maçonnerie',
		)
		await userEvent.click(screen.getByRole('button', { name: /Précédent/ }))
		await screen.findByRole('heading', { name: 'Vous êtes' })
		expect(
			(screen.getByRole('radio', { name: 'Professeur' }) as HTMLInputElement)
				.checked,
		).toBe(true)
		await userEvent.click(screen.getByRole('radio', { name: 'Élève' }))
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		expect(
			await screen.findByRole('heading', { name: 'Votre classe' }),
		).toBeTruthy()
		expect(
			calls.filter((call) => call.url === `${BASE}/responses`),
		).toHaveLength(1)
		await waitFor(() =>
			expect(calls.filter((call) => call.method === 'PUT')).toHaveLength(3),
		)
		const puts = calls.filter((call) => call.method === 'PUT')
		expect(puts[1]?.body).toEqual({
			answers: {
				'q-role': { Choice: 'Professeur' },
				'q-subject': { Text: 'Maçonnerie' },
			},
		})
		expect(puts[2]?.body).toEqual({
			answers: { 'q-role': { Choice: 'Élève' } },
		})
	})

	it('sends no Authorization header and no credentials, even with a session open', async () => {
		setAuth({ accessToken: 'secret-session-token', isAuthenticated: true })
		const { calls } = installServer(happyRoutes())
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		await heading()
		await userEvent.click(screen.getByRole('radio', { name: 'Élève' }))
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		await screen.findByRole('heading', { name: 'Votre classe' })
		await waitFor(() => expect(calls.length).toBe(3))
		for (const call of calls) {
			expect(call.headers.has('Authorization')).toBe(false)
			expect(call.credentials).toBe('omit')
			expect(call.referrerPolicy).toBe('no-referrer')
		}
	})

	it('refuses to go on without a required answer or with a malformed one', async () => {
		const { calls } = installServer(happyRoutes())
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		await heading()
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		expect(screen.getByText('Choisissez une option.')).toBeTruthy()
		expect(calls).toHaveLength(1)

		await userEvent.click(screen.getByRole('radio', { name: 'Élève' }))
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		await userEvent.type(
			await screen.findByRole('textbox', { name: 'Votre classe' }),
			'Terminale CAP',
		)
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		await userEvent.type(
			await screen.findByRole('textbox', { name: 'Votre e-mail' }),
			'pas-un-email',
		)
		await userEvent.click(screen.getByRole('button', { name: 'Terminer' }))
		expect(
			screen.getByText(
				'Saisissez une adresse e-mail valide, par exemple nom@exemple.fr.',
			),
		).toBeTruthy()
		expect(calls.some((call) => call.url.endsWith('/complete'))).toBe(false)
	})

	it('completes at the End node, shows its message and forgets the token', async () => {
		const { calls } = installServer(happyRoutes())
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		await heading()
		await userEvent.click(screen.getByRole('radio', { name: 'Élève' }))
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		await userEvent.type(
			await screen.findByRole('textbox', { name: 'Votre classe' }),
			'2nde',
		)
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		await screen.findByRole('textbox', { name: 'Votre e-mail' })
		expect(screen.getByText('Question 3 sur 3')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: 'Terminer' }))

		expect(await screen.findByText('Merci, à bientôt !')).toBeTruthy()
		expect(
			screen.queryByRole('button', { name: /Suivant|Terminer/ }),
		).toBeNull()
		expect(calls.at(-1)?.url).toBe(`${BASE}/responses/${TOKEN}/complete`)
		expect(calls.at(-2)?.body).toEqual({
			answers: { 'q-role': { Choice: 'Élève' }, 'q-class': { Text: '2nde' } },
		})
		expect(window.localStorage.getItem(resumeKey('lien-bilan'))).toBeNull()
		expect(document.title).toBe('Bilan de fin de stage · Mestier')
	})

	it('resumes from the stored token at the first unanswered question', async () => {
		window.localStorage.setItem(resumeKey('lien-bilan'), TOKEN)
		const { calls } = installServer(
			happyRoutes({
				[`GET ${BASE}/responses/${TOKEN}`]: () =>
					data(response({ 'q-role': { Choice: 'Professeur' } })),
			}),
		)
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		expect(
			await screen.findByRole('heading', {
				name: 'Quelle matière enseignez-vous ?',
			}),
		).toBeTruthy()
		expect(screen.getByText('Question 2 sur 3')).toBeTruthy()
		expect(calls.some((call) => call.method === 'POST')).toBe(false)
	})

	it('says an expired response expired, drops the stale token and starts again', async () => {
		window.localStorage.setItem(resumeKey('lien-bilan'), 'stale-token-0000')
		installServer(
			happyRoutes({
				[`GET ${BASE}/responses/stale-token-0000`]: () => ({ status: 404 }),
			}),
		)
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		expect(
			await screen.findByRole('heading', { name: 'Votre réponse a expiré' }),
		).toBeTruthy()
		expect(window.localStorage.getItem(resumeKey('lien-bilan'))).toBeNull()
		await userEvent.click(screen.getByRole('button', { name: 'Recommencer' }))
		const first = await screen.findByRole('heading', { name: 'Vous êtes' })
		await waitFor(() => expect(document.activeElement).toBe(first))
	})

	it('forgets the token when a save finds the response expired', async () => {
		installServer(
			happyRoutes({
				[`PUT ${BASE}/responses/${TOKEN}`]: () => ({ status: 404 }),
			}),
		)
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		await heading()
		await userEvent.click(screen.getByRole('radio', { name: 'Élève' }))
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		expect(
			await screen.findByRole('heading', { name: 'Votre réponse a expiré' }),
		).toBeTruthy()
		expect(window.localStorage.getItem(resumeKey('lien-bilan'))).toBeNull()
	})

	it('sends the respondent back to the identifier question, answers kept, when its value was already used', async () => {
		let completes = 0
		const { calls } = installServer({
			[`GET ${BASE}`]: () => data(respondentSurvey(IDENTIFIER_GRAPH)),
			[`POST ${BASE}/responses`]: () =>
				data({ ...response(), resume_token: TOKEN }, 201),
			[`PUT ${BASE}/responses/${TOKEN}`]: () => data(response()),
			[`POST ${BASE}/responses/${TOKEN}/complete`]: () => {
				completes += 1
				return completes === 1
					? refusal(409, 'respondent_already_answered')
					: data(response({}, 'COMPLETED'))
			},
		})
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		await userEvent.type(
			await screen.findByRole('textbox', { name: 'Votre e-mail' }),
			'lea@exemple.fr',
		)
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		await userEvent.type(
			await screen.findByRole('textbox', { name: 'Un commentaire' }),
			'RAS',
		)
		await userEvent.click(screen.getByRole('button', { name: 'Terminer' }))

		const email = (await screen.findByRole('textbox', {
			name: 'Votre e-mail',
		})) as HTMLInputElement
		expect(
			screen.getByText(
				'Une réponse a déjà été envoyée avec cette valeur. Vérifiez votre saisie ou indiquez-en une autre.',
			),
		).toBeTruthy()
		expect(screen.queryByRole('heading', { name: 'Déjà répondu' })).toBeNull()
		expect(email.value).toBe('lea@exemple.fr')
		expect(window.localStorage.getItem(resumeKey('lien-bilan'))).toBe(TOKEN)

		await userEvent.clear(email)
		await userEvent.type(email, 'lea.martin@exemple.fr')
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		expect(
			(
				(await screen.findByRole('textbox', {
					name: 'Un commentaire',
				})) as HTMLInputElement
			).value,
		).toBe('RAS')
		await userEvent.click(screen.getByRole('button', { name: 'Terminer' }))
		expect(await screen.findByText('Merci !')).toBeTruthy()
		expect(calls.filter((call) => call.method === 'PUT').at(-1)?.body).toEqual({
			answers: {
				'q-email': { Email: 'lea.martin@exemple.fr' },
				'q-note': { Text: 'RAS' },
			},
		})
	})

	it('shows the closing message and no form when the survey closes during a save', async () => {
		installServer(
			happyRoutes({
				[`PUT ${BASE}/responses/${TOKEN}`]: () => refusal(409, 'survey_closed'),
			}),
		)
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		await heading()
		await userEvent.click(screen.getByRole('radio', { name: 'Élève' }))
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		expect(
			await screen.findByRole('heading', { name: 'Questionnaire fermé' }),
		).toBeTruthy()
		expect(
			screen.getByText('Ce questionnaire n’accepte plus de réponses.'),
		).toBeTruthy()
		expect(screen.queryByRole('textbox')).toBeNull()
		expect(screen.queryByRole('button')).toBeNull()
	})

	it('shows the closing message and no form when the survey closes before completion', async () => {
		installServer(
			happyRoutes({
				[`POST ${BASE}/responses/${TOKEN}/complete`]: () =>
					refusal(409, 'survey_closed'),
			}),
		)
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		await answerToEmail()
		await userEvent.click(screen.getByRole('button', { name: 'Terminer' }))
		expect(
			await screen.findByRole('heading', { name: 'Questionnaire fermé' }),
		).toBeTruthy()
		expect(screen.queryByRole('textbox')).toBeNull()
		expect(screen.queryByRole('button')).toBeNull()
	})

	it('keeps the answers and lets the respondent retry when completion loses the network', async () => {
		let completes = 0
		const { calls } = installServer(
			happyRoutes({
				[`POST ${BASE}/responses/${TOKEN}/complete`]: () => {
					completes += 1
					return completes === 1
						? refusal(503)
						: data(response({}, 'COMPLETED'))
				},
			}),
		)
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		await answerToEmail()
		await userEvent.type(
			screen.getByRole('textbox', { name: 'Votre e-mail' }),
			'lea@exemple.fr',
		)
		await userEvent.click(screen.getByRole('button', { name: 'Terminer' }))
		expect(await screen.findByText(/Connexion impossible/)).toBeTruthy()
		expect(
			(
				screen.getByRole('textbox', {
					name: 'Votre e-mail',
				}) as HTMLInputElement
			).value,
		).toBe('lea@exemple.fr')
		expect(window.localStorage.getItem(resumeKey('lien-bilan'))).toBe(TOKEN)
		await userEvent.click(screen.getByRole('button', { name: 'Terminer' }))
		expect(await screen.findByText('Merci, à bientôt !')).toBeTruthy()
		expect(calls.filter((call) => call.url.endsWith('/complete'))).toHaveLength(
			2,
		)
	})

	it('shows the French closing message and no form when the survey is closed', async () => {
		installServer({
			[`GET ${BASE}`]: () => refusal(409, 'survey_closed'),
		})
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		expect(
			await screen.findByRole('heading', { name: 'Questionnaire fermé' }),
		).toBeTruthy()
		expect(
			screen.getByText('Ce questionnaire n’accepte plus de réponses.'),
		).toBeTruthy()
		expect(screen.queryByRole('button')).toBeNull()
		expect(screen.queryByText(/internal message/)).toBeNull()
	})

	it('blocks a second response refused at the first step', async () => {
		installServer(
			happyRoutes({
				[`POST ${BASE}/responses`]: () => refusal(409, 'response_cap_reached'),
			}),
		)
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		await heading()
		await userEvent.click(screen.getByRole('radio', { name: 'Élève' }))
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		expect(
			await screen.findByRole('heading', { name: 'Questionnaire complet' }),
		).toBeTruthy()
	})

	it('says an unknown link is not available and offers a retry when rate limited', async () => {
		installServer({ [`GET ${BASE}`]: () => ({ status: 404 }) })
		const first = renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		expect(
			await screen.findByText(/Ce questionnaire n’est pas disponible/),
		).toBeTruthy()
		first.unmount()

		installServer({ [`GET ${BASE}`]: () => refusal(429) })
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		expect(await screen.findByText(/Patientez une minute/)).toBeTruthy()
		expect(screen.getByRole('button', { name: 'Réessayer' })).toBeTruthy()
	})

	it('shows a retry when a save fails and resends the answers', async () => {
		let failures = 1
		const { calls } = installServer(
			happyRoutes({
				[`PUT ${BASE}/responses/${TOKEN}`]: () => {
					if (failures > 0) {
						failures -= 1
						return refusal(503)
					}
					return data(response())
				},
			}),
		)
		renderWithQuery(<PublicSurveyFeature slug="lien-bilan" />)
		await heading()
		await userEvent.click(screen.getByRole('radio', { name: 'Élève' }))
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		expect(await screen.findByText(/Connexion impossible/)).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: 'Réessayer' }))
		expect(await screen.findByText('Réponses enregistrées')).toBeTruthy()
		expect(calls.filter((call) => call.method === 'PUT')).toHaveLength(2)
	})
})

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { ORGANIZATION } from '#/pages/survey/test/survey-harness'
import { MemberSurveyFeature } from '#/pages/survey-respond/feature/member-survey-feature'
import {
	question,
	respondentSurvey,
} from '#/pages/survey-respond/test/respond-fixtures'
import {
	data,
	installServer,
	refusal,
} from '#/pages/survey-respond/test/respond-server'
import { clearAuth, setAuth } from '#/store/auth.store'
import { installFakePermissionsApi } from '#/test/with-permissions'

const ORG = `/api/v1/organizations/${ORGANIZATION.id}`
const SURVEY = `${ORG}/member-surveys/survey-1`
const RESPONSE = `${ORG}/member-survey-responses/response-9`

const GRAPH = {
	start: 'q-name',
	nodes: [
		question('q-name', 'TEXT', { label: 'Votre prénom', required: true }),
		question('q-date', 'DATE', { label: 'Date du chantier' }),
		{ type: 'END' as const, id: 'end', message: 'Merci, c’est noté.' },
	],
	edges: [
		{ from: 'q-name', to: 'q-date', branch: 'NEXT' as const },
		{ from: 'q-date', to: 'end', branch: 'NEXT' as const },
	],
}

function renderMember() {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})
	return render(
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
					<MemberSurveyFeature surveyId="survey-1" />
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
	)
}

afterEach(() => {
	vi.unstubAllGlobals()
	clearAuth()
})

describe('MemberSurveyFeature', () => {
	it('checks no survey permission itself and lets a member resume, answer and complete through the member routes', async () => {
		installFakePermissionsApi([])
		setAuth({ accessToken: 'member-token', isAuthenticated: true })
		const { calls } = installServer({
			[`GET ${SURVEY}`]: () =>
				data({ ...respondentSurvey(GRAPH), visibility: 'MEMBERS' }),
			[`POST ${SURVEY}/responses`]: () =>
				data({
					id: 'response-9',
					answers: { 'q-name': { Text: 'Léa' } },
					status: 'IN_PROGRESS',
				}),
			[`PUT ${RESPONSE}`]: () =>
				data({ id: 'response-9', answers: {}, status: 'IN_PROGRESS' }),
			[`POST ${RESPONSE}/complete`]: () =>
				data({ id: 'response-9', answers: {}, status: 'COMPLETED' }),
		})
		renderMember()

		expect(await screen.findByLabelText('Date du chantier')).toBeTruthy()
		expect(screen.getByText('Question 2 sur 2')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: /Précédent/ }))
		expect(
			(
				(await screen.findByRole('textbox', {
					name: 'Votre prénom',
				})) as HTMLInputElement
			).value,
		).toBe('Léa')
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		await userEvent.type(
			await screen.findByLabelText('Date du chantier'),
			'2026-11-20',
		)
		await userEvent.click(screen.getByRole('button', { name: 'Terminer' }))
		expect(await screen.findByText('Merci, c’est noté.')).toBeTruthy()

		await waitFor(() => expect(calls.at(-1)?.url).toBe(`${RESPONSE}/complete`))
		const lastSave = calls.filter((call) => call.method === 'PUT').at(-1)
		expect(lastSave?.body).toEqual({
			answers: { 'q-name': { Text: 'Léa' }, 'q-date': { Date: '2026-11-20' } },
		})
		for (const call of calls) {
			expect(call.headers.get('Authorization')).toBe('Bearer member-token')
			expect(call.url.startsWith(ORG)).toBe(true)
		}
	})

	it('blocks with « Déjà répondu » when the member already answered, refused at start', async () => {
		installFakePermissionsApi([])
		installServer({
			[`GET ${SURVEY}`]: () => data(respondentSurvey(GRAPH)),
			[`POST ${SURVEY}/responses`]: () =>
				refusal(409, 'respondent_already_answered'),
		})
		renderMember()
		expect(
			await screen.findByRole('heading', { name: 'Déjà répondu' }),
		).toBeTruthy()
		expect(screen.queryByRole('button')).toBeNull()
	})
})

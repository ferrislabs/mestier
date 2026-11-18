import { fireEvent, screen, waitFor, within } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { installFlowTestEnvironment } from '#/pages/automation/test/flow-test-env'
import { SurveyEditorFeature } from '#/pages/survey/feature/survey-editor-feature'
import { REASON_MESSAGES } from '#/pages/survey/lib/reason-messages'
import {
	installSurveyApi,
	PATHS,
	renderSurveyPage,
	survey,
} from '#/pages/survey/test/survey-harness'
import type { Survey } from '#/pages/survey/types'

const toastMock = vi.hoisted(() => ({
	success: vi.fn(),
	error: vi.fn(),
}))

vi.mock('sonner', () => ({ toast: toastMock }))

beforeEach(() => {
	installFlowTestEnvironment()
	toastMock.success.mockClear()
	toastMock.error.mockClear()
})

const GRAPH: NonNullable<Survey['graph']> = {
	start: 'q1',
	nodes: [
		{
			type: 'QUESTION',
			id: 'q1',
			kind: 'SINGLE_CHOICE',
			label: 'Venez-vous en voiture ?',
			required: true,
			options: ['Oui', 'Non'],
		},
		{
			type: 'CONDITION',
			id: 'c1',
			question: 'q1',
			option: 'Oui',
			checked: true,
		},
		{
			type: 'QUESTION',
			id: 'q2',
			kind: 'TEXT',
			label: 'Immatriculation',
			required: false,
		},
		{ type: 'END', id: 'e1', message: 'Merci' },
	],
	edges: [
		{ from: 'q1', to: 'c1', branch: 'NEXT' },
		{ from: 'c1', to: 'q2', branch: 'THEN' },
		{ from: 'c1', to: 'e1', branch: 'ELSE' },
		{ from: 'q2', to: 'e1', branch: 'NEXT' },
	],
}

const MANAGER = ['VIEW_SURVEYS', 'MANAGE_SURVEYS']
const OPEN_PATH = `${PATHS.survey}/open`
const DUPLICATE_PATH = `${PATHS.survey}/duplicate`

function refusal(status: number, reason: string, questionId?: string) {
	return Object.assign(new Error('Refused'), {
		status,
		data: {
			code: 'E_REFUSED',
			details: { reason, question_id: questionId ?? null },
		},
	})
}

interface PatchCall {
	body: { graph: unknown }
}

function patchBodies(calls: { method: string; params: unknown }[]) {
	return calls
		.filter((call) => call.method === 'patch')
		.map((call) => (call.params as PatchCall).body)
}

async function openDetails(region: string | RegExp) {
	fireEvent.doubleClick(await screen.findByRole('region', { name: region }))
	return screen.findByRole('dialog')
}

async function closeDetails() {
	fireEvent.keyDown(document.activeElement ?? document.body, {
		key: 'Escape',
		code: 'Escape',
	})
	await waitFor(() => expect(screen.queryByRole('dialog')).toBe(null))
}

async function editLabel(region: string, label: string) {
	await openDetails(region)
	fireEvent.change(await screen.findByLabelText('Libellé'), {
		target: { value: label },
	})
}

function flowNode(region: string) {
	const node = screen
		.getByRole('region', { name: region, hidden: true })
		.closest('.react-flow__node')
	if (!node) throw new Error(`no flow node for ${region}`)
	return node as HTMLElement
}

describe('SurveyEditorFeature', () => {
	it('draws the question, condition and end nodes of the graph', async () => {
		installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		expect(
			await screen.findByRole('region', {
				name: 'Question Choix unique : Venez-vous en voiture ?',
			}),
		).toBeTruthy()
		expect(
			screen.getByRole('region', {
				name: 'Condition : Si « Venez-vous en voiture ? » : l’option « Oui » est cochée',
			}),
		).toBeTruthy()
		expect(screen.getByRole('region', { name: 'Fin : Merci' })).toBeTruthy()
		expect(screen.getByText('Parcours complet').getAttribute('title')).toBe(
			'Le parcours est complet : chaque chemin mène à une fin.',
		)
		expect(screen.queryByRole('button', { name: /^Vérification/ })).toBe(null)
	})

	it('lets a manager edit a draft', async () => {
		installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		expect(
			await screen.findByRole('button', { name: 'Ajouter un nœud' }),
		).toBeTruthy()
		expect(
			screen.getByRole('button', { name: 'Ouvrir aux réponses' }),
		).toBeTruthy()
		expect(screen.queryByRole('status')).toBe(null)
	})

	it('shows an opened survey read-only, with a banner and a way to duplicate it', async () => {
		installSurveyApi({
			permissions: MANAGER,
			surveys: [
				survey({
					graph: GRAPH,
					status: 'OPENED',
					public_slug: 'abcd1234',
					opened_at: '2026-05-02T08:00:00Z',
				}),
			],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		expect(
			await screen.findByText(
				/Ce questionnaire est ouvert : sa définition est figée/,
			),
		).toBeTruthy()
		expect(
			screen.getByRole('button', { name: 'Dupliquer pour corriger' }),
		).toBeTruthy()
		expect(screen.queryByRole('button', { name: 'Ajouter un nœud' })).toBe(null)
		expect(screen.queryByRole('button', { name: 'Enregistrer' })).toBe(null)
		expect(screen.queryByRole('button', { name: 'Ouvrir aux réponses' })).toBe(
			null,
		)

		await openDetails('Fin : Merci')
		const message = await screen.findByLabelText('Message de fin')
		expect((message as HTMLTextAreaElement).disabled).toBe(true)
		expect(screen.queryByRole('button', { name: 'Supprimer le nœud' })).toBe(
			null,
		)
	})

	it('flags an invalid draft and keeps it from being opened', async () => {
		const broken = {
			...GRAPH,
			edges: GRAPH.edges.filter((edge) => edge.branch !== 'ELSE'),
		}
		installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: broken })],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		fireEvent.click(
			await screen.findByRole('button', { name: /^Vérification/ }),
		)
		expect(
			await screen.findByText(
				'Ce nœud n’a pas de suite : reliez chacune de ses sorties.',
			),
		).toBeTruthy()
		fireEvent.keyDown(document.activeElement ?? document.body, {
			key: 'Escape',
		})
		await waitFor(() =>
			expect(
				(
					screen.getByRole('button', {
						name: 'Ouvrir aux réponses',
					}) as HTMLButtonElement
				).disabled,
			).toBe(true),
		)
	})

	it('adds the first question from the picker and links it to the start', async () => {
		installSurveyApi({ permissions: MANAGER, surveys: [survey()] })
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		fireEvent.click(
			await screen.findByRole('button', {
				name: /Ajoutez votre première question/,
			}),
		)
		const picker = await screen.findByRole('dialog', {
			name: 'Ajouter un nœud',
		})
		fireEvent.click(within(picker).getByRole('button', { name: 'E-mail' }))

		expect(
			await screen.findByRole('dialog', { name: 'E-mail sans libellé' }),
		).toBeTruthy()
		expect(screen.getByLabelText(/^Identifiant du répondant/)).toBeTruthy()
		await closeDetails()
		expect(
			screen.getByRole('region', {
				name: 'Question E-mail : Question sans libellé',
			}),
		).toBeTruthy()
		expect(
			screen.getByRole('region', { name: 'Départ du questionnaire' }),
		).toBeTruthy()
		await waitFor(() =>
			expect(
				document.querySelector('.react-flow__edge[data-id="__start__:START"]'),
			).not.toBe(null),
		)
	})

	it('sends the backend graph contract on save', async () => {
		const api = installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		api.mockMutation('patch', PATHS.survey, () => ({
			data: survey({ graph: GRAPH }),
		}))
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		await editLabel('Question Texte court : Immatriculation', 'Plaque')
		await closeDetails()
		fireEvent.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => expect(patchBodies(api.calls)).toHaveLength(1))
		const graph = patchBodies(api.calls)[0].graph as NonNullable<
			Survey['graph']
		>
		expect(graph.start).toBe('q1')
		expect(graph.nodes.map((node) => [node.id, node.type])).toEqual([
			['q1', 'QUESTION'],
			['c1', 'CONDITION'],
			['q2', 'QUESTION'],
			['e1', 'END'],
		])
		expect(graph.nodes.some((node) => node.id === '__start__')).toBe(false)
		expect(graph.edges.map((edge) => edge.branch).sort()).toEqual([
			'ELSE',
			'NEXT',
			'NEXT',
			'THEN',
		])
		expect(graph.nodes[2]).toMatchObject({ label: 'Plaque' })
	})

	it('sends a null graph once the canvas is emptied', async () => {
		const small: NonNullable<Survey['graph']> = {
			start: 'q2',
			nodes: [GRAPH.nodes[2], GRAPH.nodes[3]],
			edges: [{ from: 'q2', to: 'e1', branch: 'NEXT' }],
		}
		const api = installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: small })],
		})
		api.mockMutation('patch', PATHS.survey, () => ({ data: survey() }))
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		for (const region of [
			'Question Texte court : Immatriculation',
			'Fin : Merci',
		]) {
			await openDetails(region)
			fireEvent.click(
				await screen.findByRole('button', { name: 'Supprimer le nœud' }),
			)
			await waitFor(() => expect(screen.queryByRole('dialog')).toBe(null))
		}
		fireEvent.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => expect(patchBodies(api.calls)).toHaveLength(1))
		expect(patchBodies(api.calls)[0].graph).toBe(null)
	})

	it('pins a graph refusal of the server on its node', async () => {
		const api = installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		api.mockMutation('patch', PATHS.survey, () => {
			throw refusal(422, 'graph_cycle', 'q2')
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		await editLabel('Question Texte court : Immatriculation', 'Plaque')
		await closeDetails()
		fireEvent.click(screen.getByRole('region', { name: 'Fin : Merci' }))
		fireEvent.click(screen.getByRole('button', { name: 'Enregistrer' }))

		const banner = await screen.findAllByText(REASON_MESSAGES.graph_cycle)
		expect(banner.length).toBeGreaterThan(0)
		expect(
			screen.getByLabelText(`À corriger : ${REASON_MESSAGES.graph_cycle}`),
		).toBeTruthy()
		const node = flowNode('Question Texte court : Plaque')
		expect(node.classList.contains('selected')).toBe(true)
		expect(
			screen
				.getByRole('region', {
					name: 'Question Texte court : Plaque',
					hidden: true,
				})
				.getAttribute('data-highlighted'),
		).toBe('true')
		expect(await screen.findByRole('dialog', { name: 'Plaque' })).toBeTruthy()
		expect(
			(screen.getByLabelText('Libellé') as HTMLTextAreaElement).value,
		).toBe('Plaque')
	})

	it('explains in French why the server refuses to open', async () => {
		const api = installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		api.mockMutation('post', OPEN_PATH, () => {
			throw refusal(409, 'survey_closed')
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		const open = await screen.findByRole('button', {
			name: 'Ouvrir aux réponses',
		})
		await waitFor(() =>
			expect((open as HTMLButtonElement).disabled).toBe(false),
		)
		fireEvent.click(open)
		const dialog = await screen.findByRole('dialog')
		fireEvent.click(
			within(dialog).getByRole('button', { name: 'Ouvrir et figer' }),
		)

		await waitFor(() =>
			expect(toastMock.error).toHaveBeenCalledWith(
				REASON_MESSAGES.survey_closed,
				{
					duration: 6000,
				},
			),
		)
	})

	it('toasts the saved questionnaire', async () => {
		const api = installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		api.mockMutation('patch', PATHS.survey, () => ({
			data: survey({ graph: GRAPH }),
		}))
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		await editLabel('Question Texte court : Immatriculation', 'Plaque')
		await closeDetails()
		fireEvent.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() =>
			expect(toastMock.success).toHaveBeenCalledWith(
				'Questionnaire enregistré.',
			),
		)
		expect(screen.queryByText('Questionnaire enregistré.')).toBe(null)
	})

	it('toasts a failed save with the French message', async () => {
		const api = installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		api.mockMutation('patch', PATHS.survey, () => {
			throw refusal(409, 'survey_closed')
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		await editLabel('Question Texte court : Immatriculation', 'Plaque')
		await closeDetails()
		fireEvent.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() =>
			expect(toastMock.error).toHaveBeenCalledWith(
				REASON_MESSAGES.survey_closed,
				{
					duration: 6000,
				},
			),
		)
		expect(toastMock.success).not.toHaveBeenCalled()
	})

	it('removes a selected node and an unrelated selected edge deleted together', async () => {
		installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		const { container } = await renderSurveyPage(
			<SurveyEditorFeature surveyId="survey-1" />,
		)

		fireEvent.click(
			await screen.findByRole('region', {
				name: 'Question Texte court : Immatriculation',
			}),
		)
		const edge = await waitFor(() => {
			const found = container.querySelector(
				'.react-flow__edge[data-id="c1:ELSE"]',
			)
			if (!found) throw new Error('edge not rendered')
			return found
		})
		fireEvent.click(edge)
		fireEvent.keyDown(document.body, { key: 'Delete', code: 'Delete' })
		fireEvent.keyUp(document.body, { key: 'Delete', code: 'Delete' })

		await waitFor(() =>
			expect(
				screen.queryByRole('region', {
					name: 'Question Texte court : Immatriculation',
				}),
			).toBe(null),
		)
		expect(
			container.querySelector('.react-flow__edge[data-id="c1:ELSE"]'),
		).toBe(null)
	})

	it('keeps a second duplicate from being sent while the first runs', async () => {
		const api = installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH, status: 'CLOSED' })],
		})
		api.mockMutation('post', DUPLICATE_PATH, () => new Promise(() => {}))
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		const button = await screen.findByRole('button', {
			name: 'Dupliquer pour corriger',
		})
		fireEvent.click(button)
		await waitFor(() =>
			expect((button as HTMLButtonElement).disabled).toBe(true),
		)
		fireEvent.click(button)
		expect(api.calls.filter((call) => call.method === 'post')).toHaveLength(1)
	})

	it('tells a reader opening a draft that it is read-only', async () => {
		installSurveyApi({
			permissions: ['VIEW_SURVEYS'],
			surveys: [survey({ graph: GRAPH })],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		expect(
			await screen.findByText(
				'Lecture seule : vous n’avez pas la permission de modifier ce questionnaire.',
			),
		).toBeTruthy()
		expect(screen.queryByRole('button', { name: 'Ajouter un nœud' })).toBe(null)
	})

	it('shows a condition question that is not offered as flagged instead of the placeholder', async () => {
		const detached: NonNullable<Survey['graph']> = {
			...GRAPH,
			nodes: [
				...GRAPH.nodes,
				{
					type: 'QUESTION',
					id: 'q9',
					kind: 'SINGLE_CHOICE',
					label: 'Question ailleurs',
					required: false,
					options: ['A', 'B'],
				},
			],
			edges: GRAPH.edges,
		}
		installSurveyApi({
			permissions: MANAGER,
			surveys: [
				survey({
					graph: {
						...detached,
						nodes: detached.nodes.map((node) =>
							node.id === 'c1'
								? { ...node, question: 'q9', option: 'A' }
								: node,
						),
					},
				}),
			],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		await openDetails(/^Condition : /)
		const select = (await screen.findByLabelText(
			'Question testée',
		)) as HTMLSelectElement
		expect(select.value).toBe('q9')
		expect(select.getAttribute('aria-invalid')).toBe('true')
		const chosen = select.selectedOptions[0]
		expect(chosen.disabled).toBe(true)
		expect(chosen.textContent).toContain('Question ailleurs')
	})
})

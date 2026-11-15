import { fireEvent, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { dragAnnouncements } from '#/pages/survey/feature/sortable-question-list'
import { SurveyEditorFeature } from '#/pages/survey/feature/survey-editor-feature'
import { conditionNodeId } from '#/pages/survey/lib/form-graph'
import { REASON_MESSAGES } from '#/pages/survey/lib/reason-messages'
import {
	campaign,
	installSurveyApi,
	PATHS,
	renderSurveyPage,
	survey,
} from '#/pages/survey/test/survey-harness'
import type { Survey } from '#/pages/survey/types'

const toastMock = vi.hoisted(() => ({
	success: vi.fn(),
	error: vi.fn(),
	info: vi.fn(),
}))

vi.mock('sonner', () => ({ toast: toastMock }))

beforeEach(() => {
	toastMock.success.mockClear()
	toastMock.error.mockClear()
	toastMock.info.mockClear()
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

function refusal(status: number, reason: string, questionId?: string) {
	return Object.assign(new Error('Refused'), {
		status,
		data: {
			code: 'E_REFUSED',
			details: { reason, question_id: questionId ?? null },
		},
	})
}

function patchBodies(calls: { method: string; params: unknown }[]) {
	return calls
		.filter((call) => call.method === 'patch')
		.map(
			(call) =>
				(
					call.params as {
						body: { graph?: Survey['graph']; visibility?: string }
					}
				).body,
		)
}

function card(number: number) {
	return screen.getByRole('article', {
		name: new RegExp(`^Question ${number}\\b`),
	})
}

function saveButton() {
	return screen.getByRole('button', {
		name: 'Enregistrer',
	}) as HTMLButtonElement
}

async function pickKind(trigger: HTMLElement, title: string) {
	await userEvent.click(trigger)
	await userEvent.click(
		within(await screen.findByRole('menu')).getByRole('menuitem', {
			name: new RegExp(`^${title}`),
		}),
	)
}

describe('SurveyEditorFeature', () => {
	it('lists the questions of the graph as cards followed by the end message', async () => {
		installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		const first = await screen.findByLabelText('Libellé de la question 1')
		expect((first as HTMLTextAreaElement).value).toBe('Venez-vous en voiture ?')
		expect(
			(screen.getByLabelText('Libellé de la question 2') as HTMLTextAreaElement)
				.value,
		).toBe('Immatriculation')
		expect(within(card(2)).getByText('Sous condition')).toBeTruthy()
		expect(
			within(card(2)).getByText('Afficher cette question seulement si'),
		).toBeTruthy()
		expect(
			within(card(2)).getByText(
				'Seules les questions toujours posées peuvent servir de condition.',
			),
		).toBeTruthy()
		expect(
			(screen.getByLabelText('Message de fin') as HTMLTextAreaElement).value,
		).toBe('Merci')
		expect(screen.getByText('Prêt')).toBeTruthy()
		expect(
			screen.getByRole('button', { name: 'Ouvrir aux réponses' }),
		).toBeTruthy()
	})

	it('adds a choice question without options, focuses it and blocks the save until it is complete', async () => {
		const api = installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		api.mockMutation('patch', PATHS.survey, () => ({
			data: survey({ graph: GRAPH }),
		}))
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		await pickKind(
			await screen.findByRole('button', { name: 'Ajouter une question' }),
			'Choix multiple',
		)
		const label = await screen.findByLabelText('Libellé de la question 3')
		await waitFor(() => expect(document.activeElement).toBe(label))
		expect(
			within(card(3)).getByText('Aucune option pour le moment.'),
		).toBeTruthy()
		expect(
			screen.getByRole('button', { name: '2 problèmes à corriger' }),
		).toBeTruthy()
		const reason = screen.getByText(
			'Corrigez les problèmes signalés avant d’enregistrer.',
		)
		expect(saveButton().disabled).toBe(false)
		expect(saveButton().getAttribute('aria-disabled')).toBe('true')
		expect(saveButton().getAttribute('aria-describedby')).toBe(reason.id)
		saveButton().focus()
		expect(document.activeElement).toBe(saveButton())
		await userEvent.click(saveButton())
		expect(patchBodies(api.calls)).toHaveLength(0)
		await waitFor(() => expect(document.activeElement).toBe(label))

		fireEvent.change(label, { target: { value: 'Équipements' } })
		await userEvent.click(
			within(card(3)).getByRole('button', { name: 'Ajouter une option' }),
		)
		const option = await within(card(3)).findByLabelText('Option 1')
		await waitFor(() => expect(document.activeElement).toBe(option))
		fireEvent.change(option, { target: { value: 'Casque' } })
		expect(screen.getByText('Prêt')).toBeTruthy()
		await userEvent.click(saveButton())

		await waitFor(() => expect(patchBodies(api.calls)).toHaveLength(1))
		expect(Object.keys(patchBodies(api.calls)[0])).toEqual(['graph'])
		const graph = patchBodies(api.calls)[0].graph
		if (!graph) throw new Error('graph expected')
		const condition = conditionNodeId('q2', 0)
		expect(graph.start).toBe('q1')
		expect(graph.nodes.map((node) => node.type)).toEqual([
			'QUESTION',
			'CONDITION',
			'QUESTION',
			'QUESTION',
			'END',
		])
		expect(graph.nodes[1]).toMatchObject({ id: condition, option: 'Oui' })
		expect(graph.nodes[3]).toMatchObject({
			kind: 'MULTIPLE_CHOICE',
			label: 'Équipements',
			options: ['Casque'],
		})
		const added = graph.nodes[3].id
		expect(graph.edges).toEqual(
			expect.arrayContaining([
				{ from: condition, to: added, branch: 'ELSE' },
				{ from: 'q2', to: added, branch: 'NEXT' },
				{ from: added, to: 'e1', branch: 'NEXT' },
			]),
		)
		expect(toastMock.success).toHaveBeenCalledWith('Questionnaire enregistré.')
	})

	it('keeps an opened survey editable with a calm banner and a frozen visibility', async () => {
		const api = installSurveyApi({
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
		api.mockMutation('patch', PATHS.survey, () => ({ data: survey() }))
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		expect(
			await screen.findByText(
				/Ce questionnaire est ouvert\. Vos modifications/,
			),
		).toBeTruthy()
		expect(screen.queryByText(/Dupliquer pour corriger/)).toBe(null)
		expect(screen.queryByRole('button', { name: 'Ouvrir aux réponses' })).toBe(
			null,
		)
		expect(screen.getByRole('button', { name: 'Fermer' })).toBeTruthy()
		const label = screen.getByLabelText(
			'Libellé de la question 2',
		) as HTMLTextAreaElement
		expect(label.disabled).toBe(false)
		fireEvent.change(label, { target: { value: 'Plaque' } })
		await userEvent.click(saveButton())
		await waitFor(() => expect(patchBodies(api.calls)).toHaveLength(1))
		expect('visibility' in patchBodies(api.calls)[0]).toBe(false)

		await userEvent.click(screen.getByRole('button', { name: 'Réglages' }))
		expect(
			await screen.findAllByText(
				'La visibilité d’un questionnaire ouvert ne peut pas changer : dupliquez-le.',
			),
		).not.toHaveLength(0)
	})

	it('refuses to move a condition source below the question that depends on it', async () => {
		installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		await userEvent.click(
			await screen.findByRole('button', { name: 'Descendre la question 1' }),
		)
		expect(toastMock.error).toHaveBeenCalledWith(
			expect.stringMatching(/^Déplacement refusé/),
			{ duration: 6000 },
		)
		expect(
			(screen.getByLabelText('Libellé de la question 1') as HTMLTextAreaElement)
				.value,
		).toBe('Venez-vous en voiture ?')
	})

	it('moves a question with the keyboard and announces its new position', async () => {
		const linear: NonNullable<Survey['graph']> = {
			start: 'q2',
			nodes: [
				GRAPH.nodes[2],
				{
					type: 'QUESTION',
					id: 'q3',
					kind: 'DATE',
					label: 'Date',
					required: false,
				},
				GRAPH.nodes[3],
			],
			edges: [
				{ from: 'q2', to: 'q3', branch: 'NEXT' },
				{ from: 'q3', to: 'e1', branch: 'NEXT' },
			],
		}
		installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: linear })],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		const handle = await screen.findByRole('button', {
			name: /^Réordonner la question 2/,
		})
		fireEvent.keyDown(handle, { key: 'ArrowUp' })
		expect(
			await screen.findByText('Question déplacée en position 1 sur 2.'),
		).toBeTruthy()
		expect(
			(screen.getByLabelText('Libellé de la question 1') as HTMLTextAreaElement)
				.value,
		).toBe('Date')
	})

	it('asks before deleting a condition source and removes the dependent rule', async () => {
		installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		await userEvent.click(
			await screen.findByRole('button', { name: 'Supprimer la question 1' }),
		)
		const dialog = await screen.findByRole('dialog', {
			name: 'Supprimer cette question ?',
		})
		await userEvent.click(
			within(dialog).getByRole('button', { name: 'Supprimer la question' }),
		)
		await waitFor(() =>
			expect(
				(
					screen.getByLabelText(
						'Libellé de la question 1',
					) as HTMLTextAreaElement
				).value,
			).toBe('Immatriculation'),
		)
		expect(screen.queryByText('Sous condition')).toBe(null)
	})

	it('pins a backend refusal on the card of its question', async () => {
		const api = installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		api.mockMutation('patch', PATHS.survey, () => {
			throw refusal(422, 'graph_question_label_empty', 'q2')
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		fireEvent.change(await screen.findByLabelText('Libellé de la question 2'), {
			target: { value: 'Plaque' },
		})
		await userEvent.click(saveButton())
		expect(
			await within(card(2)).findByText(
				REASON_MESSAGES.graph_question_label_empty,
			),
		).toBeTruthy()
		expect(toastMock.error).toHaveBeenCalled()
	})

	it('shows an advanced graph as a read-only summary and keeps it untouched', async () => {
		const advanced: NonNullable<Survey['graph']> = {
			...GRAPH,
			nodes: [
				...GRAPH.nodes,
				{
					type: 'QUESTION',
					id: 'q3',
					kind: 'DATE',
					label: 'Date',
					required: false,
				},
			],
			edges: [
				{ from: 'q1', to: 'c1', branch: 'NEXT' },
				{ from: 'c1', to: 'q2', branch: 'THEN' },
				{ from: 'c1', to: 'e1', branch: 'ELSE' },
				{ from: 'q2', to: 'q3', branch: 'NEXT' },
				{ from: 'q3', to: 'e1', branch: 'NEXT' },
			],
		}
		const api = installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: advanced })],
		})
		api.mockMutation('patch', PATHS.survey, () => ({ data: survey() }))
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		expect(
			await screen.findByText(
				'Ce questionnaire utilise un parcours avancé qui ne peut pas être modifié ici.',
			),
		).toBeTruthy()
		expect(screen.getByText(/Il contient 3 questions\./)).toBeTruthy()
		expect(screen.queryByRole('button', { name: 'Ajouter une question' })).toBe(
			null,
		)
		fireEvent.change(screen.getByLabelText('Titre du questionnaire'), {
			target: { value: 'Nouveau titre' },
		})
		await userEvent.click(saveButton())
		await waitFor(() => expect(patchBodies(api.calls)).toHaveLength(1))
		expect('graph' in patchBodies(api.calls)[0]).toBe(false)

		await userEvent.click(screen.getByRole('switch', { name: 'Aperçu' }))
		expect(
			await screen.findByLabelText('Venez-vous en voiture ?', { exact: false }),
		).toBeTruthy()
	})

	it('previews the unsaved form and lists the problems instead when it is broken', async () => {
		installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		fireEvent.change(await screen.findByLabelText('Libellé de la question 1'), {
			target: { value: 'Avez-vous une voiture ?' },
		})
		await userEvent.click(screen.getByRole('switch', { name: 'Aperçu' }))
		const preview = await screen.findByRole('region', { name: 'Aperçu' })
		await userEvent.click(await within(preview).findByLabelText('Oui'))
		await userEvent.click(
			within(preview).getByRole('button', { name: /Suivant/ }),
		)
		expect(
			await within(preview).findByLabelText('Immatriculation'),
		).toBeTruthy()
		await userEvent.click(screen.getByRole('switch', { name: 'Aperçu' }))
		await waitFor(() =>
			expect(screen.queryByRole('region', { name: 'Aperçu' })).toBe(null),
		)

		fireEvent.change(screen.getByLabelText('Libellé de la question 2'), {
			target: { value: '' },
		})
		await userEvent.click(screen.getByRole('switch', { name: 'Aperçu' }))
		const broken = await screen.findByRole('region', { name: 'Aperçu' })
		expect(
			within(broken).getByText('Corrigez ces problèmes pour voir l’aperçu'),
		).toBeTruthy()
		expect(
			within(broken).getByText(REASON_MESSAGES.graph_question_label_empty),
		).toBeTruthy()
	})

	it('shows everything read-only to a reader', async () => {
		installSurveyApi({
			permissions: ['VIEW_SURVEYS'],
			surveys: [survey({ graph: GRAPH })],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		expect(
			await screen.findByText(/Lecture seule : vous n’avez pas la permission/),
		).toBeTruthy()
		expect(
			(screen.getByLabelText('Libellé de la question 1') as HTMLTextAreaElement)
				.disabled,
		).toBe(true)
		expect(screen.queryByRole('button', { name: 'Enregistrer' })).toBe(null)
		expect(screen.queryByRole('button', { name: 'Ajouter une question' })).toBe(
			null,
		)
	})

	it('sends only the renamed title on a settings-only save', async () => {
		const api = installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH, campaign_ids: ['campaign-1'] })],
			campaigns: [campaign()],
		})
		api.mockMutation('patch', PATHS.survey, () => ({ data: survey() }))
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		fireEvent.change(await screen.findByLabelText('Titre du questionnaire'), {
			target: { value: 'Bilan de juin' },
		})
		await userEvent.click(saveButton())
		await waitFor(() => expect(patchBodies(api.calls)).toHaveLength(1))
		expect(patchBodies(api.calls)[0]).toEqual({ title: 'Bilan de juin' })
	})

	it('sends the full campaign list once the campaigns changed, and only then', async () => {
		const api = installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH, campaign_ids: ['campaign-1'] })],
			campaigns: [
				campaign(),
				campaign({ id: 'campaign-2', name: 'Session de juin' }),
			],
		})
		api.mockMutation('patch', PATHS.survey, () => ({ data: survey() }))
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		await userEvent.click(
			await screen.findByRole('button', { name: 'Réglages' }),
		)
		await userEvent.click(await screen.findByLabelText('Session de juin'))
		await userEvent.keyboard('{Escape}')
		await userEvent.click(saveButton())
		await waitFor(() => expect(patchBodies(api.calls)).toHaveLength(1))
		expect(patchBodies(api.calls)[0]).toEqual({
			campaign_ids: ['campaign-1', 'campaign-2'],
		})

		fireEvent.change(screen.getByLabelText('Libellé de la question 2'), {
			target: { value: 'Plaque' },
		})
		await userEvent.click(saveButton())
		await waitFor(() => expect(patchBodies(api.calls)).toHaveLength(2))
		expect(Object.keys(patchBodies(api.calls)[1])).toEqual(['graph'])
	})

	it('keeps the typed options across kind changes and asks before dropping dependent conditions', async () => {
		installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		await pickKind(
			await screen.findByRole('button', {
				name: 'Type de la question 2 : Texte court',
			}),
			'Choix unique',
		)
		expect(screen.queryByRole('dialog')).toBe(null)
		await userEvent.click(
			within(card(2)).getByRole('button', { name: 'Ajouter une option' }),
		)
		fireEvent.change(await within(card(2)).findByLabelText('Option 1'), {
			target: { value: 'AB-123-CD' },
		})
		await pickKind(
			screen.getByRole('button', {
				name: 'Type de la question 2 : Choix unique',
			}),
			'Texte court',
		)
		expect(screen.queryByRole('dialog')).toBe(null)
		await pickKind(
			screen.getByRole('button', {
				name: 'Type de la question 2 : Texte court',
			}),
			'Choix unique',
		)
		expect(
			(within(card(2)).getByLabelText('Option 1') as HTMLInputElement).value,
		).toBe('AB-123-CD')

		await pickKind(
			screen.getByRole('button', {
				name: 'Type de la question 1 : Choix unique',
			}),
			'Texte court',
		)
		const dialog = await screen.findByRole('dialog', {
			name: 'Changer le type de cette question ?',
		})
		expect(screen.getByText('Sous condition')).toBeTruthy()
		await userEvent.click(
			within(dialog).getByRole('button', { name: 'Changer le type' }),
		)
		await waitFor(() => expect(screen.queryByText('Sous condition')).toBe(null))
		expect(toastMock.info).toHaveBeenCalledWith(
			'Une condition qui en dépendait a été retirée.',
		)
		await pickKind(
			screen.getByRole('button', {
				name: 'Type de la question 1 : Texte court',
			}),
			'Choix unique',
		)
		expect(
			(within(card(1)).getByLabelText('Option 2') as HTMLInputElement).value,
		).toBe('Non')
	})

	it('shows the all-or-any choice only once a rule has two conditions', async () => {
		installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		await screen.findByLabelText('Libellé de la question 2')
		expect(within(card(2)).queryByLabelText('Conditions à remplir')).toBe(null)
		expect(
			within(card(2)).getByText('la condition suivante est remplie :'),
		).toBeTruthy()
		await userEvent.click(
			within(card(2)).getByRole('button', { name: 'Ajouter une condition' }),
		)
		expect(within(card(2)).getByLabelText('Conditions à remplir')).toBeTruthy()
	})

	it('blocks saving the end message of an empty draft with a visible reason', async () => {
		const api = installSurveyApi({
			permissions: MANAGER,
			surveys: [survey()],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		fireEvent.change(await screen.findByLabelText('Message de fin'), {
			target: { value: 'Merci beaucoup' },
		})
		expect(
			screen.getByText(
				'Ajoutez une question pour enregistrer le message de fin',
			),
		).toBeTruthy()
		expect(screen.getByText('Modifications non enregistrées')).toBeTruthy()
		expect(saveButton().disabled).toBe(false)
		expect(saveButton().getAttribute('aria-disabled')).toBe('true')
		await userEvent.click(saveButton())
		expect(patchBodies(api.calls)).toHaveLength(0)
		await waitFor(() =>
			expect(document.activeElement).toBe(
				screen.getByRole('button', { name: 'Ajouter une question' }),
			),
		)
	})

	it('picks a question up with the keyboard sensor and announces it in French', async () => {
		installSurveyApi({
			permissions: MANAGER,
			surveys: [survey({ graph: GRAPH })],
		})
		await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)

		const handle = await screen.findByRole('button', {
			name: /^Réordonner la question 2/,
		})
		expect(handle.getAttribute('aria-roledescription')).toBe('sortable')
		handle.focus()
		expect(
			document.getElementById(handle.getAttribute('aria-describedby') ?? '')
				?.textContent,
		).toMatch(/^Pour déplacer une question, appuyez sur Espace/)
		fireEvent.keyDown(handle, { code: 'Space', key: ' ' })
		const live = () =>
			document.querySelector('[id^="DndLiveRegion"]')?.textContent ?? ''
		await waitFor(() =>
			expect(live()).toMatch(/^Question 2 (saisie|au-dessus de la position)/),
		)
		fireEvent.keyDown(handle, { code: 'Escape', key: 'Escape' })
		await waitFor(() =>
			expect(live()).toBe(
				'Déplacement annulé : la question 2 reste en position 2.',
			),
		)
		expect(
			(screen.getByLabelText('Libellé de la question 1') as HTMLTextAreaElement)
				.value,
		).toBe('Venez-vous en voiture ?')
	})
})

describe('dragAnnouncements', () => {
	const form = {
		endId: 'e1',
		endMessage: '',
		questions: [
			{
				id: 'a',
				kind: 'SINGLE_CHOICE' as const,
				label: 'A',
				required: false,
				options: ['Oui'],
				maxPhotos: null,
				identifier: false,
				rule: null,
			},
			{
				id: 'b',
				kind: 'TEXT' as const,
				label: 'B',
				required: false,
				options: [],
				maxPhotos: null,
				identifier: false,
				rule: null,
			},
			{
				id: 'c',
				kind: 'TEXT' as const,
				label: 'C',
				required: false,
				options: [],
				maxPhotos: null,
				identifier: false,
				rule: {
					join: 'ALL' as const,
					conditions: [{ question: 'a', option: 'Oui', checked: true }],
				},
			},
		],
	}
	const node = (id: string) => ({ id }) as never

	it('announces a drop in French and the refusal of a dependant above its source', () => {
		const announcements = dragAnnouncements(form)
		expect(
			announcements.onDragEnd({ active: node('c'), over: node('b') }),
		).toBe('Question 3 déplacée en position 2')
		expect(
			announcements.onDragEnd({ active: node('c'), over: node('a') }),
		).toMatch(/^Déplacement refusé/)
	})
})

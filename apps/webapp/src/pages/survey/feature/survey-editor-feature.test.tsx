import { fireEvent, screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it } from 'vitest'
import { installFlowTestEnvironment } from '#/pages/automation/test/flow-test-env'
import { SurveyEditorFeature } from '#/pages/survey/feature/survey-editor-feature'
import {
	installSurveyApi,
	renderSurveyPage,
	survey,
} from '#/pages/survey/test/survey-harness'
import type { Survey } from '#/pages/survey/types'

beforeEach(() => {
	installFlowTestEnvironment()
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
				name: 'Condition : Si « Venez-vous en voiture ? » = « Oui » est coché',
			}),
		).toBeTruthy()
		expect(screen.getByRole('region', { name: 'Fin : Merci' })).toBeTruthy()
		expect(
			screen.getByText(
				'Le parcours est complet : chaque chemin mène à une fin.',
			),
		).toBeTruthy()
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

		fireEvent.click(screen.getByRole('region', { name: 'Fin : Merci' }))
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

		expect(
			await screen.findByText(
				'Ce nœud n’a pas de suite : reliez chacune de ses sorties.',
			),
		).toBeTruthy()
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
			await screen.findByRole('button', { name: 'Ajouter une question' }),
		)
		fireEvent.click(screen.getByRole('button', { name: 'E-mail' }))

		expect(
			await screen.findByRole('region', {
				name: 'Question E-mail : Question sans libellé',
			}),
		).toBeTruthy()
		expect(screen.getByText('Départ', { selector: 'span' })).toBeTruthy()
		expect(screen.getByLabelText(/^Identifiant du répondant/)).toBeTruthy()
	})
})

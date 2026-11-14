import { fireEvent, screen, waitFor, within } from '@testing-library/react'
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

const BROKEN: NonNullable<Survey['graph']> = {
	...GRAPH,
	edges: GRAPH.edges.filter((edge) => edge.branch !== 'ELSE'),
}

const MANAGER = ['VIEW_SURVEYS', 'MANAGE_SURVEYS']
const Q1 = 'Question Choix unique : Venez-vous en voiture ?'
const Q2 = 'Question Texte court : Immatriculation'

async function renderEditor(
	graph: Survey['graph'],
	patch: Partial<Survey> = {},
	permissions = MANAGER,
) {
	installSurveyApi({ permissions, surveys: [survey({ graph, ...patch })] })
	await renderSurveyPage(<SurveyEditorFeature surveyId="survey-1" />)
}

async function openDetails(region: string | RegExp) {
	fireEvent.doubleClick(await screen.findByRole('region', { name: region }))
	return screen.findByRole('dialog')
}

function pressEscape() {
	fireEvent.keyDown(document.activeElement ?? document.body, {
		key: 'Escape',
		code: 'Escape',
	})
}

function flowNode(region: string) {
	const node = screen
		.getByRole('region', { name: region, hidden: true })
		.closest('.react-flow__node')
	if (!node) throw new Error(`no flow node for ${region}`)
	return node as HTMLElement
}

function dialogTitle(): string {
	return screen.getByRole('dialog').getAttribute('aria-labelledby')
		? (document.getElementById(
				screen.getByRole('dialog').getAttribute('aria-labelledby') ?? '',
			)?.textContent ?? '')
		: ''
}

describe('questionnaire node details view', () => {
	it('opens a large view on double-click and goes back to the canvas on Escape, the node still selected', async () => {
		await renderEditor(GRAPH)

		const dialog = await openDetails(Q1)
		expect(dialogTitle()).toBe('Venez-vous en voiture ?')
		expect(
			within(dialog).getByRole('region', { name: 'Paramètres' }),
		).toBeTruthy()
		expect(
			within(dialog).getByRole('region', { name: 'Aperçu répondant' }),
		).toBeTruthy()

		pressEscape()
		await waitFor(() => expect(screen.queryByRole('dialog')).toBe(null))
		expect(flowNode(Q1).classList.contains('selected')).toBe(true)
	})

	it('closes with the X button', async () => {
		await renderEditor(GRAPH)
		const dialog = await openDetails(Q2)
		fireEvent.click(
			within(dialog).getByRole('button', {
				name: 'Fermer les détails du nœud',
			}),
		)
		await waitFor(() => expect(screen.queryByRole('dialog')).toBe(null))
	})

	it('opens on Enter, from the focused card and from the selected node', async () => {
		await renderEditor(GRAPH)

		const card = within(
			await screen.findByRole('region', { name: Q2 }),
		).getByRole('button', { name: /Appuyez sur Entrée/ })
		fireEvent.keyDown(card, { key: 'Enter' })
		expect(await screen.findByRole('dialog')).toBeTruthy()
		expect(dialogTitle()).toBe('Immatriculation')
		pressEscape()
		await waitFor(() => expect(screen.queryByRole('dialog')).toBe(null))

		fireEvent.click(screen.getByRole('region', { name: 'Fin : Merci' }))
		fireEvent.keyDown(document.body, { key: 'Enter' })
		expect(await screen.findByRole('dialog')).toBeTruthy()
		expect(dialogTitle()).toBe('Fin')
	})

	it('opens from the node context menu', async () => {
		await renderEditor(GRAPH)
		fireEvent.contextMenu(await screen.findByRole('region', { name: Q2 }))
		fireEvent.click(await screen.findByRole('menuitem', { name: /Ouvrir/ }))
		expect(await screen.findByRole('dialog')).toBeTruthy()
		expect(dialogTitle()).toBe('Immatriculation')
	})

	it('walks the graph with the previous and next controls', async () => {
		await renderEditor(GRAPH)

		await openDetails(Q1)
		expect(screen.queryByRole('navigation', { name: 'Nœuds précédents' })).toBe(
			null,
		)
		fireEvent.click(
			screen.getByRole('button', {
				name: 'Ouvrir le nœud suivant : Condition',
			}),
		)
		await waitFor(() => expect(dialogTitle()).toBe('Condition'))

		const next = screen.getByRole('navigation', { name: 'Nœuds suivants' })
		const targets = within(next)
			.getAllByRole('button')
			.map((button) => button.getAttribute('aria-label'))
		expect(targets).toEqual([
			'Ouvrir le nœud suivant : Immatriculation (branche Oui)',
			'Ouvrir le nœud suivant : Fin (branche Non)',
		])
		expect(
			within(
				screen.getByRole('navigation', { name: 'Nœuds précédents' }),
			).getByRole('button', {
				name: 'Ouvrir le nœud précédent : Venez-vous en voiture ?',
			}),
		).toBeTruthy()

		fireEvent.click(
			within(next).getByRole('button', { name: /Fin \(branche Non\)/ }),
		)
		await waitFor(() => expect(dialogTitle()).toBe('Fin'))
		expect(screen.queryByRole('navigation', { name: 'Nœuds suivants' })).toBe(
			null,
		)
		expect(
			within(
				screen.getByRole('navigation', { name: 'Nœuds précédents' }),
			).getAllByRole('button'),
		).toHaveLength(2)
		expect(flowNode('Fin : Merci').classList.contains('selected')).toBe(true)
	})

	it('keeps unsaved edits in the draft while navigating', async () => {
		await renderEditor(GRAPH)

		await openDetails(Q2)
		fireEvent.change(screen.getByLabelText('Libellé'), {
			target: { value: 'Plaque' },
		})
		fireEvent.click(
			screen.getByRole('button', { name: 'Ouvrir le nœud suivant : Fin' }),
		)
		await waitFor(() => expect(dialogTitle()).toBe('Fin'))
		fireEvent.click(
			screen.getByRole('button', { name: 'Ouvrir le nœud précédent : Plaque' }),
		)
		await waitFor(() => expect(dialogTitle()).toBe('Plaque'))
		expect(
			(screen.getByLabelText('Libellé') as HTMLTextAreaElement).value,
		).toBe('Plaque')
		expect(
			(
				screen.getByRole('button', {
					name: 'Enregistrer',
					hidden: true,
				}) as HTMLButtonElement
			).disabled,
		).toBe(false)
	})

	it('previews the question as the respondent sees it, with the draft label and options', async () => {
		await renderEditor(GRAPH)

		await openDetails(Q1)
		const preview = screen.getByTestId('respondent-preview')
		expect(preview.textContent).toContain('Venez-vous en voiture ?')
		expect(preview.textContent).toContain('Bilan de session')
		expect(
			within(preview).getAllByRole('radio', { hidden: true }),
		).toHaveLength(2)

		fireEvent.change(screen.getByLabelText('Libellé'), {
			target: { value: 'Avez-vous un véhicule ?' },
		})
		fireEvent.change(screen.getByLabelText('Option 2'), {
			target: { value: 'Non, à pied' },
		})
		expect(preview.textContent).toContain('Avez-vous un véhicule ?')
		expect(preview.textContent).toContain('Non, à pied')
	})

	it('previews the end screen and the branches of a condition', async () => {
		await renderEditor(GRAPH)

		await openDetails('Fin : Merci')
		expect(screen.getByTestId('respondent-preview').textContent).toContain(
			'Merci',
		)
		pressEscape()
		await waitFor(() => expect(screen.queryByRole('dialog')).toBe(null))

		await openDetails(/^Condition : /)
		const branches = screen.getByRole('list', {
			name: 'Branches de la condition',
		})
		expect(branches.textContent).toContain('OuiImmatriculation')
		expect(branches.textContent).toContain('NonFin')
	})

	it('opens read-only for a survey that is no longer a draft', async () => {
		await renderEditor(GRAPH, {
			status: 'OPENED',
			public_slug: 'abcd1234',
			opened_at: '2026-05-02T08:00:00Z',
		})

		const dialog = await openDetails(Q1)
		expect(within(dialog).getByText('Lecture seule')).toBeTruthy()
		expect(
			(screen.getByLabelText('Libellé') as HTMLTextAreaElement).disabled,
		).toBe(true)
		expect(
			(screen.getByLabelText('Option 1') as HTMLInputElement).disabled,
		).toBe(true)
		expect(screen.queryByRole('button', { name: 'Ajouter une option' })).toBe(
			null,
		)
		expect(screen.queryByRole('button', { name: 'Supprimer le nœud' })).toBe(
			null,
		)
	})
})

describe('empty questionnaire canvas', () => {
	it('draws the start node and a separate first-question placeholder beside it', async () => {
		await renderEditor(null)

		const start = await screen.findByRole('region', {
			name: 'Départ du questionnaire',
		})
		const placeholder = await screen.findByRole('region', {
			name: 'Première question à ajouter',
		})
		const startNode = start.closest('.react-flow__node')
		const placeholderNode = placeholder.closest('.react-flow__node')
		expect(startNode).not.toBe(null)
		expect(placeholderNode).not.toBe(null)
		expect(startNode).not.toBe(placeholderNode)
		expect(startNode?.contains(placeholderNode ?? null)).toBe(false)
		const prompt = screen.getByText('Ajoutez votre première question')
		expect(start.contains(prompt)).toBe(false)
		expect(placeholder.contains(prompt)).toBe(true)
		expect(
			within(start).queryByRole('button', { name: /Ajouter un nœud après/ }),
		).toBe(null)
		await waitFor(() =>
			expect(
				document.querySelector(
					'.react-flow__edge[data-id="__start__:__first_question__"]',
				),
			).not.toBe(null),
		)

		fireEvent.click(
			within(placeholder).getByRole('button', {
				name: 'Ajoutez votre première question',
			}),
		)
		const picker = await screen.findByRole('dialog', {
			name: 'Ajouter un nœud',
		})
		expect(picker.textContent).toContain('« Départ »')
	})

	it('shows no placeholder on a read-only empty questionnaire', async () => {
		await renderEditor(null, {}, ['VIEW_SURVEYS'])
		expect(
			await screen.findByRole('region', { name: 'Départ du questionnaire' }),
		).toBeTruthy()
		expect(
			screen.queryByRole('region', { name: 'Première question à ajouter' }),
		).toBe(null)
	})
})

describe('questionnaire node creator', () => {
	it('filters on the search and closes on Escape', async () => {
		await renderEditor(GRAPH)

		fireEvent.click(
			await screen.findByRole('button', { name: 'Ajouter un nœud' }),
		)
		const picker = await screen.findByRole('dialog', {
			name: 'Ajouter un nœud',
		})
		expect(
			within(picker).getByRole('region', { name: 'Questions' }),
		).toBeTruthy()
		expect(
			within(picker).getByRole('region', { name: 'Parcours' }),
		).toBeTruthy()

		fireEvent.change(within(picker).getByLabelText('Rechercher un nœud'), {
			target: { value: 'photo' },
		})
		expect(
			within(picker)
				.getAllByRole('button')
				.filter((button) => button.id.startsWith('survey-node-choice-'))
				.map((button) => button.getAttribute('aria-label')),
		).toEqual(['Photo'])

		pressEscape()
		await waitFor(() =>
			expect(screen.queryByRole('dialog', { name: 'Ajouter un nœud' })).toBe(
				null,
			),
		)
	})

	it('picks the highlighted entry with the arrows and Enter, then opens it', async () => {
		await renderEditor(GRAPH)

		fireEvent.click(
			await screen.findByRole('button', { name: 'Ajouter un nœud' }),
		)
		const picker = await screen.findByRole('dialog', {
			name: 'Ajouter un nœud',
		})
		const search = within(picker).getByLabelText('Rechercher un nœud')
		fireEvent.keyDown(search, { key: 'ArrowDown' })
		expect(search.getAttribute('aria-activedescendant')).toBe(
			'survey-node-choice-LONG_TEXT',
		)
		fireEvent.keyDown(search, { key: 'Enter' })

		expect(
			await screen.findByRole('dialog', { name: 'Texte long sans libellé' }),
		).toBeTruthy()
		expect(
			screen.getByRole('region', {
				name: 'Question Texte long : Question sans libellé',
				hidden: true,
			}),
		).toBeTruthy()
	})

	it('connects the new node to the free exit whose « + » was used', async () => {
		await renderEditor(BROKEN)

		fireEvent.click(
			await screen.findByRole('button', {
				name: 'Ajouter un nœud après « Condition » (Non)',
			}),
		)
		const picker = await screen.findByRole('dialog', {
			name: 'Ajouter un nœud',
		})
		expect(picker.textContent).toContain('« Condition (Non) »')
		fireEvent.click(within(picker).getByRole('button', { name: 'Fin' }))

		await waitFor(() =>
			expect(
				document.querySelector('.react-flow__edge[data-id="c1:ELSE"]'),
			).not.toBe(null),
		)
		expect(screen.queryByRole('dialog')).toBe(null)
		expect(
			screen.getByText('Parcours complet').getAttribute('title'),
		).toContain('Le parcours est complet')
	})
})

describe('questionnaire verification', () => {
	it('lists the problems and opens the node of the one picked', async () => {
		await renderEditor(BROKEN)

		expect(
			await screen.findByLabelText(
				'À corriger : Ce nœud n’a pas de suite : reliez chacune de ses sorties.',
			),
		).toBeTruthy()
		fireEvent.click(screen.getByRole('button', { name: /^Vérification/ }))
		const list = await screen.findByRole('region', {
			name: 'Problèmes du questionnaire',
		})
		fireEvent.click(within(list).getByRole('button', { name: /Condition/ }))

		await waitFor(() => expect(dialogTitle()).toBe('Condition'))
		expect(
			screen.getByRole('list', { name: 'Problèmes de ce nœud' }).textContent,
		).toContain('Ce nœud n’a pas de suite')
	})
})

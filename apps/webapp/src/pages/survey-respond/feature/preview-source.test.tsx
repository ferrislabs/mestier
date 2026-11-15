import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useMemo } from 'react'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { previewSource } from '#/pages/survey-respond/feature/respond-sources'
import { useSurveyRespondent } from '#/pages/survey-respond/feature/use-survey-respondent'
import {
	blockGraph,
	branchingGraph,
	photoGraph,
} from '#/pages/survey-respond/test/respond-fixtures'
import { SurveyRespondView } from '#/pages/survey-respond/ui/survey-respond-view'

function Preview({ graph = branchingGraph() }: { graph?: SurveyGraph }) {
	const source = useMemo(
		() =>
			previewSource({
				title: 'Aperçu du bilan',
				description: null,
				graph,
			}),
		[graph],
	)
	const view = useSurveyRespondent(source, {
		queryKey: ['survey-preview-test'],
		convertHeic: async (file) => file,
	})
	return <SurveyRespondView variant="member" view={view} />
}

type SurveyGraph = Schemas.SurveyGraphSchema

const PHOTO_GRAPH = photoGraph(2)
const BLOCK_GRAPH = blockGraph()

describe('previewSource', () => {
	it('runs the respondent flow in memory, branches live and restarts', async () => {
		const fetchSpy = vi.fn()
		vi.stubGlobal('fetch', fetchSpy)
		document.title = 'Éditeur'
		render(
			<QueryClientProvider client={new QueryClient()}>
				<Preview />
			</QueryClientProvider>,
		)

		await userEvent.click(await screen.findByLabelText('Élève'))
		expect(screen.getByText('Aperçu : rien n’est enregistré')).toBeTruthy()
		expect(screen.queryByText('Réponses enregistrées')).toBe(null)
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		await userEvent.type(await screen.findByLabelText('Votre classe'), '3e B')
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		expect(await screen.findByLabelText('Votre e-mail')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: /Précédent/ }))
		expect(
			((await screen.findByLabelText('Votre classe')) as HTMLInputElement)
				.value,
		).toBe('3e B')
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		await userEvent.click(
			await screen.findByRole('button', { name: 'Terminer' }),
		)
		expect(
			await screen.findByRole('heading', { name: 'Fin de l’aperçu' }),
		).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: 'Recommencer' }))
		expect(await screen.findByLabelText('Professeur')).toBeTruthy()
		expect(fetchSpy).not.toHaveBeenCalled()
		expect(document.title).toBe('Éditeur')
		vi.unstubAllGlobals()
	})

	it('revokes the photo previews when the preview restarts', async () => {
		const revoked: string[] = []
		const createObjectURL = URL.createObjectURL
		const revokeObjectURL = URL.revokeObjectURL
		URL.createObjectURL = vi.fn(() => 'blob:preview-1')
		URL.revokeObjectURL = vi.fn((url: string) => {
			revoked.push(url)
		})
		render(
			<QueryClientProvider client={new QueryClient()}>
				<Preview graph={PHOTO_GRAPH} />
			</QueryClientProvider>,
		)

		await screen.findByRole('heading', { name: 'Photographiez votre ouvrage' })
		await userEvent.upload(
			screen.getByLabelText('Choisir des photos'),
			new File(['jpeg-bytes'], 'mur.jpg', { type: 'image/jpeg' }),
		)
		await screen.findByRole('img')
		await userEvent.click(screen.getByRole('button', { name: 'Terminer' }))
		await userEvent.click(
			await screen.findByRole('button', { name: 'Recommencer' }),
		)
		expect(revoked).toEqual(['blob:preview-1'])
		URL.createObjectURL = createObjectURL
		URL.revokeObjectURL = revokeObjectURL
	})

	it('previews a block as one screen', async () => {
		render(
			<QueryClientProvider client={new QueryClient()}>
				<Preview graph={BLOCK_GRAPH} />
			</QueryClientProvider>,
		)
		await userEvent.click(await screen.findByLabelText('Oui'))
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		expect(await screen.findByLabelText('Marque')).toBeTruthy()
		expect(screen.getByLabelText('Immatriculation')).toBeTruthy()
		expect(screen.getByLabelText('E-mail du conducteur')).toBeTruthy()
		expect(screen.getByText('Questions 2 à 4 sur 5')).toBeTruthy()
		await userEvent.type(screen.getByLabelText('Marque'), 'Renault')
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		expect(await screen.findByLabelText('Un commentaire')).toBeTruthy()
	})

	it('previews the descriptions under the questions, smaller on a block screen', async () => {
		const graph = blockGraph()
		const described: SurveyGraph = {
			...graph,
			nodes: graph.nodes.map((node) =>
				node.type === 'QUESTION' && node.id === 'q-car'
					? { ...node, description: 'Même partiellement' }
					: node.type === 'QUESTION' && node.id === 'q-plate'
						? { ...node, description: 'Telle que sur\nla carte grise' }
						: node,
			),
		}
		render(
			<QueryClientProvider client={new QueryClient()}>
				<Preview graph={described} />
			</QueryClientProvider>,
		)
		const first = await screen.findByText('Même partiellement')
		expect(first.className).toContain('text-sm')
		expect(
			screen
				.getByRole('group', { name: 'Venez-vous en voiture ?' })
				.getAttribute('aria-describedby'),
		).toBe(first.id)
		await userEvent.click(screen.getByLabelText('Oui'))
		await userEvent.click(screen.getByRole('button', { name: /Suivant/ }))
		const plate = await screen.findByText(/Telle que sur/)
		expect(plate.textContent).toBe('Telle que sur\nla carte grise')
		expect(plate.className).toContain('text-xs')
		expect(
			screen.getByLabelText('Immatriculation').getAttribute('aria-describedby'),
		).toBe(plate.id)
		expect(
			screen.getByLabelText('Marque').getAttribute('aria-describedby'),
		).toBeNull()
	})
})

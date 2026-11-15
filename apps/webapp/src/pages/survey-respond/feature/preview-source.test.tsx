import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useMemo } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { previewSource } from '#/pages/survey-respond/feature/respond-sources'
import { useSurveyRespondent } from '#/pages/survey-respond/feature/use-survey-respondent'
import { branchingGraph } from '#/pages/survey-respond/test/respond-fixtures'
import { SurveyRespondView } from '#/pages/survey-respond/ui/survey-respond-view'

function Preview() {
	const source = useMemo(
		() =>
			previewSource({
				title: 'Aperçu du bilan',
				description: null,
				graph: branchingGraph(),
			}),
		[],
	)
	const view = useSurveyRespondent(source, {
		queryKey: ['survey-preview-test'],
		convertHeic: async (file) => file,
	})
	return <SurveyRespondView variant="member" view={view} />
}

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
})

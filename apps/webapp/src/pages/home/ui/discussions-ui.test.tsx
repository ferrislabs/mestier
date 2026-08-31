import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { DiscussionsUI } from '#/pages/home/ui/discussions-ui'
import { renderWithRouter } from '#/test/render-with-router'

describe('DiscussionsUI', () => {
	it('lists each channel with its topic when it has one', async () => {
		await renderWithRouter(
			<DiscussionsUI
				organizationSlug="atelier-bois"
				isLoading={false}
				error={null}
				channels={[
					{
						id: 'c1',
						name: 'chantier-dupont',
						topic: 'Coordination du chantier Dupont',
						unread: false,
					},
				]}
			/>,
		)

		expect(screen.getByText('#chantier-dupont')).toBeDefined()
		expect(screen.getByText('Coordination du chantier Dupont')).toBeDefined()
	})

	it('never shows a message snippet — no data source backs one', async () => {
		await renderWithRouter(
			<DiscussionsUI
				organizationSlug="atelier-bois"
				isLoading={false}
				error={null}
				channels={[{ id: 'c1', name: 'general', topic: null, unread: false }]}
			/>,
		)

		expect(screen.getByText('#general')).toBeDefined()
		expect(screen.queryByText(/Livraison|reporté/)).toBeNull()
	})

	it('says outright when there is no channel', async () => {
		await renderWithRouter(
			<DiscussionsUI
				organizationSlug="atelier-bois"
				isLoading={false}
				error={null}
				channels={[]}
			/>,
		)

		expect(
			screen.getByText('Aucun canal de discussion pour le moment.'),
		).toBeDefined()
	})

	it('links through to the chat', async () => {
		await renderWithRouter(
			<DiscussionsUI
				organizationSlug="atelier-bois"
				isLoading={false}
				error={null}
				channels={[]}
			/>,
		)

		const link = screen.getByRole('link', { name: /Ouvrir le chat/ })
		expect(link.getAttribute('href')).toBe('/o/atelier-bois/chat')
	})
})

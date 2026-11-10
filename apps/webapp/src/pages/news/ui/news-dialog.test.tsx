import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { NewsDialog } from './news-dialog'

function news(id: string, extra: Partial<Schemas.NewsResponse> = {}) {
	return {
		id,
		title: `Titre ${id}`,
		items: [{ icon: 'rocket', title: `Point ${id}`, text: 'Une phrase.' }],
		created_at: '2026-11-01T00:00:00Z',
		updated_at: '2026-11-01T00:00:00Z',
		...extra,
	} as Schemas.NewsResponse
}

function setup(list: Schemas.NewsResponse[], index = 0) {
	const onIndexChange = vi.fn()
	const onClose = vi.fn()
	render(
		<NewsDialog
			open
			news={list}
			index={index}
			imageUrls={{ k1: 'https://img/1.png' }}
			onIndexChange={onIndexChange}
			onClose={onClose}
		/>,
	)
	return { onIndexChange, onClose }
}

describe('NewsDialog', () => {
	it('shows no carousel controls for a single news', () => {
		setup([news('a')])
		expect(screen.getByText('Titre a')).toBeTruthy()
		expect(screen.queryByLabelText('Nouveauté suivante')).toBeNull()
		expect(screen.queryByLabelText('Nouveauté 1 sur 1')).toBeNull()
	})

	it('falls back to a gradient banner without image', () => {
		setup([news('a')])
		expect(screen.getByTestId('news-banner-fallback')).toBeTruthy()
	})

	it('shows the image when a key resolves', () => {
		setup([news('a', { image_key: 'k1' })])
		expect(screen.queryByTestId('news-banner-fallback')).toBeNull()
		expect(document.querySelector('img')?.getAttribute('src')).toBe(
			'https://img/1.png',
		)
	})

	it('renders the cta as a safe external link', () => {
		setup([news('a', { cta: { label: 'Lire', url: 'https://x.fr/a' } })])
		const link = screen.getByRole('link', { name: 'Lire' })
		expect(link.getAttribute('href')).toBe('https://x.fr/a')
		expect(link.getAttribute('target')).toBe('_blank')
		expect(link.getAttribute('rel')).toBe('noopener noreferrer')
	})

	it('navigates with buttons, dots and keys', async () => {
		const user = userEvent.setup()
		const { onIndexChange } = setup([news('a'), news('b'), news('c')], 1)
		await user.click(screen.getByLabelText('Nouveauté suivante'))
		expect(onIndexChange).toHaveBeenLastCalledWith(2)
		await user.click(screen.getByLabelText('Nouveauté précédente'))
		expect(onIndexChange).toHaveBeenLastCalledWith(0)
		await user.click(screen.getByLabelText('Nouveauté 3 sur 3'))
		expect(onIndexChange).toHaveBeenLastCalledWith(2)
		await user.keyboard('{ArrowRight}')
		expect(onIndexChange).toHaveBeenLastCalledWith(2)
		await user.keyboard('{ArrowLeft}')
		expect(onIndexChange).toHaveBeenLastCalledWith(0)
	})

	it('disables previous on the first and next on the last', () => {
		setup([news('a'), news('b')], 0)
		expect(
			(screen.getByLabelText('Nouveauté précédente') as HTMLButtonElement)
				.disabled,
		).toBe(true)
	})

	it('closes from the close button', async () => {
		const user = userEvent.setup()
		const { onClose } = setup([news('a')])
		await user.click(screen.getByLabelText('Fermer'))
		expect(onClose).toHaveBeenCalled()
	})
})

import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { PageHeader } from '#/components/ui/surface'

describe('PageHeader', () => {
	it('wraps the title block and the actions on one line that ends at the bottom edge', () => {
		render(<PageHeader title="Workflows" actions={<button type="button" />} />)

		const classes = screen.getByRole('banner').className.split(' ')
		for (const expected of [
			'flex',
			'flex-wrap',
			'items-end',
			'justify-between',
			'gap-4',
		]) {
			expect(classes).toContain(expected)
		}
		expect(classes).not.toContain('flex-col')
		expect(classes).not.toContain('sm:flex-row')
	})

	it('lets a long title wrap instead of truncating it', () => {
		render(<PageHeader title="Workflows" />)

		const classes = screen
			.getByRole('heading', { level: 1 })
			.className.split(' ')
		expect(classes).not.toContain('truncate')
	})

	it('sets the description at the text-sm line height without a width cap', () => {
		render(<PageHeader title="Workflows" description="Automatisez." />)

		const classes = screen.getByText('Automatisez.').className.split(' ')
		expect(classes).toContain('mt-1')
		expect(classes).toContain('text-sm')
		expect(classes).not.toContain('leading-6')
		expect(classes).not.toContain('max-w-2xl')
	})

	it('lays the actions out in a row with a small gap', () => {
		render(
			<PageHeader
				title="Workflows"
				actions={
					<>
						<button type="button">A</button>
						<button type="button">B</button>
					</>
				}
			/>,
		)

		const wrapper = screen.getByRole('button', { name: 'A' }).parentElement
		const classes = wrapper?.className.split(' ') ?? []
		expect(classes).toContain('flex')
		expect(classes).toContain('items-center')
		expect(classes).toContain('gap-2')
	})
})

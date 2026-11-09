import { render, waitFor } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { RichTextView } from './rich-text-view'

describe('RichTextView', () => {
	it('renders a list and a link', async () => {
		const { container } = render(
			<RichTextView
				markdown={'- un\n- deux\n\n[Mestier](https://mestier.app)'}
			/>,
		)
		await waitFor(() => {
			expect(container.querySelectorAll('ul > li')).toHaveLength(2)
		})
		expect(container.querySelector('a')?.getAttribute('href')).toBe(
			'https://mestier.app',
		)
		expect(
			container.querySelector('.ProseMirror')?.getAttribute('contenteditable'),
		).toBe('false')
	})

	it('renders nothing for empty markdown', () => {
		const { container } = render(<RichTextView markdown="  " />)
		expect(container.firstChild).toBeNull()
	})

	it('drops raw html and unsafe links', async () => {
		const { container } = render(
			<RichTextView
				markdown={'<script>alert(1)</script>\n\n[x](javascript:alert(1))'}
			/>,
		)
		await waitFor(() => {
			expect(container.querySelector('.ProseMirror')).toBeTruthy()
		})
		expect(container.querySelector('script')).toBeNull()
		expect(container.querySelector('a[href^="javascript"]')).toBeNull()
	})
})

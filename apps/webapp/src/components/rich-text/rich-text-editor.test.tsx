import { act, render, screen, waitFor } from '@testing-library/react'
import { useState } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { RichTextEditor } from './rich-text-editor'

const SAMPLE =
	'**gras** et *italique*\n\n- un\n- deux\n\n[Mestier](https://mestier.app)'

describe('RichTextEditor', () => {
	it('renders markdown as formatted content', async () => {
		const { container } = render(
			<RichTextEditor
				value={SAMPLE}
				onChange={() => {}}
				aria-label="Mission"
			/>,
		)
		await waitFor(() => {
			expect(container.querySelector('strong')?.textContent).toBe('gras')
		})
		expect(container.querySelector('em')?.textContent).toBe('italique')
		expect(container.querySelectorAll('ul > li')).toHaveLength(2)
		expect(container.querySelector('a')?.getAttribute('href')).toBe(
			'https://mestier.app',
		)
		expect(screen.getByRole('textbox', { name: 'Mission' })).toBeTruthy()
	})

	it('keeps bold, list and link in the markdown emitted on change', async () => {
		const onChange = vi.fn()
		const { container } = render(
			<RichTextEditor value={SAMPLE} onChange={onChange} />,
		)
		await waitFor(() => {
			expect(container.querySelector('strong')).toBeTruthy()
		})
		const pm = container.querySelector('.ProseMirror') as HTMLElement & {
			editor: {
				chain: () => {
					focus: (position: 'end') => {
						insertContent: (text: string) => { run: () => boolean }
					}
				}
			}
		}
		act(() => {
			pm.editor.chain().focus('end').insertContent('x').run()
		})
		await waitFor(() => expect(onChange).toHaveBeenCalled())
		const markdown = onChange.mock.calls.at(-1)?.[0] as string
		expect(markdown).toContain('**gras**')
		expect(markdown).toContain('*italique*')
		expect(markdown).toMatch(/^- un$/m)
		expect(markdown).toMatch(/\[Mestier\w*\]\(https:\/\/mestier\.app\)/)
	})

	it('resets content when the value changes externally without emitting', async () => {
		const onChange = vi.fn()
		const { container, rerender } = render(
			<RichTextEditor value="un" onChange={onChange} />,
		)
		await waitFor(() => {
			expect(container.querySelector('.ProseMirror')?.textContent).toBe('un')
		})
		rerender(<RichTextEditor value="**deux**" onChange={onChange} />)
		await waitFor(() => {
			expect(container.querySelector('strong')?.textContent).toBe('deux')
		})
		expect(onChange).not.toHaveBeenCalled()
	})

	it('works as a controlled field without looping', async () => {
		const seen: string[] = []
		function Harness() {
			const [value, setValue] = useState('- a')
			return (
				<RichTextEditor
					value={value}
					onChange={(next) => {
						seen.push(next)
						setValue(next)
					}}
				/>
			)
		}
		const { container } = render(<Harness />)
		await waitFor(() => {
			expect(container.querySelector('li')).toBeTruthy()
		})
		const pm = container.querySelector('.ProseMirror') as HTMLElement & {
			editor: {
				chain: () => {
					focus: (position: 'end') => {
						insertContent: (text: string) => { run: () => boolean }
					}
				}
			}
		}
		act(() => {
			pm.editor.chain().focus('end').insertContent('b').run()
		})
		await waitFor(() => expect(seen.length).toBe(1))
		expect(container.querySelectorAll('li')).toHaveLength(1)
		expect(seen[0]).toMatch(/^- /)
	})

	it('is not editable when disabled', async () => {
		const { container } = render(
			<RichTextEditor value="a" onChange={() => {}} disabled />,
		)
		await waitFor(() => {
			expect(
				container
					.querySelector('.ProseMirror')
					?.getAttribute('contenteditable'),
			).toBe('false')
		})
	})
})

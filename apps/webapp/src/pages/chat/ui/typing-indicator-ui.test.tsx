import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { TypingIndicatorUI } from './typing-indicator-ui'

describe('TypingIndicatorUI', () => {
	it('renders nothing when nobody is typing', () => {
		const { container } = render(<TypingIndicatorUI typingCount={0} />)
		expect(container.firstChild).toBeNull()
	})

	it('shows the singular form for one typer', () => {
		render(<TypingIndicatorUI typingCount={1} />)
		expect(screen.getByText(/Quelqu’un écrit/)).toBeDefined()
	})

	it('shows a count for several typers', () => {
		render(<TypingIndicatorUI typingCount={2} />)
		expect(screen.getByText(/2 personnes écrivent/)).toBeDefined()
	})
})

import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { PresenceSummaryUI } from './presence-summary-ui'

describe('PresenceSummaryUI', () => {
	it('renders nothing when nobody is online', () => {
		const { container } = render(<PresenceSummaryUI onlineCount={0} />)
		expect(container.firstChild).toBeNull()
	})

	it('shows the singular form for one member', () => {
		render(<PresenceSummaryUI onlineCount={1} />)
		expect(screen.getByText(/1 personne en ligne/)).toBeDefined()
	})

	it('shows the plural form for several members', () => {
		render(<PresenceSummaryUI onlineCount={3} />)
		expect(screen.getByText(/3 personnes en ligne/)).toBeDefined()
	})
})

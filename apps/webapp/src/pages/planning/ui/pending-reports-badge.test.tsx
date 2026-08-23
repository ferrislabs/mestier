import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { PendingReportsBadge } from '#/pages/planning/ui/pending-reports-badge'
import { renderWithRouter } from '#/test/render-with-router'

describe('PendingReportsBadge', () => {
	it('renders nothing at zero', async () => {
		const { container } = await renderWithRouter(
			<PendingReportsBadge organizationSlug="acme" count={0} />,
		)
		expect(container.querySelector('a')).toBeNull()
	})

	it('shows the count and links to the reports list', async () => {
		await renderWithRouter(
			<PendingReportsBadge organizationSlug="acme" count={3} />,
		)

		expect(screen.getByText(/3 écarts en attente/i)).toBeDefined()
		const link = screen.getByRole('link')
		expect(link.getAttribute('href')).toBe('/o/acme/planning/reports')
	})

	it('uses the singular for exactly one', async () => {
		await renderWithRouter(
			<PendingReportsBadge organizationSlug="acme" count={1} />,
		)

		expect(screen.getByText(/1 écart en attente/i)).toBeDefined()
	})
})

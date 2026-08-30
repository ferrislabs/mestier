import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import {
	ProfitabilityCardUI,
	type ProfitabilityCardUIProps,
} from '#/pages/home/ui/profitability-card-ui'
import { renderWithRouter } from '#/test/render-with-router'

function props(
	overrides: Partial<ProfitabilityCardUIProps> = {},
): ProfitabilityCardUIProps {
	return {
		periodLabel: 'août 2026',
		organizationSlug: 'atelier-bois',
		quotedCents: 100_000,
		marginCents: 53_000,
		costsRedacted: false,
		trendPercent: null,
		isLoading: false,
		error: null,
		...overrides,
	}
}

describe('ProfitabilityCardUI', () => {
	it('shows the margin and the quoted hint when costs are not redacted', async () => {
		await renderWithRouter(<ProfitabilityCardUI {...props()} />)

		expect(screen.getByText('530,00 €')).toBeDefined()
		expect(screen.getByText(/Devisé/)).toBeDefined()
	})

	/** #306: locked here too, not just on the full breakdown this card links
	 * to — a caller landing on the homepage must never see a margin figure it
	 * has no `VIEW_COST` for, even reduced to a single tile. */
	it('never renders the margin figure when costs are redacted', async () => {
		await renderWithRouter(
			<ProfitabilityCardUI {...props({ costsRedacted: true })} />,
		)

		expect(screen.queryByText('530,00 €')).toBeNull()
		expect(screen.queryByText(/Devisé/)).toBeNull()
		expect(screen.getByText(/Accès restreint/)).toBeDefined()
	})

	it('surfaces the error message instead of a figure when the read failed', async () => {
		await renderWithRouter(
			<ProfitabilityCardUI
				{...props({ error: 'Impossible de charger la rentabilité' })}
			/>,
		)

		expect(
			screen.getByText('Impossible de charger la rentabilité'),
		).toBeDefined()
		expect(screen.queryByText('530,00 €')).toBeNull()
	})
})

describe('ProfitabilityCardUI — trend', () => {
	it('shows no trend when it cannot be stated', async () => {
		await renderWithRouter(
			<ProfitabilityCardUI {...props({ trendPercent: null })} />,
		)

		expect(screen.queryByText(/vs mois dernier/)).toBeNull()
	})

	it('shows a positive trend with a plus sign', async () => {
		await renderWithRouter(
			<ProfitabilityCardUI {...props({ trendPercent: 8 })} />,
		)

		expect(screen.getByText(/\+8\s?%\s?vs mois dernier/)).toBeDefined()
	})

	it('shows a negative trend without a plus sign', async () => {
		await renderWithRouter(
			<ProfitabilityCardUI {...props({ trendPercent: -12 })} />,
		)

		expect(screen.getByText(/-12\s?%\s?vs mois dernier/)).toBeDefined()
		expect(screen.queryByText(/\+-12/)).toBeNull()
	})

	it('never shows a trend when costs are redacted, even if one is given', async () => {
		await renderWithRouter(
			<ProfitabilityCardUI
				{...props({ trendPercent: 8, costsRedacted: true })}
			/>,
		)

		expect(screen.queryByText(/vs mois dernier/)).toBeNull()
	})
})

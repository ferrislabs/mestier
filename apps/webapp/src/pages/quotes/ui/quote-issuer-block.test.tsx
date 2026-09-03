import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import type { Organization } from '#/hooks/use-organizations'
import { QuoteIssuerDetails, QuoteIssuerMark } from './quote-issuer-block'

function organization(overrides: Partial<Organization> = {}): Organization {
	return {
		id: 'org-1',
		name: 'Atelier Bois & Co',
		slug: 'atelier-bois',
		owner_id: 'user-1',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		field_clock_enabled: false,
		vat_on_debits: false,
		missing_legal_identity_fields: [],
		...overrides,
	}
}

describe('QuoteIssuerMark', () => {
	it('shows initials from the display name', () => {
		render(<QuoteIssuerMark organization={organization()} />)

		expect(screen.getByText('AB')).toBeDefined()
	})
})

describe('QuoteIssuerDetails', () => {
	it('prefers the legal name over the display name', () => {
		render(
			<QuoteIssuerDetails
				organization={organization({
					name: 'Atelier Bois & Co',
					legal_name: 'SARL Atelier Bois & Co',
				})}
			/>,
		)

		expect(screen.getByText('SARL Atelier Bois & Co')).toBeDefined()
	})

	it('falls back to the display name when no legal name is set', () => {
		render(<QuoteIssuerDetails organization={organization()} />)

		expect(screen.getByText('Atelier Bois & Co')).toBeDefined()
	})

	it('writes the address as it would be on an envelope', () => {
		render(
			<QuoteIssuerDetails
				organization={organization({
					address_line1: '12 rue des Artisans',
					address_postal_code: '69001',
					address_city: 'Lyon',
				})}
			/>,
		)

		expect(screen.getByText('12 rue des Artisans')).toBeDefined()
		expect(screen.getByText('69001 Lyon')).toBeDefined()
	})

	it('skips legal mentions and contact details that were never filled in', () => {
		render(<QuoteIssuerDetails organization={organization()} />)

		expect(screen.queryByText(/SIRET/)).toBeNull()
		expect(screen.queryByText(/Capital de/)).toBeNull()
	})

	it('shows the SIRET and share capital once they exist', () => {
		render(
			<QuoteIssuerDetails
				organization={organization({
					registration_number: '123 456 789 00012',
					share_capital_cents: 1_000_000,
				})}
			/>,
		)

		expect(screen.getByText(/SIRET 123 456 789 00012/)).toBeDefined()
		expect(screen.getByText(/Capital de 10 000 €/)).toBeDefined()
	})
})

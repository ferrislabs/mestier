import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { DocumentPaper, type PaperDocument } from './document-paper'

function paper(overrides: Partial<PaperDocument> = {}): PaperDocument {
	return {
		kind: 'quote',
		draft: false,
		heading: 'Devis',
		number: 'DEV-001',
		title: 'Salle de bain',
		issueDate: '2026-06-02',
		dueDate: null,
		issuer: {
			name: 'Atelier Bois & Co',
			legalForm: 'SARL',
			capital: '10 000,00 €',
			addressLines: ['5 rue des Forges', '69001 Lyon'],
			phone: '',
			email: '',
			legalId: '12345678901234',
			vatNumber: '',
			insurance: 'Assurance décennale n° 42',
		},
		recipient: {
			name: 'Menuiserie Dupont',
			contactName: '',
			addressLines: ['1 rue Neuve'],
			legalId: '',
		},
		lines: [
			{
				id: 'line-1',
				title: 'Pose de parquet',
				details: 'Chêne massif',
				quantity: '4',
				unit: 'HOUR',
				unitPriceCents: 4500,
				vatRateBp: 2000,
				totalCents: 18000,
			},
		],
		totals: {
			netCents: 18000,
			vat: [{ rateBp: 2000, vatCents: 3600 }],
			grossCents: 21600,
		},
		showVat: true,
		vatNotice: null,
		notes: '',
		reference: null,
		reason: '',
		...overrides,
	}
}

describe('DocumentPaper', () => {
	it('prints the issuer, the recipient, the lines and the backend totals', () => {
		render(<DocumentPaper document={paper()} />)

		expect(screen.getByText('Atelier Bois & Co')).toBeDefined()
		expect(screen.getByText('SIRET 123 456 789 01234')).toBeDefined()
		expect(screen.getByText('Menuiserie Dupont')).toBeDefined()
		expect(screen.getByText('Pose de parquet')).toBeDefined()
		expect(screen.getByText('Chêne massif')).toBeDefined()
		expect(screen.getByText('216,00 €')).toBeDefined()
		expect(screen.getByText('TVA 20 %', { selector: 'dt' })).toBeDefined()
	})

	it('invites to sign a quote', () => {
		render(<DocumentPaper document={paper()} />)

		expect(screen.getByText(/Bon pour accord/)).toBeDefined()
	})

	it('has no signature box on an invoice', () => {
		render(
			<DocumentPaper
				document={paper({
					kind: 'invoice',
					heading: 'Facture',
					dueDate: '2026-07-01',
				})}
			/>,
		)

		expect(screen.queryByText(/Bon pour accord/)).toBeNull()
		expect(screen.getByText('Échéance')).toBeDefined()
	})

	it('watermarks a draft and says the number comes at issue', () => {
		render(<DocumentPaper document={paper({ draft: true, number: null })} />)

		expect(screen.getByText('Brouillon')).toBeDefined()
		expect(screen.getByText('Numéro attribué à l’émission')).toBeDefined()
	})

	it('prints the reason of a credit note and credits the total', () => {
		render(
			<DocumentPaper
				document={paper({
					kind: 'creditNote',
					heading: 'Avoir',
					reason: 'Taux de TVA erroné',
					reference: 'Avoir sur la facture FAC-001',
				})}
			/>,
		)

		expect(screen.getByText('Taux de TVA erroné')).toBeDefined()
		expect(screen.getByText('Total TTC crédité')).toBeDefined()
		expect(
			screen.getByText('Référence : Avoir sur la facture FAC-001'),
		).toBeDefined()
		expect(screen.queryByText(/Assurance décennale/)).toBeNull()
	})

	it('says so when there is no line', () => {
		render(<DocumentPaper document={paper({ lines: [] })} />)

		expect(screen.getByText('Aucune ligne')).toBeDefined()
	})

	it('hides the VAT column when the organization is not subject to VAT', () => {
		render(
			<DocumentPaper
				document={paper({ showVat: false, vatNotice: 'TVA non applicable' })}
			/>,
		)

		expect(screen.queryByRole('columnheader', { name: 'TVA' })).toBeNull()
		expect(screen.getByText('TVA non applicable')).toBeDefined()
	})
})

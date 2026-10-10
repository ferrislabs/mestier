import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { buildProductRequest } from '#/pages/catalog/lib/product-request'
import {
	ProductTechnicalSheet,
	type ProductTechnicalSheetProps,
} from '#/pages/catalog/ui/product-technical-sheet'

function props(
	overrides: Partial<ProductTechnicalSheetProps> = {},
): ProductTechnicalSheetProps {
	return {
		values: {
			supplierId: 's1',
			manufacturer: 'Pépinière Durand',
			technicalSpecs: 'Hauteur 2 m',
			datasheetKeys: ['docs/fiche.pdf'],
		},
		suppliers: [
			{ id: 's1', name: 'Martin' },
			{ id: 's2', name: 'Durand' },
		],
		datasheets: [
			{
				key: 'docs/fiche.pdf',
				name: 'fiche.pdf',
				type: 'PDF',
				url: 'https://files/fiche.pdf',
			},
		],
		canCreateSupplier: true,
		onChange: vi.fn(),
		onUpload: vi.fn(),
		onRemove: vi.fn(),
		onCreateSupplier: vi.fn(),
		...overrides,
	}
}

describe('ProductTechnicalSheet', () => {
	it('renders the values and the attached documents', () => {
		render(<ProductTechnicalSheet {...props()} />)

		expect(
			(screen.getByLabelText('Fournisseur') as HTMLInputElement).value,
		).toBe('s1')
		expect(
			(screen.getByLabelText('Fabricant / pépinière') as HTMLInputElement)
				.value,
		).toBe('Pépinière Durand')
		expect(
			(screen.getByLabelText('Caractéristiques techniques') as HTMLInputElement)
				.value,
		).toBe('Hauteur 2 m')
		expect(screen.getByText('fiche.pdf')).toBeTruthy()
		expect(
			screen
				.getByRole('link', { name: 'Ouvrir fiche.pdf' })
				.getAttribute('href'),
		).toBe('https://files/fiche.pdf')
	})

	it('reports edits, removal and supplier creation', async () => {
		const user = userEvent.setup()
		const p = props()
		render(<ProductTechnicalSheet {...p} />)

		await user.selectOptions(screen.getByLabelText('Fournisseur'), 's2')
		expect(p.onChange).toHaveBeenCalledWith({ supplierId: 's2' })

		await user.type(screen.getByLabelText('Fabricant / pépinière'), 'X')
		expect(p.onChange).toHaveBeenCalledWith({
			manufacturer: 'Pépinière DurandX',
		})

		await user.click(screen.getByRole('button', { name: 'Retirer fiche.pdf' }))
		expect(p.onRemove).toHaveBeenCalledWith('docs/fiche.pdf')

		await user.click(
			screen.getByRole('button', { name: 'Nouveau fournisseur' }),
		)
		expect(p.onCreateSupplier).toHaveBeenCalled()
	})

	it('hides the supplier creation action without permission', () => {
		render(<ProductTechnicalSheet {...props({ canCreateSupplier: false })} />)
		expect(
			screen.queryByRole('button', { name: 'Nouveau fournisseur' }),
		).toBeNull()
	})
})

describe('buildProductRequest', () => {
	const base = {
		name: ' Thuya ',
		sku: '',
		unit: 'UNIT',
		unitPrice: '12,5',
		description: '',
		photoKeys: ['p1'],
		supplierId: '',
		manufacturer: '',
		technicalSpecs: '',
		datasheetKeys: [],
	}

	it('sends the four technical sheet fields', () => {
		expect(
			buildProductRequest({
				...base,
				supplierId: 's1',
				manufacturer: ' Durand ',
				technicalSpecs: 'a\nb',
				datasheetKeys: ['k1'],
			}),
		).toMatchObject({
			name: 'Thuya',
			supplier_id: 's1',
			manufacturer: 'Durand',
			technical_specs: 'a\nb',
			datasheet_keys: ['k1'],
			photo_keys: ['p1'],
		})
	})

	it('sends null for blank fields', () => {
		expect(buildProductRequest(base)).toMatchObject({
			supplier_id: null,
			manufacturer: null,
			technical_specs: null,
			datasheet_keys: [],
		})
	})
})

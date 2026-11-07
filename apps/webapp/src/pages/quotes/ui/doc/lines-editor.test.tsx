import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { CatalogItem } from '#/hooks/use-catalog-items'
import {
	type QuoteLineFormValues,
	quoteLineTotalCents,
} from '#/pages/quotes/types'
import { LinesEditor, lineProblems } from './lines-editor'

class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
globalThis.ResizeObserver ??= ResizeObserverStub
Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false
Element.prototype.releasePointerCapture ??= () => {}

function line(
	overrides: Partial<QuoteLineFormValues> = {},
): QuoteLineFormValues {
	return {
		clientId: 'line-1',
		catalogItemId: '',
		catalogItemType: 'CUSTOM',
		serviceRateId: '',
		label: 'Taille de haie',
		quantity: '4',
		unit: 'HOUR',
		unitPrice: '45,00',
		vatRateBp: '',
		notes: '',
		photoKeys: [],
		discountPercent: '',
		frequency: 'one_shot',
		...overrides,
	}
}

const CATALOG_ITEM: CatalogItem = {
	id: 'catalog-1',
	type: 'SERVICE',
	sourceId: 'service-1',
	label: 'Pose de parquet',
	unit: 'HOUR',
	unitPriceCents: 4500,
	description: '',
	defaultVatRateBp: null,
}

function baseProps() {
	return {
		lines: [line()],
		idPrefix: 'quote-1',
		vatEnabled: false,
		totalOf: quoteLineTotalCents,
		onPatch: vi.fn(),
		onMove: vi.fn(),
		onDuplicate: vi.fn(() => 'line-copy'),
		onRemove: vi.fn(),
		onAdd: vi.fn(() => 'line-new'),
		catalog: { items: [CATALOG_ITEM], onPick: vi.fn(() => 'line-new') },
		details: { photoUrls: {}, onUpload: vi.fn() },
	}
}

describe('LinesEditor', () => {
	it('reads as a table: designation, quantity, unit price and total', () => {
		render(<LinesEditor {...baseProps()} />)

		expect(screen.getByText('Désignation')).toBeDefined()
		expect(screen.getByText('Quantité')).toBeDefined()
		expect(screen.getByText('PU HT')).toBeDefined()
		expect(screen.getByText('Total HT')).toBeDefined()
		expect(
			(
				screen.getByRole('textbox', {
					name: 'Désignation, ligne 1',
				}) as HTMLTextAreaElement
			).value,
		).toBe('Taille de haie')
		expect(screen.getByText('180,00 €')).toBeDefined()
	})

	it('adds a VAT column only when the organization charges VAT', () => {
		const { rerender } = render(<LinesEditor {...baseProps()} />)
		expect(screen.queryByText('TVA')).toBeNull()

		rerender(<LinesEditor {...baseProps()} vatEnabled />)
		expect(screen.getByText('TVA')).toBeDefined()
	})

	it('reports what is typed in a cell, replacing the selected value', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<LinesEditor {...props} />)

		await user.type(
			screen.getByRole('textbox', { name: 'Prix unitaire HT, ligne 1' }),
			'1',
		)

		expect(props.onPatch).toHaveBeenCalledWith(0, { unitPrice: '1' })
	})

	it('asks to add a line', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<LinesEditor {...props} />)

		await user.click(screen.getByRole('button', { name: /ajouter une ligne/i }))

		expect(props.onAdd).toHaveBeenCalledTimes(1)
	})

	it('adds a line from the catalogue', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<LinesEditor {...props} />)

		await user.click(
			screen.getByRole('button', { name: /ajouter depuis le catalogue/i }),
		)
		await user.click(screen.getByRole('option', { name: /Pose de parquet/ }))

		expect(props.catalog.onPick).toHaveBeenCalledWith(CATALOG_ITEM)
	})

	it('removes the line whose menu was used, by its index', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(
			<LinesEditor
				{...props}
				lines={[line({ clientId: 'line-1' }), line({ clientId: 'line-2' })]}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Actions de la ligne 2' }),
		)
		await user.click(
			screen.getByRole('menuitem', { name: /supprimer la ligne/i }),
		)

		expect(props.onRemove).toHaveBeenCalledWith(1)
	})

	it('moves and duplicates a line from its menu', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(
			<LinesEditor
				{...props}
				lines={[line({ clientId: 'line-1' }), line({ clientId: 'line-2' })]}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Actions de la ligne 2' }),
		)
		await user.click(screen.getByRole('menuitem', { name: /monter la ligne/i }))
		expect(props.onMove).toHaveBeenCalledWith(1, -1)

		await user.click(
			screen.getByRole('button', { name: 'Actions de la ligne 1' }),
		)
		await user.click(
			screen.getByRole('menuitem', { name: /dupliquer la ligne/i }),
		)
		expect(props.onDuplicate).toHaveBeenCalledWith(0)
	})

	it('unfolds the note and photos of a line', async () => {
		const user = userEvent.setup()
		render(
			<LinesEditor
				{...baseProps()}
				lines={[line({ notes: 'Détail interne' })]}
			/>,
		)

		expect(screen.queryByRole('textbox', { name: 'Note' })).toBeNull()

		await user.click(screen.getByRole('button', { name: /note et photos/i }))

		expect(screen.getByRole('textbox', { name: 'Note' })).toHaveProperty(
			'value',
			'Détail interne',
		)
	})

	it('says so when there is no line', () => {
		render(<LinesEditor {...baseProps()} lines={[]} />)

		expect(screen.getByText('Aucune ligne pour le moment.')).toBeDefined()
	})
})

describe('lineProblems', () => {
	it('leaves a blank new line alone', () => {
		expect(
			lineProblems(line({ label: '', unitPrice: '', quantity: '1' })),
		).toEqual({ label: false, quantity: false, unitPrice: false })
	})

	it('flags a priced line without a designation', () => {
		expect(lineProblems(line({ label: '' })).label).toBe(true)
	})

	it('flags a named line without a price', () => {
		expect(lineProblems(line({ unitPrice: '' })).unitPrice).toBe(true)
	})

	it('flags a quantity that is not positive', () => {
		expect(lineProblems(line({ quantity: '0' })).quantity).toBe(true)
		expect(lineProblems(line({ quantity: 'abc' })).quantity).toBe(true)
	})

	it('has no discount or frequency column on a document without them', () => {
		const { lines: _lines, totalOf: _totalOf, ...rest } = baseProps()
		render(
			<LinesEditor
				{...rest}
				lines={[
					{
						clientId: 'line-1',
						label: 'Taille de haie',
						quantity: '1',
						unitPrice: '10',
						vatRateBp: '',
					},
				]}
				totalOf={() => 1000}
			/>,
		)

		expect(screen.queryByText('Remise %')).toBeNull()
		expect(screen.queryByText('Facturation')).toBeNull()
	})

	it('reports a typed discount', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<LinesEditor {...props} />)

		expect(screen.getByText('Remise %')).toBeDefined()
		await user.type(
			screen.getByRole('textbox', { name: 'Remise en pourcentage, ligne 1' }),
			'5',
		)

		expect(props.onPatch).toHaveBeenCalledWith(0, { discountPercent: '5' })
	})

	it('offers the four billing frequencies', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<LinesEditor {...props} />)

		await user.click(
			screen.getByRole('combobox', {
				name: 'Fréquence de facturation, ligne 1',
			}),
		)

		expect(
			screen.getAllByRole('option').map((option) => option.textContent),
		).toEqual(['Ponctuel', 'Mensuel', 'Trimestriel', 'Annuel'])
		await user.click(screen.getByRole('option', { name: 'Mensuel' }))

		expect(props.onPatch).toHaveBeenCalledWith(0, { frequency: 'monthly' })
	})
})

import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { CatalogItem } from '#/hooks/use-catalog-items'
import type { QuoteLineFormValues } from '../types'
import { QuoteLineEditor } from './quote-line-editor'

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
		label: '',
		quantity: '1',
		unit: 'HOUR',
		unitPrice: '',
		vatRateBp: '',
		notes: '',
		photoKeys: [],
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
		index: 0,
		line: line(),
		catalogItems: [CATALOG_ITEM],
		photos: [],
		isOpen: true,
		canRemove: true,
		vatEnabled: false,
		gridTemplateColumns: '1fr 100px 110px 110px 40px',
		onOpenChange: vi.fn(),
		onChange: vi.fn(),
		onSelectCatalogItem: vi.fn(),
		onRemove: vi.fn(),
		onUploadPhoto: vi.fn(),
		onRemovePhoto: vi.fn(),
	}
}

describe('QuoteLineEditor — folded row preview', () => {
	it('prints the description under the label instead of repeating quantity and price', () => {
		render(
			<QuoteLineEditor
				{...baseProps()}
				isOpen={false}
				line={line({
					label: 'Maquette sur-mesure',
					notes: 'Wireframe, webdesign interactif, déclinaison des maquettes',
				})}
			/>,
		)

		expect(
			screen.getByText(
				'Wireframe, webdesign interactif, déclinaison des maquettes',
			),
		).toBeDefined()
	})

	it('falls back to the line source when there is no description yet', () => {
		render(
			<QuoteLineEditor
				{...baseProps()}
				isOpen={false}
				line={line({ label: 'Maquette sur-mesure', notes: '' })}
			/>,
		)

		expect(screen.getByText('Ligne libre')).toBeDefined()
	})

	it('shows the line source, not the description, while it is open for edit', () => {
		render(
			<QuoteLineEditor
				{...baseProps()}
				isOpen={true}
				line={line({
					label: 'Maquette sur-mesure',
					notes: 'Détail interne',
					catalogItemId: 'catalog-1',
					catalogItemType: 'SERVICE',
				})}
			/>,
		)

		// The description does show while open — inside its own notes field,
		// not doubled up as the row's sub-label.
		expect(screen.getByText('Service catalogue')).toBeDefined()
		expect(screen.getByRole('textbox', { name: 'Note' })).toHaveProperty(
			'value',
			'Détail interne',
		)
	})
})

describe('QuoteLineEditor — catalogue pick on a handwritten line', () => {
	it('applies a catalogue pick immediately on a blank line — nothing to lose', async () => {
		const user = userEvent.setup()
		const onSelectCatalogItem = vi.fn()
		render(
			<QuoteLineEditor
				{...baseProps()}
				line={line({ label: '' })}
				onSelectCatalogItem={onSelectCatalogItem}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Catalogue' }))
		await user.click(screen.getByRole('option', { name: 'Pose de parquet' }))

		expect(onSelectCatalogItem).toHaveBeenCalledWith('catalog-1')
		expect(screen.queryByText('Remplacer cette ligne libre ?')).toBeNull()
	})

	it('asks before overwriting a line that was already typed out by hand', async () => {
		const user = userEvent.setup()
		const onSelectCatalogItem = vi.fn()
		render(
			<QuoteLineEditor
				{...baseProps()}
				line={line({ label: 'Réparation de volet' })}
				onSelectCatalogItem={onSelectCatalogItem}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Catalogue' }))
		await user.click(screen.getByRole('option', { name: 'Pose de parquet' }))

		expect(onSelectCatalogItem).not.toHaveBeenCalled()
		expect(screen.getByText('Remplacer cette ligne libre ?')).toBeDefined()
	})

	it('applies the pick once the confirmation is accepted', async () => {
		const user = userEvent.setup()
		const onSelectCatalogItem = vi.fn()
		render(
			<QuoteLineEditor
				{...baseProps()}
				line={line({ label: 'Réparation de volet' })}
				onSelectCatalogItem={onSelectCatalogItem}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Catalogue' }))
		await user.click(screen.getByRole('option', { name: 'Pose de parquet' }))
		await user.click(screen.getByRole('button', { name: 'Remplacer' }))

		expect(onSelectCatalogItem).toHaveBeenCalledWith('catalog-1')
	})

	it('leaves the line untouched when the confirmation is declined', async () => {
		const user = userEvent.setup()
		const onSelectCatalogItem = vi.fn()
		render(
			<QuoteLineEditor
				{...baseProps()}
				line={line({ label: 'Réparation de volet' })}
				onSelectCatalogItem={onSelectCatalogItem}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Catalogue' }))
		await user.click(screen.getByRole('option', { name: 'Pose de parquet' }))
		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(onSelectCatalogItem).not.toHaveBeenCalled()
	})

	it('never asks when switching back to a free line', async () => {
		const user = userEvent.setup()
		const onSelectCatalogItem = vi.fn()
		render(
			<QuoteLineEditor
				{...baseProps()}
				line={line({
					catalogItemId: 'catalog-1',
					catalogItemType: 'SERVICE',
					label: 'Pose de parquet',
				})}
				onSelectCatalogItem={onSelectCatalogItem}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Catalogue' }))
		await user.click(screen.getByRole('option', { name: 'Ligne libre' }))

		expect(onSelectCatalogItem).toHaveBeenCalledWith('')
	})
})

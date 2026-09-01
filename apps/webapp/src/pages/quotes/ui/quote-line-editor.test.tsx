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

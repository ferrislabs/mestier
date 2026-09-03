import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { QuoteLineFormValues } from '../types'
import { QuoteLinesTable } from './quote-lines-table'

class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
// biome-ignore lint/suspicious/noExplicitAny: test-only global polyfill
;(globalThis as any).ResizeObserver ??= ResizeObserverStub
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
		...overrides,
	}
}

function baseProps() {
	return {
		lines: [line()],
		catalogItems: [],
		photosByLine: {},
		openLineId: null,
		vatEnabled: false,
		onOpenLineChange: vi.fn(),
		onLineChange: vi.fn(),
		onSelectCatalogItem: vi.fn(),
		onRemoveLine: vi.fn(),
		onAddLine: vi.fn(),
		onUploadLinePhoto: vi.fn(),
		onRemoveLinePhoto: vi.fn(),
	}
}

describe('QuoteLinesTable', () => {
	it('reads as a table: description, quantity, price and total, one column per header', () => {
		render(<QuoteLinesTable {...baseProps()} />)

		expect(screen.getByText('Détails')).toBeDefined()
		expect(screen.getByText('Quantité')).toBeDefined()
		expect(screen.getByText('Prix unitaire')).toBeDefined()
		expect(screen.getByText('Montant HT')).toBeDefined()
		expect(screen.getByText('Taille de haie')).toBeDefined()
		expect(screen.getByText('180,00 €')).toBeDefined()
	})

	it('adds a VAT column only when the organization charges VAT', () => {
		const { rerender } = render(<QuoteLinesTable {...baseProps()} />)
		expect(screen.queryByText('TVA')).toBeNull()

		rerender(<QuoteLinesTable {...baseProps()} vatEnabled />)
		expect(screen.getByText('TVA')).toBeDefined()
	})

	it('asks to add a line', async () => {
		const user = userEvent.setup()
		const onAddLine = vi.fn()
		render(<QuoteLinesTable {...baseProps()} onAddLine={onAddLine} />)

		await user.click(screen.getByRole('button', { name: /ajouter une ligne/i }))

		expect(onAddLine).toHaveBeenCalledTimes(1)
	})

	it('asks to remove the line that was clicked, by its id', async () => {
		const user = userEvent.setup()
		const onRemoveLine = vi.fn()
		render(
			<QuoteLinesTable
				{...baseProps()}
				lines={[line({ clientId: 'line-1' }), line({ clientId: 'line-2' })]}
				onRemoveLine={onRemoveLine}
			/>,
		)

		const removeButtons = screen.getAllByRole('button', {
			name: /supprimer la ligne/i,
		})
		await user.click(removeButtons[1])

		expect(onRemoveLine).toHaveBeenCalledWith('line-2')
	})

	it('keeps the last line from being removed', () => {
		render(<QuoteLinesTable {...baseProps()} lines={[line()]} />)

		expect(
			screen.getByRole('button', { name: /supprimer la ligne/i }),
		).toHaveProperty('disabled', true)
	})
})

import { describe, expect, it } from 'vitest'
import {
	supplierInvoiceSourceLabel,
	supplierInvoiceStatusLabel,
} from '#/pages/purchase/types'

describe('supplierInvoiceStatusLabel', () => {
	it('translates every known status', () => {
		expect(supplierInvoiceStatusLabel('RECEIVED')).toBe('Reçue')
		expect(supplierInvoiceStatusLabel('CONFIRMED')).toBe('Confirmée')
		expect(supplierInvoiceStatusLabel('REJECTED')).toBe('Rejetée')
	})
})

describe('supplierInvoiceSourceLabel', () => {
	it('names a Factur-X document distinctly from a manual entry', () => {
		expect(supplierInvoiceSourceLabel('FACTUR_X')).toBe('Factur-X')
		expect(supplierInvoiceSourceLabel('MANUAL')).toBe('Saisie manuelle')
	})
})

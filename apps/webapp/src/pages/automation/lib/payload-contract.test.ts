import { describe, expect, it } from 'vitest'
import {
	payloadContractSource,
	payloadTypeName,
} from '#/pages/automation/lib/payload-contract'

describe('payloadTypeName', () => {
	it('pascal-cases the event name', () => {
		expect(payloadTypeName('quote.sent')).toBe('QuoteSentPayload')
	})
})

describe('payloadContractSource', () => {
	it('infers an interface from the example', () => {
		expect(
			payloadContractSource('quote.sent', {
				id: 'a',
				total: 12,
				paid: false,
				note: null,
				lignes: [{ prix: 1 }],
				'a b': {},
			}),
		).toBe(
			[
				'export interface QuoteSentPayload {',
				'  id: string',
				'  total: number',
				'  paid: boolean',
				'  note: null',
				'  lignes: {',
				'    prix: number',
				'  }[]',
				'  "a b": {}',
				'}',
			].join('\n'),
		)
	})

	it('falls back to an empty interface for a non-object example', () => {
		expect(payloadContractSource('x.y', null)).toBe(
			'export interface XYPayload {}',
		)
	})
})

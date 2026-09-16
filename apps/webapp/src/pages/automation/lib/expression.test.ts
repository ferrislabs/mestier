import { describe, expect, it } from 'vitest'
import {
	expressionToken,
	insertExpressionAtCursor,
} from '#/pages/automation/lib/expression'

describe('expressionToken', () => {
	it('wraps a path in the double-brace expression syntax', () => {
		expect(expressionToken('connectors.c3.output.id')).toBe(
			'{{ connectors.c3.output.id }}',
		)
	})
})

describe('insertExpressionAtCursor', () => {
	it('inserts at the cursor rather than replacing the field', () => {
		const result = insertExpressionAtCursor(
			'Hello ',
			'trigger.customer.name',
			{ start: 6, end: 6 },
		)

		expect(result.text).toBe('Hello {{ trigger.customer.name }}')
	})

	it('places the cursor right after the inserted token', () => {
		const result = insertExpressionAtCursor(
			'Hello ',
			'trigger.customer.name',
			{ start: 6, end: 6 },
		)

		expect(result.cursor).toBe('Hello {{ trigger.customer.name }}'.length)
	})

	it('replaces a selection rather than inserting inside it', () => {
		const result = insertExpressionAtCursor('Hello world', 'trigger.name', {
			start: 6,
			end: 11,
		})

		expect(result.text).toBe('Hello {{ trigger.name }}')
	})

	it('inserts in the middle of existing text', () => {
		const result = insertExpressionAtCursor(
			'Dear , welcome',
			'trigger.customer.name',
			{ start: 5, end: 5 },
		)

		expect(result.text).toBe(
			'Dear {{ trigger.customer.name }}, welcome',
		)
	})

	it('appends when the value is empty', () => {
		const result = insertExpressionAtCursor('', 'connectors.c1.output.id', {
			start: 0,
			end: 0,
		})

		expect(result.text).toBe('{{ connectors.c1.output.id }}')
		expect(result.cursor).toBe(result.text.length)
	})

	it('clamps a selection past the end of the value', () => {
		const result = insertExpressionAtCursor('ab', 'trigger.x', {
			start: 50,
			end: 60,
		})

		expect(result.text).toBe('ab{{ trigger.x }}')
	})
})

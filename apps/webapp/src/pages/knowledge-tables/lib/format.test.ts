import { describe, expect, it } from 'vitest'
import {
	formatCents,
	formatIsoDate,
	formatNumber,
	numberInputText,
	parseDateInput,
	parseEurosInput,
	parseNumberInput,
} from '#/pages/knowledge-tables/lib/format'

const spaces = (text: string) => text.replace(/\s/g, ' ')

describe('parseNumberInput', () => {
	it('reads an empty input as empty', () => {
		expect(parseNumberInput('')).toEqual({ kind: 'empty' })
		expect(parseNumberInput('   ')).toEqual({ kind: 'empty' })
	})

	it('accepts a French comma and spaces between thousands', () => {
		expect(parseNumberInput('12,5')).toEqual({ kind: 'value', value: 12.5 })
		expect(parseNumberInput('1 250,75')).toEqual({
			kind: 'value',
			value: 1250.75,
		})
		expect(parseNumberInput('1 250')).toEqual({
			kind: 'value',
			value: 1250,
		})
		expect(parseNumberInput('1.250,5')).toEqual({
			kind: 'value',
			value: 1250.5,
		})
	})

	it('accepts a dot and a sign', () => {
		expect(parseNumberInput('3.25')).toEqual({ kind: 'value', value: 3.25 })
		expect(parseNumberInput('-4')).toEqual({ kind: 'value', value: -4 })
	})

	it('refuses what is not a number', () => {
		expect(parseNumberInput('abc')).toEqual({ kind: 'invalid' })
		expect(parseNumberInput('12,3,4')).toEqual({ kind: 'invalid' })
		expect(parseNumberInput('1-2')).toEqual({ kind: 'invalid' })
	})
})

describe('parseEurosInput', () => {
	it('turns euros into cents', () => {
		expect(parseEurosInput('1 250,50 €')).toEqual({
			kind: 'value',
			value: 125050,
		})
		expect(parseEurosInput('12')).toEqual({ kind: 'value', value: 1200 })
		expect(parseEurosInput('0,1')).toEqual({ kind: 'value', value: 10 })
		expect(parseEurosInput('19.99')).toEqual({ kind: 'value', value: 1999 })
	})

	it('refuses more than two decimals', () => {
		expect(parseEurosInput('1,234')).toEqual({ kind: 'invalid' })
	})

	it('reads an empty input as empty', () => {
		expect(parseEurosInput(' € ')).toEqual({ kind: 'empty' })
	})
})

describe('formatNumber', () => {
	it('shows the decimals of the column', () => {
		expect(spaces(formatNumber(1250.5, 2))).toBe('1 250,50')
		expect(spaces(formatNumber(3, 0))).toBe('3')
		expect(spaces(formatNumber(2.34567, 4))).toBe('2,3457')
	})

	it('shows up to four decimals when the column says nothing', () => {
		expect(formatNumber(1.5, null)).toBe('1,5')
		expect(formatNumber(1.123456, undefined)).toBe('1,1235')
	})
})

describe('formatCents', () => {
	it('shows euros with a comma and a space before the sign', () => {
		expect(spaces(formatCents(125050))).toBe('1 250,50 €')
		expect(spaces(formatCents(-5))).toBe('-0,05 €')
	})
})

describe('numberInputText', () => {
	it('writes a number for editing with a comma and no grouping', () => {
		expect(numberInputText(1250.5)).toBe('1250,5')
		expect(numberInputText(-3)).toBe('-3')
	})
})

describe('formatIsoDate', () => {
	it('writes JJ/MM/AAAA', () => {
		expect(formatIsoDate('2026-11-05')).toBe('05/11/2026')
	})

	it('keeps an unreadable value as is', () => {
		expect(formatIsoDate('demain')).toBe('demain')
	})
})

describe('parseDateInput', () => {
	it('reads JJ/MM/AAAA and AAAA-MM-JJ', () => {
		expect(parseDateInput('05/11/2026')).toEqual({
			kind: 'value',
			value: '2026-11-05',
		})
		expect(parseDateInput('2026-11-05')).toEqual({
			kind: 'value',
			value: '2026-11-05',
		})
	})

	it('refuses a day that does not exist', () => {
		expect(parseDateInput('31/02/2026')).toEqual({ kind: 'invalid' })
		expect(parseDateInput('2026-13-01')).toEqual({ kind: 'invalid' })
		expect(parseDateInput('hier')).toEqual({ kind: 'invalid' })
	})

	it('reads an empty input as empty', () => {
		expect(parseDateInput('')).toEqual({ kind: 'empty' })
	})
})

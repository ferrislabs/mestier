import { describe, expect, it } from 'vitest'
import {
	dafNumberLabel,
	formatDate,
	STATUS_OPTIONS,
	statusClassName,
	statusLabel,
} from '#/pages/daf/lib/daf-status'

describe('daf status', () => {
	it('maps each status to its French label', () => {
		expect(statusLabel('pending')).toBe('En attente')
		expect(statusLabel('accepted')).toBe('Acceptée')
		expect(statusLabel('rejected_with_comments')).toBe('Refus avec obs.')
		expect(statusLabel('rejected')).toBe('Refus ferme')
		expect(STATUS_OPTIONS.map((option) => option.value)).toEqual([
			'pending',
			'accepted',
			'rejected_with_comments',
			'rejected',
		])
	})

	it('gives each status a distinct colour', () => {
		expect(statusClassName('pending')).toContain('amber')
		expect(statusClassName('accepted')).toContain('success')
		expect(statusClassName('rejected_with_comments')).toContain('orange')
		expect(statusClassName('rejected')).toContain('destructive')
	})

	it('zero-pads the number', () => {
		expect(dafNumberLabel(1)).toBe('D.A.F n°01')
		expect(dafNumberLabel(12)).toBe('D.A.F n°12')
		expect(dafNumberLabel(123)).toBe('D.A.F n°123')
	})

	it('formats dates day first and dashes missing ones', () => {
		expect(formatDate('2026-11-18')).toBe('18/11/2026')
		expect(formatDate(null)).toBe('–')
		expect(formatDate(undefined)).toBe('–')
	})
})

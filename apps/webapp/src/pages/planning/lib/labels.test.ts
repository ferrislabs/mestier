import { describe, expect, it } from 'vitest'
import {
	LABEL_COLOR_PALETTE,
	matchLabelByName,
	nextLabelColor,
	toggleLabelSelection,
} from '#/pages/planning/lib/labels'

const LABELS = [
	{ id: 'l1', name: 'Réunion', color: '#2563EB' },
	{ id: 'l2', name: 'Déplacement', color: '#16A34A' },
	{ id: 'l3', name: 'Formation', color: '#F59E0B' },
]

describe('matchLabelByName', () => {
	it('finds a label by exact name', () => {
		expect(matchLabelByName(LABELS, 'Réunion')).toEqual(LABELS[0])
	})

	it('matches case-insensitively and trims whitespace', () => {
		expect(matchLabelByName(LABELS, '  réunion  ')).toEqual(LABELS[0])
	})

	it('returns null when nothing matches — the caller creates a new label', () => {
		expect(matchLabelByName(LABELS, 'Urgent')).toBeNull()
	})

	it('returns null for a blank name', () => {
		expect(matchLabelByName(LABELS, '   ')).toBeNull()
	})
})

describe('toggleLabelSelection', () => {
	it('adds a label id not yet selected', () => {
		expect(toggleLabelSelection(['l1'], 'l2')).toEqual(['l1', 'l2'])
	})

	it('removes a label id already selected', () => {
		expect(toggleLabelSelection(['l1', 'l2'], 'l1')).toEqual(['l2'])
	})

	it('does not mutate the input array', () => {
		const selected = ['l1']
		toggleLabelSelection(selected, 'l2')
		expect(selected).toEqual(['l1'])
	})
})

describe('nextLabelColor', () => {
	it('picks the first palette color when no label exists yet', () => {
		expect(nextLabelColor([])).toBe(LABEL_COLOR_PALETTE[0])
	})

	it('cycles to a color not already used by an existing label, when possible', () => {
		const used = LABEL_COLOR_PALETTE.slice(0, 2).map((color) => ({ color }))
		expect(nextLabelColor(used)).toBe(LABEL_COLOR_PALETTE[2])
	})

	it('wraps around and reuses a color once every palette color is taken', () => {
		const used = LABEL_COLOR_PALETTE.map((color) => ({ color }))
		expect(LABEL_COLOR_PALETTE).toContain(nextLabelColor(used))
	})
})

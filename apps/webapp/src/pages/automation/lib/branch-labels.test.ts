import { describe, expect, it } from 'vitest'
import { branchLabel } from '#/pages/automation/lib/branch-labels'

describe('branchLabel', () => {
	it('traduit les branches du backend', () => {
		expect(branchLabel('Then')).toBe('vrai')
		expect(branchLabel('Else')).toBe('faux')
		expect(branchLabel('Each')).toBe('chaque élément')
		expect(branchLabel('After')).toBe('terminé')
	})

	it('laisse passer une branche inconnue', () => {
		expect(branchLabel('Retry')).toBe('Retry')
	})
})

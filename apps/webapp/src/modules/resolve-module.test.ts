import { describe, expect, it } from 'vitest'
import { resolveModule } from '#/modules/resolve-module'

describe('resolveModule', () => {
	it('retourne le module home pour la racine', () => {
		expect(resolveModule('/').id).toBe('home')
	})

	it('retourne le module dont le basePath correspond', () => {
		expect(resolveModule('/crm').id).toBe('crm')
		expect(resolveModule('/crm/customers').id).toBe('crm')
		expect(resolveModule('/crm/customers/abc-123').id).toBe('crm')
	})

	it('ne matche que sur une frontière de segment', () => {
		expect(resolveModule('/crmsomething').id).toBe('home')
	})

	it('replie sur home pour une URL globale ou inconnue', () => {
		expect(resolveModule('/settings').id).toBe('home')
		expect(resolveModule('/inconnu').id).toBe('home')
	})

	it('résout les pages clients vers le module crm', () => {
		expect(resolveModule('/crm/customers').id).toBe('crm')
		expect(resolveModule('/crm/customers/pipeline').id).toBe('crm')
	})

	it('résout un module désactivé comme les autres', () => {
		expect(resolveModule('/discussions').id).toBe('discussions')
	})
})

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

	it('rattache les paramètres à leur propre module', () => {
		expect(resolveModule('/settings').id).toBe('settings')
	})

	it('replie sur home pour une URL inconnue', () => {
		expect(resolveModule('/inconnu').id).toBe('home')
	})

	it('résout les pages clients vers le module crm', () => {
		expect(resolveModule('/crm/customers').id).toBe('crm')
		expect(resolveModule('/crm/customers/pipeline').id).toBe('crm')
	})

	it('résout un module annoncé comme les autres', () => {
		expect(resolveModule('/chat').id).toBe('chat')
	})

	it('résout les pages RH vers le module hr', () => {
		expect(resolveModule('/hr/employees').id).toBe('hr')
	})

	it('résout les pages de planning vers le module planning', () => {
		expect(resolveModule('/planning').id).toBe('planning')
		expect(resolveModule('/planning/team').id).toBe('planning')
		expect(resolveModule('/planning/tasks').id).toBe('planning')
	})
})

import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	connectorLevelErrors,
	fieldErrorMessage,
	projectGraphErrors,
} from '#/pages/automation/lib/validation'

function error(
	overrides: Partial<Schemas.GraphErrorResponse> = {},
): Schemas.GraphErrorResponse {
	return { message: 'invalid', ...overrides }
}

describe('projectGraphErrors', () => {
	it('is empty on an empty error list', () => {
		const result = projectGraphErrors([])

		expect(result.connectorErrors.size).toBe(0)
		expect(result.graphErrors).toEqual([])
	})

	it('badges the named connector for an error carrying only a connector_id', () => {
		const result = projectGraphErrors([
			error({
				connector_id: 'c1',
				message: 'Identifiant de credential invalide',
			}),
		])

		expect(result.connectorErrors.get('c1')).toEqual([
			{ field: null, message: 'Identifiant de credential invalide' },
		])
		expect(result.graphErrors).toEqual([])
	})

	it('keeps the field alongside the message for a field-scoped error', () => {
		const result = projectGraphErrors([
			error({ connector_id: 'c1', field: 'url', message: 'URL manquante' }),
		])

		expect(result.connectorErrors.get('c1')).toEqual([
			{ field: 'url', message: 'URL manquante' },
		])
	})

	it('banners an error with no connector_id as graph-shaped', () => {
		const result = projectGraphErrors([
			error({ message: 'Le graphe contient un cycle' }),
		])

		expect(result.graphErrors).toEqual(['Le graphe contient un cycle'])
		expect(result.connectorErrors.size).toBe(0)
	})

	it('treats an explicit null connector_id the same as an absent one', () => {
		const result = projectGraphErrors([
			error({ connector_id: null, message: 'Arête orpheline' }),
		])

		expect(result.graphErrors).toEqual(['Arête orpheline'])
		expect(result.connectorErrors.size).toBe(0)
	})

	it('accumulates several errors on the same connector, in order', () => {
		const result = projectGraphErrors([
			error({ connector_id: 'c1', field: 'url', message: 'URL manquante' }),
			error({
				connector_id: 'c1',
				field: 'method',
				message: 'Méthode invalide',
			}),
		])

		expect(result.connectorErrors.get('c1')).toEqual([
			{ field: 'url', message: 'URL manquante' },
			{ field: 'method', message: 'Méthode invalide' },
		])
	})

	it('accumulates several graph-shaped errors, in order', () => {
		const result = projectGraphErrors([
			error({ message: 'Le graphe contient un cycle' }),
			error({ message: 'Une arête pointe vers un connecteur inconnu' }),
		])

		expect(result.graphErrors).toEqual([
			'Le graphe contient un cycle',
			'Une arête pointe vers un connecteur inconnu',
		])
	})

	it('sorts errors onto distinct connectors independently', () => {
		const result = projectGraphErrors([
			error({ connector_id: 'c1', message: 'Erreur sur c1' }),
			error({ connector_id: 'c2', message: 'Erreur sur c2' }),
		])

		expect(result.connectorErrors.get('c1')).toEqual([
			{ field: null, message: 'Erreur sur c1' },
		])
		expect(result.connectorErrors.get('c2')).toEqual([
			{ field: null, message: 'Erreur sur c2' },
		])
	})
})

describe('fieldErrorMessage', () => {
	it('returns the message carried by the error naming that field', () => {
		const errors = [
			{ field: 'url', message: 'URL manquante' },
			{ field: 'method', message: 'Méthode invalide' },
		]

		expect(fieldErrorMessage(errors, 'url')).toBe('URL manquante')
		expect(fieldErrorMessage(errors, 'method')).toBe('Méthode invalide')
	})

	it('returns null when no error names that field', () => {
		const errors = [{ field: 'url', message: 'URL manquante' }]

		expect(fieldErrorMessage(errors, 'method')).toBeNull()
	})

	it('ignores a connector-level error (null field) when asked for a named field', () => {
		const errors = [{ field: null, message: 'Identifiant de credential manquant' }]

		expect(fieldErrorMessage(errors, 'credential_id')).toBeNull()
	})
})

describe('connectorLevelErrors', () => {
	it('keeps only the messages of errors carrying no field', () => {
		const errors = [
			{ field: null, message: 'Identifiant de credential manquant' },
			{ field: 'url', message: 'URL manquante' },
			{ field: null, message: 'Type de credential refusé' },
		]

		expect(connectorLevelErrors(errors)).toEqual([
			'Identifiant de credential manquant',
			'Type de credential refusé',
		])
	})

	it('is empty when every error names a field', () => {
		const errors = [{ field: 'url', message: 'URL manquante' }]

		expect(connectorLevelErrors(errors)).toEqual([])
	})
})

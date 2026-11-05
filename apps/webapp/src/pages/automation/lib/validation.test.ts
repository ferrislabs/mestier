import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	connectorLevelErrors,
	fieldErrorMessage,
	type GraphValidation,
	projectGraphErrors,
	pruneCorrectedErrors,
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
		const errors = [
			{ field: null, message: 'Identifiant de credential manquant' },
		]

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

describe('pruneCorrectedErrors', () => {
	const before: Schemas.GraphDto = {
		triggers: [{ id: 't1', kind: { Events: ['quote.sent'] } }],
		connectors: [
			{ id: 'c1', kind: 'http.request', version: 1, config: { url: '' } },
			{ id: 'c2', kind: 'http.request', version: 1, config: { url: 'x' } },
		],
		edges: [
			{ from: 't1', to: 'c1', branch: null },
			{ from: 'c1', to: 'c2', branch: null },
		],
	}

	function validation(): GraphValidation {
		return projectGraphErrors([
			error({ connector_id: 'c1', field: 'url', message: 'URL manquante' }),
			error({ connector_id: 'c2', field: 'url', message: 'URL invalide' }),
			error({ trigger_id: 't1', message: 'Ne mène nulle part' }),
			error({ message: 'Le graphe contient un cycle' }),
		])
	}

	it('drops the errors of the node whose settings changed, and only that node', () => {
		const after: Schemas.GraphDto = {
			...before,
			connectors: [
				{ ...before.connectors[0], config: { url: 'https://a.example' } },
				before.connectors[1],
			],
		}

		const result = pruneCorrectedErrors(validation(), before, after)

		expect([...result.connectorErrors.keys()]).toEqual(['c2', 't1'])
		expect(result.graphErrors).toEqual(['Le graphe contient un cycle'])
	})

	it('drops the errors of a trigger whose events changed', () => {
		const after: Schemas.GraphDto = {
			...before,
			triggers: [{ id: 't1', kind: { Events: ['quote.accepted'] } }],
		}

		const result = pruneCorrectedErrors(validation(), before, after)

		expect(result.connectorErrors.has('t1')).toBe(false)
		expect(result.connectorErrors.has('c1')).toBe(true)
	})

	it('drops the errors of both ends of an edge that was added or removed', () => {
		const after: Schemas.GraphDto = {
			...before,
			edges: [{ from: 't1', to: 'c1', branch: null }],
		}

		const result = pruneCorrectedErrors(validation(), before, after)

		expect([...result.connectorErrors.keys()]).toEqual(['t1'])
	})

	it('drops the errors of a deleted node', () => {
		const after: Schemas.GraphDto = {
			triggers: before.triggers,
			connectors: [before.connectors[1]],
			edges: [],
		}

		const result = pruneCorrectedErrors(validation(), before, after)

		expect(result.connectorErrors.has('c1')).toBe(false)
	})

	it('hands back the very same validation when nothing it names was touched', () => {
		const current = validation()
		const after: Schemas.GraphDto = {
			...before,
			connectors: [
				before.connectors[0],
				before.connectors[1],
				{ id: 'c3', kind: 'http.request', version: 1, config: {} },
			],
		}

		expect(pruneCorrectedErrors(current, before, after)).toBe(current)
		expect(pruneCorrectedErrors(current, before, before)).toBe(current)
	})
})

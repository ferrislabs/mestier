import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	connectorLevelErrors,
	fieldErrorMessage,
	type GraphValidation,
	graphErrorsOf,
	mergeValidations,
	nodeTextErrors,
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

describe('nodeTextErrors', () => {
	const NO_FIELDS: ReadonlySet<string> = new Set()

	it('puts a name error on the name, in French when the server message is known', () => {
		const result = nodeTextErrors(
			[
				{
					field: 'name',
					message:
						'connector `c2` reuses the name `Envoyer`, already given to another node',
				},
			],
			NO_FIELDS,
		)

		expect(result.name).toBe('Un autre nœud porte déjà ce nom.')
		expect(result.description).toBeNull()
		expect(result.rest).toEqual([])
	})

	it.each([
		['connector `c1` has an empty name', 'Le nom ne peut pas être vide.'],
		[
			'trigger `t1` has a name longer than 120 characters',
			'Le nom ne peut pas dépasser 120 caractères.',
		],
	])('translates “%s”', (message, expected) => {
		expect(nodeTextErrors([{ field: 'name', message }], NO_FIELDS).name).toBe(
			expected,
		)
	})

	it('puts a description error on the description', () => {
		const result = nodeTextErrors(
			[
				{
					field: 'description',
					message:
						'connector `c1` has a description longer than 500 characters',
				},
			],
			NO_FIELDS,
		)

		expect(result.description).toBe(
			'La description ne peut pas dépasser 500 caractères.',
		)
		expect(result.name).toBeNull()
	})

	it('shows a message it does not know as it came', () => {
		expect(
			nodeTextErrors([{ field: 'name', message: 'nom refusé' }], NO_FIELDS)
				.name,
		).toBe('nom refusé')
	})

	it('leaves the other errors of the node where they were', () => {
		const other = { field: 'url', message: 'manquant' }
		const connectorLevel = { field: null, message: 'identification inconnue' }

		const result = nodeTextErrors(
			[
				other,
				{ field: 'name', message: 'x has an empty name' },
				connectorLevel,
			],
			NO_FIELDS,
		)

		expect(result.rest).toEqual([other, connectorLevel])
	})

	it('does not take a problem of a connector field called “name” for a problem of the node name', () => {
		const configError = {
			field: 'name',
			message: 'missing required field `name`',
		}

		const result = nodeTextErrors([configError], new Set(['name']))

		expect(result.name).toBeNull()
		expect(result.rest).toEqual([configError])
	})

	it('still knows the node name error when the connector has a field called “name”', () => {
		const result = nodeTextErrors(
			[
				{
					field: 'name',
					message: 'connector `c1` has an empty name',
				},
			],
			new Set(['name']),
		)

		expect(result.name).toBe('Le nom ne peut pas être vide.')
		expect(result.rest).toEqual([])
	})

	it('has nothing to say about a node without errors', () => {
		expect(nodeTextErrors([], NO_FIELDS)).toEqual({
			name: null,
			description: null,
			rest: [],
		})
	})
})

describe('graphErrorsOf', () => {
	function refusal(status: number, errors: unknown) {
		return Object.assign(new Error('Invalid graph'), {
			status,
			data: { details: { errors } },
		})
	}

	it('reads the errors of a 422 carrying them', () => {
		expect(
			graphErrorsOf(refusal(422, [{ connector_id: 'c1', message: 'bad' }])),
		).toEqual([{ connector_id: 'c1', message: 'bad' }])
	})

	it('reads nothing from another status, another shape or something that is no error', () => {
		expect(graphErrorsOf(refusal(503, [{ message: 'bad' }]))).toBeNull()
		expect(graphErrorsOf(refusal(422, 'bad'))).toBeNull()
		expect(graphErrorsOf(refusal(422, []))).toBeNull()
		expect(
			graphErrorsOf(Object.assign(new Error('x'), { status: 422 })),
		).toBeNull()
		expect(graphErrorsOf('boom')).toBeNull()
		expect(graphErrorsOf(null)).toBeNull()
	})
})

describe('mergeValidations', () => {
	it('returns the same validation when the other one is empty', () => {
		const filled = projectGraphErrors([
			error({ connector_id: 'c1', message: 'a' }),
			error({ message: 'cycle' }),
		])
		const empty = projectGraphErrors([])

		expect(mergeValidations(filled, empty)).toBe(filled)
		expect(mergeValidations(empty, filled)).toBe(filled)
	})

	it('adds the errors of a node and of the graph from both sides, once each', () => {
		const left = projectGraphErrors([
			error({ connector_id: 'c1', field: 'url', message: 'URL manquante' }),
			error({ message: 'cycle' }),
		])
		const right = projectGraphErrors([
			error({ connector_id: 'c1', field: 'url', message: 'URL manquante' }),
			error({ connector_id: 'c1', message: 'Identification manquante' }),
			error({ connector_id: 'c2', message: 'autre' }),
			error({ message: 'cycle' }),
			error({ message: 'sans déclencheur' }),
		])

		const merged = mergeValidations(left, right)

		expect(merged.connectorErrors.get('c1')).toEqual([
			{ field: 'url', message: 'URL manquante' },
			{ field: null, message: 'Identification manquante' },
		])
		expect(merged.connectorErrors.get('c2')).toEqual([
			{ field: null, message: 'autre' },
		])
		expect(merged.graphErrors).toEqual(['cycle', 'sans déclencheur'])
	})
})

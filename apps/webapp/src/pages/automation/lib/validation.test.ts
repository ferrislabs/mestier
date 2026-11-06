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
	triggerErrorMessage,
	triggerFieldErrors,
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

	it('reads the errors of a 409 refusing a webhook path, naming the trigger and the field', () => {
		const errors = [{ trigger_id: 't1', field: 'path', message: 'taken' }]
		const taken = Object.assign(new Error('taken'), {
			status: 409,
			data: { code: 'E_WEBHOOK_PATH_TAKEN', details: { errors } },
		})

		expect(graphErrorsOf(taken)).toEqual(errors)
	})

	it('reads nothing from another 409', () => {
		expect(
			graphErrorsOf(
				Object.assign(new Error('archived'), {
					status: 409,
					data: {
						code: 'E_WORKFLOW_ARCHIVED',
						details: { errors: [{ message: 'x' }] },
					},
				}),
			),
		).toBeNull()
		expect(graphErrorsOf(refusal(409, [{ message: 'x' }]))).toBeNull()
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

describe('projectGraphErrors — the caps', () => {
	it('banners the caps of the organization and of the graph in French', () => {
		const result = projectGraphErrors([
			error({ message: 'an organization holds at most 200 webhook routes' }),
			error({ message: 'an organization holds at most 200 schedules' }),
			error({
				field: 'triggers',
				message: 'a graph holds at most 20 triggers',
			}),
			error({ message: 'Le graphe contient un cycle' }),
		])

		expect(result.graphErrors).toEqual([
			'Votre organisation a atteint la limite de 200 webhooks.',
			'Votre organisation a atteint la limite de 200 planificateurs.',
			'Un workflow compte au plus 20 déclencheurs.',
			'Le graphe contient un cycle',
		])
		expect(result.connectorErrors.size).toBe(0)
	})
})

describe('triggerFieldErrors', () => {
	function fields(entries: [string, string][]) {
		return triggerFieldErrors(
			entries.map(([field, message]) => ({ field, message })),
		)
	}

	it('says the path is taken, without telling by whom', () => {
		expect(
			fields([
				[
					'path',
					'trigger `t1` uses a webhook path another workflow already holds',
				],
			]).path,
		).toBe('Ce chemin est déjà utilisé.')
	})

	it('puts the other path refusals in French on the path field', () => {
		const path = (message: string) =>
			fields([
				['path', `trigger \`t1\` has an invalid webhook path: ${message}`],
			]).path

		expect(path('the path is empty')).toBe(
			'Indiquez la fin de l’URL après /webhooks/.',
		)
		expect(path('the path is longer than 200 characters')).toBe(
			'L’URL ne peut pas dépasser 200 caractères.',
		)
		expect(
			path('the path must not start with `/webhooks/`, which is added for you'),
		).toBe('Le préfixe /webhooks/ est déjà ajouté : retirez-le.')
		expect(path('the path must not contain a `..` segment')).toBe(
			'Lettres minuscules, chiffres, « - », « _ » et « / » uniquement.',
		)
		expect(
			path('the path only allows lowercase letters, digits, `-`, `_` and `/`'),
		).toBe('Lettres minuscules, chiffres, « - », « _ » et « / » uniquement.')
		expect(
			fields([
				['path', 'trigger `t1` reuses the webhook path `x` of another trigger'],
			]).path,
		).toBe('Cette URL est déjà utilisée par un autre déclencheur.')
	})

	it('keeps the credential, the time and the timezone on their own field', () => {
		const result = fields([
			['credential', 'trigger `t1` references unknown credential `c9`'],
			['time', 'trigger `t1` has the time `9h`, which is not HH:MM'],
			['timezone', 'trigger `t1` has the unknown timezone `Mars/Olympus`'],
		])

		expect(result.credential).toBe('Cette identification n’existe plus.')
		expect(result.time).toBe(
			'Indiquez une heure au format HH:MM, par exemple 09:00.',
		)
		expect(result.timezone).toBe('Ce fuseau horaire n’existe pas.')
		expect(
			fields([
				[
					'credential',
					'trigger `t1` credential `c1` has scheme `odoo_api`, which a webhook does not accept',
				],
			]).credential,
		).toBe('Cette identification n’est pas utilisable pour un webhook.')
	})

	it('sends the errors of a contract key to the key and the others to the contract', () => {
		const result = fields([
			['contract', 'trigger `t1` has a contract key with no name'],
			[
				'contract.montant',
				'trigger `t1` names the contract key `montant` more than once',
			],
			[
				'contract.1 x',
				'trigger `t1` names the contract key `1 x`, which is not a valid key name',
			],
		])

		expect(result.contract).toBe('Donnez un nom à cette clé.')
		expect(result.keys).toEqual({
			montant: 'Clé en double à ce niveau.',
			'1 x': 'Lettres, chiffres, « _ » et « - » uniquement.',
		})
	})

	it('keeps an error it cannot place, and the node level ones, in the rest', () => {
		const result = triggerFieldErrors([
			{ field: null, message: 'something else' },
			{ field: 'unheard', message: 'odd' },
		])

		expect(result.rest).toEqual([
			{ field: null, message: 'something else' },
			{ field: 'unheard', message: 'odd' },
		])
		expect(result.path).toBeNull()
	})

	it('shows an unknown message of a known field as the server wrote it', () => {
		expect(fields([['weekday', 'weekday is odd']]).weekday).toBe(
			'weekday is odd',
		)
	})

	it('keeps the first error of a field', () => {
		expect(
			fields([
				['timezone', 'trigger `t1` has the unknown timezone `a`'],
				['timezone', 'trigger `t1` has the unknown timezone `b`'],
			]).timezone,
		).toBe('Ce fuseau horaire n’existe pas.')
	})
})

describe('triggerErrorMessage', () => {
	it('puts the error of a trigger field in French', () => {
		expect(
			triggerErrorMessage({
				field: 'path',
				message:
					'trigger `t1` uses a webhook path another workflow already holds',
			}),
		).toBe('Ce chemin est déjà utilisé.')
		expect(
			triggerErrorMessage({
				field: 'contract.montant',
				message: 'trigger `t1` names the contract key `montant` more than once',
			}),
		).toBe('Clé en double à ce niveau.')
	})

	it('leaves the other errors as they are', () => {
		expect(triggerErrorMessage({ field: null, message: 'tel quel' })).toBe(
			'tel quel',
		)
		expect(triggerErrorMessage({ field: 'name', message: 'tel quel' })).toBe(
			'tel quel',
		)
	})
})

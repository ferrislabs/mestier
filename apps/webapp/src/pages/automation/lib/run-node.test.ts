import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	connectorTypeLabel,
	displayValue,
	isExpressionValue,
	nodeNeighbours,
	parameterEntries,
} from '#/pages/automation/lib/run-node'

const DESCRIPTOR = {
	kind: 'mail',
	label: 'Mail',
	family: 'mestier',
	auth: 'None',
	branches: [],
	fields: [
		{ name: 'to', label: 'Destinataire', secret: false },
		{ name: 'token', label: 'Jeton', secret: true },
	],
} as unknown as Schemas.ConnectorDescriptorResponse

function descriptorWith(
	overrides: Partial<Schemas.ConnectorDescriptorResponse>,
): Schemas.ConnectorDescriptorResponse {
	return { ...DESCRIPTOR, ...overrides }
}

function connectorOf(
	config: Record<string, unknown>,
	credentialId: string | null = null,
): Schemas.PlacedConnectorDto {
	return {
		id: 'c1',
		kind: 'mail',
		version: 1,
		config,
		credential_id: credentialId,
	}
}

const METHOD_FIELDS = [
	{
		name: 'method',
		label: 'Méthode',
		kind: {
			Select: {
				options: [
					{ value: 'GET', label: 'Lecture' },
					{ value: 'POST', label: 'Envoi' },
				],
			},
		},
		required: true,
		secret: false,
		expression: false,
	},
	{
		name: 'body',
		label: 'Corps',
		kind: 'Json',
		required: false,
		secret: false,
		expression: true,
		visible_when: { field: 'method', any_of: ['POST'] },
	},
] as unknown as Schemas.FieldResponse[]

describe('run node helpers', () => {
	it('stringifies values for display', () => {
		expect(displayValue(undefined)).toBe('')
		expect(displayValue('a')).toBe('a')
		expect(displayValue({ a: 1 })).toBe('{\n  "a": 1\n}')
	})

	it('detects expressions', () => {
		expect(isExpressionValue('{{ trigger.email }}')).toBe(true)
		expect(isExpressionValue('plain')).toBe(false)
		expect(isExpressionValue(3)).toBe(false)
	})

	it('lists described fields without secrets, then undeclared config keys', () => {
		const connector = {
			id: 'c1',
			kind: 'mail',
			version: 1,
			config: { to: '{{ trigger.email }}', token: 'x', extra: 1 },
		}
		expect(parameterEntries(connector, DESCRIPTOR)).toEqual([
			{
				id: 'to',
				label: 'Destinataire',
				value: '{{ trigger.email }}',
				template: '{{ trigger.email }}',
				expression: true,
			},
			{
				id: 'extra',
				label: 'extra',
				value: '1',
				template: 1,
				expression: false,
			},
		])
		expect(parameterEntries(null, DESCRIPTOR)).toEqual([])
	})

	it('leaves out a field whose condition is not met, and does not list it as an extra', () => {
		const descriptor = descriptorWith({ fields: METHOD_FIELDS })
		const entries = parameterEntries(
			connectorOf({ method: 'GET', body: '{"a":1}' }),
			descriptor,
		)
		expect(entries.map((entry) => entry.id)).toEqual(['method'])
	})

	it('lists a conditional field once its condition is met', () => {
		const descriptor = descriptorWith({ fields: METHOD_FIELDS })
		const entries = parameterEntries(
			connectorOf({ method: 'POST', body: '{{ trigger.payload }}' }),
			descriptor,
		)
		expect(entries.map((entry) => [entry.id, entry.expression])).toEqual([
			['method', false],
			['body', true],
		])
	})

	it('shows the label of the chosen option of a select', () => {
		const descriptor = descriptorWith({ fields: METHOD_FIELDS })
		const [method] = parameterEntries(
			connectorOf({ method: 'POST' }),
			descriptor,
		)
		expect(method?.value).toBe('Envoi')
		expect(method?.template).toBe('POST')
	})

	it('falls back to the raw value of a select when no option matches', () => {
		const descriptor = descriptorWith({ fields: METHOD_FIELDS })
		const [method] = parameterEntries(
			connectorOf({ method: 'PATCH' }),
			descriptor,
		)
		expect(method?.value).toBe('PATCH')
	})

	it('shows the name of the identification of the connector first', () => {
		const descriptor = descriptorWith({ auth: { Exactly: 'http_basic' } })
		const names = new Map([['cred-1', 'Compte Odoo']])
		const entries = parameterEntries(
			connectorOf({ to: 'a@b.fr' }, 'cred-1'),
			descriptor,
			names,
		)
		expect(
			entries.map((entry) => [entry.id, entry.label, entry.value]),
		).toEqual([
			['credential_id', 'Identification', 'Compte Odoo'],
			['to', 'Destinataire', 'a@b.fr'],
		])
	})

	it('says there is no identification when the connector has none', () => {
		const descriptor = descriptorWith({ auth: { Optional: ['http_basic'] } })
		const [credential] = parameterEntries(connectorOf({}), descriptor)
		expect(credential?.value).toBe('Aucune identification')
	})

	it('keeps the identifier of an identification that can no longer be found', () => {
		const descriptor = descriptorWith({ auth: { Exactly: 'http_basic' } })
		const [credential] = parameterEntries(
			connectorOf({}, 'cred-gone'),
			descriptor,
			new Map(),
		)
		expect(credential?.value).toBe('cred-gone')
	})

	it('has no identification line for a connector that needs none', () => {
		const entries = parameterEntries(connectorOf({}, 'cred-1'), DESCRIPTOR)
		expect(entries.map((entry) => entry.id)).not.toContain('credential_id')
	})

	it('shows the name of the signing identification stored in the config', () => {
		const descriptor = descriptorWith({
			fields: [
				{
					name: 'signing_credential_id',
					label: 'Signature',
					kind: 'Text',
					required: false,
					secret: false,
					expression: false,
				},
			] as unknown as Schemas.FieldResponse[],
		})
		const names = new Map([['cred-9', 'Clé de signature']])
		const withKey = parameterEntries(
			connectorOf({ signing_credential_id: 'cred-9' }),
			descriptor,
			names,
		)
		const without = parameterEntries(connectorOf({}), descriptor, names)
		expect(withKey[0]?.value).toBe('Clé de signature')
		expect(without[0]?.value).toBe('Aucune identification')
	})

	it('names the type of a connector after its kind, then its descriptor, then its kind', () => {
		expect(connectorTypeLabel('flow.condition', DESCRIPTOR)).toBe('Si')
		expect(connectorTypeLabel('mail', DESCRIPTOR)).toBe('Mail')
		expect(connectorTypeLabel('mail', undefined)).toBe('mail')
	})

	it('finds the neighbours of a node with their branch', () => {
		const graph: Schemas.GraphDto = {
			triggers: [],
			connectors: [],
			edges: [
				{ from: 'a', to: 'b' },
				{ from: 'b', to: 'c', branch: 'Then' },
				{ from: 'b', to: 'd', branch: 'Else' },
			],
		}
		expect(nodeNeighbours(graph, 'b')).toEqual({
			previous: [{ id: 'a', branch: undefined }],
			next: [
				{ id: 'c', branch: 'Then' },
				{ id: 'd', branch: 'Else' },
			],
		})
	})
})

import type { Schemas } from '#/api/api.client'
import type { JsonSchema } from '#/pages/automation/lib/collections'
import { pascalCase, typeOf } from '#/pages/automation/lib/schema-typescript'

export type ContractKey = Schemas.ContractKeyDto

export type ContractKind = Schemas.ContractKindDto

export type WebhookRule = Extract<
	Schemas.TriggerKindDto,
	{ Webhook: unknown }
>['Webhook']

export const CONTRACT_KINDS: readonly { value: ContractKind; label: string }[] =
	[
		{ value: 'Text', label: 'Texte' },
		{ value: 'Number', label: 'Nombre' },
		{ value: 'Boolean', label: 'Booléen' },
		{ value: 'Object', label: 'Objet' },
		{ value: 'List', label: 'Liste' },
	]

export const EXAMPLE_PLACEHOLDERS: Record<ContractKind, string> = {
	Text: 'texte',
	Number: '42',
	Boolean: 'true',
	Object: '{"nom": "Claire"}',
	List: '["urgent"]',
}

const KEY_NAME = /^[A-Za-z_][\w-]*$/

const KIND_SCHEMAS: Record<ContractKind, JsonSchema> = {
	Text: { type: 'string' },
	Number: { type: 'number' },
	Boolean: { type: 'boolean' },
	Object: { type: 'object' },
	List: { type: 'array' },
}

const DEFAULT_SAMPLES: Record<Exclude<ContractKind, 'Text'>, unknown> = {
	Number: 12.5,
	Boolean: true,
	Object: {},
	List: [],
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

export function defaultWebhook(): WebhookRule {
	return {
		method: 'POST',
		path: '',
		credential_id: null,
		contract: [],
		on_invalid: 'Reject',
	}
}

export function kindLabel(kind: ContractKind): string {
	return CONTRACT_KINDS.find((entry) => entry.value === kind)?.label ?? kind
}

export function newContractKey(name = ''): ContractKey {
	return { name, kind: 'Text', required: true, example: '' }
}

function parsedExample(kind: ContractKind, example: string): unknown {
	try {
		const value: unknown = JSON.parse(example.trim())
		switch (kind) {
			case 'Number':
				return typeof value === 'number' ? value : undefined
			case 'Boolean':
				return typeof value === 'boolean' ? value : undefined
			case 'Object':
				return isRecord(value) ? value : undefined
			case 'List':
				return Array.isArray(value) ? value : undefined
			default:
				return undefined
		}
	} catch {
		return undefined
	}
}

export function contractIssues(
	contract: readonly ContractKey[],
): (string | null)[] {
	return contract.map((key, index) => {
		if (key.name.trim() === '') return 'Donnez un nom à cette clé.'
		if (!KEY_NAME.test(key.name))
			return 'Lettres, chiffres, « _ » et « - » uniquement.'
		if (contract.findIndex((other) => other.name === key.name) !== index)
			return 'Clé en double à ce niveau.'
		const example = key.example?.trim() ?? ''
		if (
			key.kind !== 'Text' &&
			example !== '' &&
			parsedExample(key.kind, example) === undefined
		)
			return `Exemple invalide pour le type « ${kindLabel(key.kind)} ».`
		return null
	})
}

export function hasContractIssue(contract: readonly ContractKey[]): boolean {
	return contractIssues(contract).some((issue) => issue !== null)
}

function sampleValue(key: ContractKey): unknown {
	const raw = key.example?.trim() ?? ''
	if (key.kind === 'Text') return raw === '' ? 'texte' : raw
	return parsedExample(key.kind, raw) ?? DEFAULT_SAMPLES[key.kind]
}

export function sampleBody(
	contract: readonly ContractKey[],
): Record<string, unknown> {
	return Object.fromEntries(
		contract
			.filter((key) => key.name.trim() !== '')
			.map((key) => [key.name, sampleValue(key)]),
	)
}

export function contractSource(
	path: string,
	contract: readonly ContractKey[],
): string {
	const named = contract.filter((key) => KEY_NAME.test(key.name))
	const schema: JsonSchema = {
		type: 'object',
		properties: Object.fromEntries(
			named.map((key) => [key.name, KIND_SCHEMAS[key.kind]]),
		),
		required: named.filter((key) => key.required).map((key) => key.name),
	}
	return `export interface ${pascalCase(path) || 'Webhook'}Body ${typeOf(schema, new Set())}`
}

export function updateContractKey(
	contract: readonly ContractKey[],
	index: number,
	patch: Partial<ContractKey>,
): ContractKey[] {
	return contract.map((key, position) =>
		position === index ? { ...key, ...patch } : key,
	)
}

export function removeContractKey(
	contract: readonly ContractKey[],
	index: number,
): ContractKey[] {
	return contract.filter((_, position) => position !== index)
}

function inferKey(name: string, value: unknown): ContractKey {
	if (isRecord(value))
		return {
			name,
			kind: 'Object',
			required: true,
			example: JSON.stringify(value),
		}
	if (Array.isArray(value))
		return {
			name,
			kind: 'List',
			required: true,
			example: JSON.stringify(value),
		}
	if (typeof value === 'number')
		return { name, kind: 'Number', required: true, example: String(value) }
	if (typeof value === 'boolean')
		return { name, kind: 'Boolean', required: true, example: String(value) }
	return {
		name,
		kind: 'Text',
		required: true,
		example: typeof value === 'string' ? value : '',
	}
}

export function contractFromExample(value: unknown): ContractKey[] | null {
	if (!isRecord(value)) return null
	return Object.entries(value).map(([name, child]) => inferKey(name, child))
}

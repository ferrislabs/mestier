import type { Schemas } from '#/api/api.client'
import {
	isFieldVisible,
	isSelectKind,
	isSigningCredentialField,
} from '#/pages/automation/lib/connector-config'
import { connectorCopy } from '#/pages/automation/lib/connector-copy'

export function connectorTypeLabel(
	kind: string,
	descriptor: Schemas.ConnectorDescriptorResponse | undefined,
): string {
	return connectorCopy(kind)?.label ?? descriptor?.label ?? kind
}

export interface ParameterEntry {
	id: string
	label: string
	value: string
	template: unknown
	expression: boolean
}

export function displayValue(value: unknown): string {
	if (value === undefined || value === null) return ''
	return typeof value === 'string' ? value : JSON.stringify(value, null, 2)
}

export function isExpressionValue(value: unknown): boolean {
	return typeof value === 'string' && value.includes('{{')
}

const NO_CREDENTIAL = 'Aucune identification'

const CREDENTIAL_ENTRY_ID = 'credential_id'

function credentialValue(
	id: unknown,
	names: ReadonlyMap<string, string>,
): string {
	if (typeof id !== 'string' || id === '') return NO_CREDENTIAL
	return names.get(id) ?? id
}

function fieldValue(
	field: Schemas.FieldResponse,
	value: unknown,
	credentialNames: ReadonlyMap<string, string>,
): string {
	if (isSigningCredentialField(field))
		return credentialValue(value, credentialNames)
	if (isSelectKind(field.kind)) {
		const option = field.kind.Select.options.find(
			(candidate) => candidate.value === value,
		)
		if (option) return option.label
	}
	return displayValue(value)
}

export function parameterEntries(
	connector: Schemas.PlacedConnectorDto | null,
	descriptor: Schemas.ConnectorDescriptorResponse | undefined,
	credentialNames: ReadonlyMap<string, string> = new Map(),
): ParameterEntry[] {
	if (!connector) return []
	const known = new Set<string>()
	const entries: ParameterEntry[] = []
	if (descriptor && descriptor.auth !== 'None') {
		known.add(CREDENTIAL_ENTRY_ID)
		entries.push({
			id: CREDENTIAL_ENTRY_ID,
			label: 'Identification',
			value: credentialValue(connector.credential_id, credentialNames),
			template: connector.credential_id ?? null,
			expression: false,
		})
	}
	for (const field of descriptor?.fields ?? []) {
		known.add(field.name)
		if (field.secret || !isFieldVisible(field, connector.config)) continue
		entries.push({
			id: field.name,
			label: field.label,
			value: fieldValue(field, connector.config[field.name], credentialNames),
			template: connector.config[field.name],
			expression: isExpressionValue(connector.config[field.name]),
		})
	}
	for (const [key, value] of Object.entries(connector.config)) {
		if (known.has(key)) continue
		entries.push({
			id: key,
			label: key,
			value: displayValue(value),
			template: value,
			expression: isExpressionValue(value),
		})
	}
	return entries
}

export interface NodeNeighbours {
	previous: { id: string; branch?: string }[]
	next: { id: string; branch?: string }[]
}

export function nodeNeighbours(
	graph: Schemas.GraphDto,
	nodeId: string,
): NodeNeighbours {
	return {
		previous: graph.edges
			.filter((edge) => edge.to === nodeId)
			.map((edge) => ({ id: edge.from, branch: edge.branch ?? undefined })),
		next: graph.edges
			.filter((edge) => edge.from === nodeId)
			.map((edge) => ({ id: edge.to, branch: edge.branch ?? undefined })),
	}
}

const PASS_OF_LOOP = /^(.*)\[(\d+)\]$/

export function loopFrameOf(
	step: Schemas.RunStepResponse | null,
	steps: readonly Schemas.RunStepResponse[],
): Schemas.LoopFrameBody | null {
	if (!step || step.iteration_path === '') return null
	const segments = step.iteration_path.split('.')
	const match = PASS_OF_LOOP.exec(segments.pop() ?? '')
	if (!match) return null
	const index = Number(match[2])
	const parentPath = segments.join('.')
	const loop = steps.find(
		(candidate) =>
			candidate.connector_id === match[1] &&
			candidate.iteration_path === parentPath,
	)
	const output = loop?.output
	if (typeof output !== 'object' || output === null || Array.isArray(output)) {
		return null
	}
	const each = (output as Record<string, unknown>).Each
	if (!Array.isArray(each) || index >= each.length) return null
	return { index, item: each[index] }
}

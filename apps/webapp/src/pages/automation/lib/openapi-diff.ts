import type {
	CollectionDraft,
	CollectionOperation,
	JsonSchema,
	OperationParameter,
} from '#/pages/automation/lib/collections'
import {
	resolveSchema,
	schemaType,
} from '#/pages/automation/lib/schema-typescript'

export type VersionChange =
	| 'major'
	| 'minor'
	| 'patch'
	| 'same'
	| 'downgrade'
	| 'different'

export type ChangeSeverity = 'breaking' | 'compatible'

export type OperationChange = { severity: ChangeSeverity; message: string }

export type ChangedOperation = {
	previous: CollectionOperation
	next: CollectionOperation
	changes: readonly OperationChange[]
}

export type CollectionDiff = {
	previousVersion: string
	nextVersion: string
	versionChange: VersionChange
	added: readonly CollectionOperation[]
	removed: readonly CollectionOperation[]
	changed: readonly ChangedOperation[]
	breakingCount: number
	identical: boolean
}

type Schemas = Readonly<Record<string, JsonSchema>>

type FieldShape = { type: string; required: boolean }

const VERSION = /^v?(\d+)(?:\.(\d+))?(?:\.(\d+))?$/

const LEVELS = ['major', 'minor', 'patch'] as const

function parseVersion(version: string): number[] | null {
	const match = VERSION.exec(version.trim())
	return match ? [1, 2, 3].map((index) => Number(match[index] ?? 0)) : null
}

export function compareVersions(previous: string, next: string): VersionChange {
	if (previous.trim() === next.trim()) return 'same'
	const before = parseVersion(previous)
	const after = parseVersion(next)
	if (!before || !after) return 'different'
	for (const [index, level] of LEVELS.entries()) {
		if (after[index] > before[index]) return level
		if (after[index] < before[index]) return 'downgrade'
	}
	return 'same'
}

function flatten(
	schema: JsonSchema,
	schemas: Schemas,
	prefix: string,
	out: Map<string, FieldShape>,
	depth = 0,
): Map<string, FieldShape> {
	if (depth > 5) return out
	const resolved = resolveSchema(schema, schemas)
	if (resolved.items && !resolved.properties) {
		flatten(resolved.items, schemas, `${prefix}[]`, out, depth + 1)
		return out
	}
	const required = new Set(resolved.required ?? [])
	for (const [name, property] of Object.entries(resolved.properties ?? {})) {
		const path = prefix ? `${prefix}.${name}` : name
		out.set(path, {
			type: schemaType(property, schemas),
			required: required.has(name),
		})
		const child = resolveSchema(property, schemas)
		if (child.properties || child.items)
			flatten(child, schemas, path, out, depth + 1)
	}
	return out
}

function fieldsOf(schema: JsonSchema | null, schemas: Schemas) {
	return schema ? flatten(schema, schemas, '', new Map()) : new Map()
}

function parentPath(path: string): string | null {
	const index = path.lastIndexOf('.')
	return index === -1 ? null : path.slice(0, index)
}

function topmost(paths: readonly string[]): string[] {
	const set = new Set(paths)
	return paths.filter((path) => {
		for (let parent = parentPath(path); parent; parent = parentPath(parent))
			if (set.has(parent)) return false
		return true
	})
}

function schemaChanges(
	where: 'body' | 'response',
	previous: JsonSchema | null,
	previousSchemas: Schemas,
	next: JsonSchema | null,
	nextSchemas: Schemas,
): OperationChange[] {
	const place = where === 'body' ? 'du corps' : 'de la réponse'
	if (!previous && !next) return []
	if (previous && !next)
		return [
			{
				severity: 'breaking',
				message:
					where === 'body'
						? 'Le corps de requête a été supprimé'
						: 'La réponse ne décrit plus de contenu',
			},
		]
	const before = fieldsOf(previous, previousSchemas)
	const after = fieldsOf(next, nextSchemas)
	const changes: OperationChange[] = []
	const added = topmost([...after.keys()].filter((path) => !before.has(path)))
	const removed = topmost([...before.keys()].filter((path) => !after.has(path)))
	for (const path of added) {
		const required = where === 'body' && after.get(path)?.required
		changes.push({
			severity: required ? 'breaking' : 'compatible',
			message:
				where === 'body'
					? `Nouveau champ ${required ? 'requis' : 'facultatif'} « ${path} » dans le corps`
					: `Nouveau champ « ${path} » dans la réponse`,
		})
	}
	for (const path of removed)
		changes.push({
			severity: 'breaking',
			message: `Champ « ${path} » retiré ${place}`,
		})
	for (const [path, shape] of after) {
		const old = before.get(path)
		if (!old) continue
		if (old.type !== shape.type)
			changes.push({
				severity: 'breaking',
				message: `Type de « ${path} » modifié ${place} : ${old.type} → ${shape.type}`,
			})
		if (where === 'body' && old.required !== shape.required)
			changes.push({
				severity: shape.required ? 'breaking' : 'compatible',
				message: `« ${path} » devient ${shape.required ? 'requis' : 'facultatif'} dans le corps`,
			})
	}
	return changes
}

function parameterKey(parameter: OperationParameter): string {
	return `${parameter.location}:${parameter.name}`
}

function parameterChanges(
	previous: CollectionOperation,
	previousSchemas: Schemas,
	next: CollectionOperation,
	nextSchemas: Schemas,
): OperationChange[] {
	const before = new Map(previous.parameters.map((p) => [parameterKey(p), p]))
	const after = new Map(next.parameters.map((p) => [parameterKey(p), p]))
	const changes: OperationChange[] = []
	for (const [key, parameter] of after) {
		const old = before.get(key)
		if (!old) {
			changes.push({
				severity: parameter.required ? 'breaking' : 'compatible',
				message: `Nouveau paramètre ${parameter.required ? 'requis' : 'facultatif'} « ${parameter.name} »`,
			})
			continue
		}
		const oldType = schemaType(old.schema, previousSchemas)
		const newType = schemaType(parameter.schema, nextSchemas)
		if (oldType !== newType)
			changes.push({
				severity: 'breaking',
				message: `Type du paramètre « ${parameter.name} » modifié : ${oldType} → ${newType}`,
			})
		if (!old.required && parameter.required)
			changes.push({
				severity: 'breaking',
				message: `Le paramètre « ${parameter.name} » devient requis`,
			})
	}
	for (const [key, parameter] of before)
		if (!after.has(key))
			changes.push({
				severity: 'breaking',
				message: `Paramètre « ${parameter.name} » supprimé`,
			})
	return changes
}

function operationKey(operation: CollectionOperation): string {
	return `${operation.method} ${operation.path}`
}

function matchOperations(previous: CollectionDraft, next: CollectionDraft) {
	const byId = new Map(previous.operations.map((op) => [op.id, op]))
	const byKey = new Map(previous.operations.map((op) => [operationKey(op), op]))
	const matched = new Map<string, CollectionOperation>()
	for (const operation of next.operations) {
		const old = byId.get(operation.id) ?? byKey.get(operationKey(operation))
		if (old && ![...matched.values()].includes(old))
			matched.set(operation.id, old)
	}
	return matched
}

export function diffCollections(
	previous: CollectionDraft,
	next: CollectionDraft,
): CollectionDiff {
	const matched = matchOperations(previous, next)
	const matchedPrevious = new Set(matched.values())
	const added = next.operations.filter(
		(operation) => !matched.has(operation.id),
	)
	const removed = previous.operations.filter(
		(operation) => !matchedPrevious.has(operation),
	)
	const changed: ChangedOperation[] = []
	for (const operation of next.operations) {
		const old = matched.get(operation.id)
		if (!old) continue
		const changes: OperationChange[] = []
		if (old.method !== operation.method)
			changes.push({
				severity: 'breaking',
				message: `Méthode modifiée : ${old.method} → ${operation.method}`,
			})
		if (old.path !== operation.path)
			changes.push({
				severity: 'compatible',
				message: `Chemin modifié : ${old.path} → ${operation.path}`,
			})
		changes.push(
			...parameterChanges(old, previous.schemas, operation, next.schemas),
			...schemaChanges(
				'body',
				old.requestBody?.schema ?? null,
				previous.schemas,
				operation.requestBody?.schema ?? null,
				next.schemas,
			),
			...schemaChanges(
				'response',
				old.response,
				previous.schemas,
				operation.response,
				next.schemas,
			),
		)
		if (changes.length > 0)
			changed.push({ previous: old, next: operation, changes })
	}
	const breakingCount =
		removed.length +
		changed.reduce(
			(total, entry) =>
				total +
				entry.changes.filter((change) => change.severity === 'breaking').length,
			0,
		)
	return {
		previousVersion: previous.version,
		nextVersion: next.version,
		versionChange: compareVersions(previous.version, next.version),
		added,
		removed,
		changed,
		breakingCount,
		identical:
			added.length === 0 && removed.length === 0 && changed.length === 0,
	}
}

export function versionWarning(diff: CollectionDiff): string | null {
	const { versionChange, previousVersion, nextVersion } = diff
	if (versionChange === 'downgrade')
		return `La version importée (${nextVersion}) est antérieure à la version actuelle (${previousVersion}).`
	if (versionChange === 'same' && !diff.identical)
		return `Le contenu a changé mais la version est restée ${nextVersion} : l’éditeur de l’API n’a pas publié de nouvelle version.`
	const stable = (parseVersion(previousVersion)?.[0] ?? 0) >= 1
	if (
		diff.breakingCount > 0 &&
		stable &&
		(versionChange === 'minor' || versionChange === 'patch')
	)
		return `Changements incompatibles dans une version ${versionChange === 'minor' ? 'mineure' : 'corrective'} : la spec ne respecte pas le versionnage sémantique.`
	return null
}

export function withStableOperationIds(
	previous: CollectionDraft,
	next: CollectionDraft,
): CollectionDraft {
	const matched = matchOperations(previous, next)
	const taken = new Set<string>()
	const operations = next.operations.map((operation) => {
		const old = matched.get(operation.id)
		const id = old && !taken.has(old.id) ? old.id : operation.id
		taken.add(id)
		return id === operation.id ? operation : { ...operation, id }
	})
	return { ...next, operations }
}

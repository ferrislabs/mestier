import { parse as parseYaml } from 'yaml'
import type {
	ApiKeyLocation,
	CollectionDraft,
	CollectionOperation,
	CollectionSource,
	HttpMethod,
	JsonSchema,
	OperationParameter,
	ParameterLocation,
	SecurityKind,
	SecurityScheme,
} from '#/pages/automation/lib/collections'

export class OpenApiError extends Error {}

type Dictionary = Record<string, unknown>

const METHODS: readonly HttpMethod[] = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE']
const LOCATIONS: readonly ParameterLocation[] = ['path', 'query', 'header']
const API_KEY_LOCATIONS: readonly ApiKeyLocation[] = [
	'header',
	'query',
	'cookie',
]
const SERVER_VARIABLE = /\{([^{}]*)\}/g

function isDictionary(value: unknown): value is Dictionary {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function text(value: unknown): string {
	return typeof value === 'string' ? value : ''
}

export function slugify(value: string): string {
	return value
		.normalize('NFD')
		.replace(/\p{Diacritic}/gu, '')
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-|-$/g, '')
}

export function readSpecText(source: string): unknown {
	const trimmed = source.trim()
	if (trimmed === '') throw new OpenApiError('Le fichier est vide.')
	try {
		return trimmed.startsWith('{') ? JSON.parse(trimmed) : parseYaml(trimmed)
	} catch {
		throw new OpenApiError(
			'Le contenu n’est ni du JSON ni du YAML valide. Vérifiez le fichier.',
		)
	}
}

function resolveReference(document: Dictionary, reference: string): unknown {
	if (!reference.startsWith('#/')) return undefined
	return reference
		.slice(2)
		.split('/')
		.reduce<unknown>(
			(node, key) => (isDictionary(node) ? node[key] : undefined),
			document,
		)
}

function dereference(document: Dictionary, value: unknown): unknown {
	if (isDictionary(value) && typeof value.$ref === 'string') {
		return resolveReference(document, value.$ref)
	}
	return value
}

function jsonBodySchema(
	document: Dictionary,
	holder: unknown,
): JsonSchema | null {
	const resolved = dereference(document, holder)
	if (!isDictionary(resolved) || !isDictionary(resolved.content)) return null
	const media =
		resolved.content['application/json'] ??
		Object.entries(resolved.content).find(([type]) =>
			type.includes('json'),
		)?.[1]
	if (!isDictionary(media) || !isDictionary(media.schema)) return null
	return media.schema as JsonSchema
}

function readParameters(
	document: Dictionary,
	raw: unknown,
): OperationParameter[] {
	if (!Array.isArray(raw)) return []
	return raw.flatMap((entry) => {
		const parameter = dereference(document, entry)
		if (!isDictionary(parameter)) return []
		const location = parameter.in as ParameterLocation
		if (!LOCATIONS.includes(location)) return []
		return [
			{
				name: text(parameter.name),
				location,
				required: location === 'path' || parameter.required === true,
				description: text(parameter.description),
				schema: isDictionary(parameter.schema)
					? (parameter.schema as JsonSchema)
					: { type: 'string' },
			},
		]
	})
}

function mergeParameters(
	shared: readonly OperationParameter[],
	own: readonly OperationParameter[],
): OperationParameter[] {
	const key = (parameter: OperationParameter) =>
		`${parameter.location}:${parameter.name}`
	const ownKeys = new Set(own.map(key))
	return [...shared.filter((parameter) => !ownKeys.has(key(parameter))), ...own]
}

function successResponse(
	document: Dictionary,
	responses: unknown,
): { status: string | null; schema: JsonSchema | null } {
	if (!isDictionary(responses)) return { status: null, schema: null }
	const status = Object.keys(responses)
		.filter((code) => /^2\d\d$/.test(code))
		.sort()[0]
	if (!status) return { status: null, schema: null }
	return { status, schema: jsonBodySchema(document, responses[status]) }
}

function requestBody(
	document: Dictionary,
	holder: unknown,
): CollectionOperation['requestBody'] {
	const schema = jsonBodySchema(document, holder)
	return schema ? { schema } : null
}

function readOperations(document: Dictionary): CollectionOperation[] {
	const paths = isDictionary(document.paths) ? document.paths : {}
	const usedIds = new Set<string>()
	return Object.entries(paths).flatMap(([path, rawItem]) => {
		const item = dereference(document, rawItem)
		if (!isDictionary(item)) return []
		const shared = readParameters(document, item.parameters)
		return METHODS.flatMap((method) => {
			const operation = item[method.toLowerCase()]
			if (!isDictionary(operation)) return []
			const baseId =
				text(operation.operationId) ||
				slugify(`${method} ${path}`) ||
				'operation'
			let id = baseId
			for (let suffix = 2; usedIds.has(id); suffix += 1)
				id = `${baseId}-${suffix}`
			usedIds.add(id)
			const response = successResponse(document, operation.responses)
			const tags = Array.isArray(operation.tags) ? operation.tags : []
			return [
				{
					id,
					method,
					path,
					summary: text(operation.summary) || `${method} ${path}`,
					tag: text(tags[0]) || 'Autres',
					parameters: mergeParameters(
						shared,
						readParameters(document, operation.parameters),
					),
					requestBody: requestBody(document, operation.requestBody),
					response: response.schema,
					responseStatus: response.status,
				},
			]
		})
	})
}

function securityKind(scheme: unknown): SecurityKind {
	if (!isDictionary(scheme)) return { type: 'other', declaredType: 'inconnu' }
	const declaredType = text(scheme.type)
	switch (declaredType) {
		case 'http':
			return text(scheme.scheme)
				? { type: 'http', scheme: text(scheme.scheme) }
				: { type: 'other', declaredType }
		case 'apiKey': {
			const location = scheme.in as ApiKeyLocation
			return API_KEY_LOCATIONS.includes(location) && text(scheme.name)
				? { type: 'api_key', location, parameterName: text(scheme.name) }
				: { type: 'other', declaredType }
		}
		case 'oauth2':
			return { type: 'oauth2' }
		case 'openIdConnect':
			return { type: 'open_id_connect' }
		default:
			return { type: 'other', declaredType: declaredType || 'inconnu' }
	}
}

function readSecurity(
	document: Dictionary,
	securitySchemes: Dictionary,
): SecurityScheme[] {
	return Object.entries(securitySchemes).map(([name, scheme]) => ({
		name,
		kind: securityKind(dereference(document, scheme)),
	}))
}

function resolveServerVariables(server: Dictionary): string {
	const variables = isDictionary(server.variables) ? server.variables : {}
	return text(server.url).replace(SERVER_VARIABLE, (match, name: string) => {
		const variable = variables[name]
		const value = isDictionary(variable) ? variable.default : undefined
		return typeof value === 'string' || typeof value === 'number'
			? String(value)
			: match
	})
}

function parseUrl(value: string, base?: string): URL | null {
	try {
		return new URL(value, base)
	} catch {
		return null
	}
}

function readBaseUrl(
	server: Dictionary | undefined,
	source: CollectionSource,
): string {
	if (!server) return ''
	const raw = resolveServerVariables(server).trim()
	if (raw === '') return ''
	if (/[{}]/.test(raw)) {
		throw new OpenApiError(
			`L’URL du serveur « ${raw} » contient une variable sans valeur par défaut. Renseignez-la dans la spec avant de l’importer.`,
		)
	}
	const absolute = parseUrl(raw)
	const url =
		absolute ?? (source.kind === 'url' ? parseUrl(raw, source.url) : null)
	if (!url) {
		throw new OpenApiError(
			`L’URL du serveur « ${raw} » n’est pas une adresse complète. Indiquez une URL en http ou https dans la spec.`,
		)
	}
	if (url.protocol !== 'http:' && url.protocol !== 'https:') {
		throw new OpenApiError(
			`L’URL du serveur « ${raw} » doit commencer par http ou https.`,
		)
	}
	if (url.username !== '' || url.password !== '') {
		throw new OpenApiError(
			'L’URL du serveur ne doit pas contenir d’identifiants.',
		)
	}
	if (/[?#]/.test(raw)) {
		throw new OpenApiError(
			'L’URL du serveur ne doit contenir ni paramètres de requête ni fragment.',
		)
	}
	return absolute ? raw : url.href
}

export function parseOpenApi(
	raw: unknown,
	source: CollectionSource,
): CollectionDraft {
	if (!isDictionary(raw)) {
		throw new OpenApiError('Le document n’est pas un objet OpenAPI.')
	}
	if (typeof raw.swagger === 'string') {
		throw new OpenApiError(
			'Les specs Swagger 2.0 ne sont pas prises en charge. Convertissez-la en OpenAPI 3.',
		)
	}
	if (typeof raw.openapi !== 'string' || !raw.openapi.startsWith('3.')) {
		throw new OpenApiError(
			'Champ « openapi » absent ou invalide : seules les specs OpenAPI 3 sont prises en charge.',
		)
	}
	const info = isDictionary(raw.info) ? raw.info : {}
	const title = text(info.title) || 'Collection sans titre'
	const servers = Array.isArray(raw.servers) ? raw.servers : []
	const components = isDictionary(raw.components) ? raw.components : {}
	const securitySchemes = isDictionary(components.securitySchemes)
		? components.securitySchemes
		: {}
	const operations = readOperations(raw)
	if (operations.length === 0) {
		throw new OpenApiError('La spec ne décrit aucune opération dans « paths ».')
	}
	return {
		title,
		version: text(info.version) || '—',
		description: text(info.description),
		baseUrl: readBaseUrl(servers.find(isDictionary), source),
		source,
		security: readSecurity(raw, securitySchemes),
		operations,
		schemas: isDictionary(components.schemas)
			? (components.schemas as Record<string, JsonSchema>)
			: {},
	}
}

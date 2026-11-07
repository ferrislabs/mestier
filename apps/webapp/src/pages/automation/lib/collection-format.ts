import type {
	ApiKeyLocation,
	CollectionSource,
	SecurityScheme,
} from '#/pages/automation/lib/collections'

const DATE_FORMAT = new Intl.DateTimeFormat('fr-FR', {
	day: 'numeric',
	month: 'long',
	year: 'numeric',
})

const GENERIC_ERROR = 'Une erreur est survenue. Réessayez dans un instant.'
const VALIDATION_PREFIX = /^Validation failed:\s*(?:invalid collection:\s*)?/
const FETCHER_FALLBACK = /^HTTP \d{3}\b/

const API_KEY_LOCATION_LABELS: Record<ApiKeyLocation, string> = {
	header: 'en-tête',
	query: 'paramètre',
	cookie: 'cookie',
}

export function describeSource(source: CollectionSource): string {
	return source.kind === 'url'
		? `Importée depuis ${source.url}`
		: `Importée depuis le fichier ${source.fileName}`
}

export function formatImportDate(isoDate: string): string {
	const date = new Date(isoDate)
	return Number.isNaN(date.getTime()) ? '' : DATE_FORMAT.format(date)
}

export function operationCount(count: number): string {
	return `${count} opération${count > 1 ? 's' : ''}`
}

export function describeSecurity({ name, kind }: SecurityScheme): string {
	switch (kind.type) {
		case 'http':
			return `${name} (http ${kind.scheme})`
		case 'api_key':
			return `${name} (apiKey, ${API_KEY_LOCATION_LABELS[kind.location]} ${kind.parameterName})`
		case 'oauth2':
			return `${name} (oauth2)`
		case 'open_id_connect':
			return `${name} (openIdConnect)`
		case 'other':
			return `${name} (${kind.declaredType})`
	}
}

function errorStatus(error: unknown): number | null {
	if (typeof error !== 'object' || error === null) return null
	const { status } = error as { status?: unknown }
	return typeof status === 'number' ? status : null
}

function refusalReason(error: unknown): string | null {
	if (!(error instanceof Error)) return null
	const reason = error.message.replace(VALIDATION_PREFIX, '').trim()
	return reason === '' || FETCHER_FALLBACK.test(reason) ? null : reason
}

export function isCollectionMissing(error: unknown): boolean {
	return errorStatus(error) === 404
}

export function collectionLoadErrorMessage(error: unknown): string | null {
	if (error == null) return null
	return errorStatus(error) === 403
		? 'Vous n’avez pas la permission de voir les collections.'
		: GENERIC_ERROR
}

export function collectionErrorMessage(error: unknown): string | null {
	if (error == null) return null
	switch (errorStatus(error)) {
		case 400: {
			const reason = refusalReason(error)
			return reason
				? `Le serveur refuse cette collection : ${reason}.`
				: 'Le serveur refuse cette collection : son modèle n’est pas valide.'
		}
		case 403:
			return 'Vous n’avez pas la permission de gérer les collections.'
		case 404:
			return 'Cette collection n’existe plus.'
		case 409:
			return 'Cette organisation a atteint la limite de 200 collections. Supprimez-en une avant d’en importer une autre.'
		case 413:
			return 'Cette collection est trop volumineuse : le serveur refuse une spec de plus de 2 Mo. Retirez les opérations ou les schémas dont vous n’avez pas besoin, puis réessayez.'
		default:
			return GENERIC_ERROR
	}
}

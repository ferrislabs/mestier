import type {
	CollectionDraft,
	CollectionSource,
} from '#/pages/automation/lib/collections'
import {
	OpenApiError,
	parseOpenApi,
	readSpecText,
} from '#/pages/automation/lib/openapi'

export class SpecLoadError extends Error {}

function parse(text: string, source: CollectionSource): CollectionDraft {
	try {
		return parseOpenApi(readSpecText(text), source)
	} catch (caught) {
		throw new SpecLoadError(
			caught instanceof OpenApiError
				? caught.message
				: 'La spec n’a pas pu être lue. Vérifiez son contenu.',
		)
	}
}

export async function loadSpecFromUrl(url: string): Promise<CollectionDraft> {
	let response: Response
	try {
		response = await fetch(url)
	} catch {
		throw new SpecLoadError(
			'Impossible de récupérer la spec : le serveur ne répond pas ou refuse l’accès depuis le navigateur (CORS). Téléchargez le fichier et importez-le.',
		)
	}
	if (!response.ok)
		throw new SpecLoadError(
			`Le serveur a répondu ${response.status}. Vérifiez l’URL.`,
		)
	return parse(await response.text(), { kind: 'url', url })
}

export async function loadSpecFromFile(file: File): Promise<CollectionDraft> {
	return parse(await file.text(), { kind: 'file', fileName: file.name })
}

export function loadErrorMessage(caught: unknown): string {
	return caught instanceof SpecLoadError
		? caught.message
		: 'La spec n’a pas pu être lue. Vérifiez son contenu.'
}

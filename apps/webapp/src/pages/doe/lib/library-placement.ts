import type { PhotoPlacement, PlacementResult } from '#/hooks/use-photo-library'
import type { DoeFile } from '#/pages/doe/types'

export type LibraryTarget =
	| { kind: 'section'; sectionId: string }
	| { kind: 'supply'; supplyId: string }

export function buildPlacements(
	photoIds: readonly string[],
	target: LibraryTarget,
): PhotoPlacement[] {
	return photoIds.map((photoId) =>
		target.kind === 'section'
			? { photo_id: photoId, target: 'section', section_id: target.sectionId }
			: { photo_id: photoId, target: 'supply', supply_id: target.supplyId },
	)
}

export function failedResults(
	results: readonly PlacementResult[],
): PlacementResult[] {
	return results.filter((result) => result.status === 'failed')
}

export function placedIds(results: readonly PlacementResult[]): Set<string> {
	return new Set(
		results
			.filter((result) => result.status === 'placed')
			.map((result) => result.photo_id),
	)
}

export function failureLabel(result: PlacementResult): string {
	return result.error?.trim() || 'Cette photo n’a pas pu être placée.'
}

export function mergePhotos(
	local: readonly DoeFile[],
	saved: readonly DoeFile[],
): DoeFile[] {
	const known = new Set(local.map((photo) => photo.key))
	return [...local, ...saved.filter((photo) => !known.has(photo.key))]
}

export function placementSummary(placed: number, failed: number): string {
	const parts: string[] = []
	if (placed > 0)
		parts.push(
			`${placed} photo${placed > 1 ? 's' : ''} placée${placed > 1 ? 's' : ''}`,
		)
	if (failed > 0) parts.push(`${failed} en échec`)
	return parts.join(', ')
}

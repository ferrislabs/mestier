import type { PhotosPerPage } from '#/pages/doe/types'
import { PER_PAGE_VALUES } from '#/pages/doe/types'

export function photoPageCount(
	photoCount: number,
	perPage: PhotosPerPage,
): number {
	if (photoCount <= 0) return 0
	return Math.ceil(photoCount / PER_PAGE_VALUES[perPage])
}

export function photoSummary(
	photoCount: number,
	perPage: PhotosPerPage,
): string {
	const pages = photoPageCount(photoCount, perPage)
	const photoWord = photoCount > 1 ? 'photos' : 'photo'
	const pageWord = pages > 1 ? 'pages' : 'page'
	return `${photoCount} ${photoWord}, ${PER_PAGE_VALUES[perPage]} par page = ${pages} ${pageWord}`
}

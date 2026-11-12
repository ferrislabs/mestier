import type { Schemas } from '#/api/api.client'
import {
	moveRect,
	PAGE_HEIGHT_MM,
	PAGE_WIDTH_MM,
	type Rect,
	type ResizeHandle,
	resizeRect,
} from '#/pages/doe-style/lib/decorations'

export type CoverBlock = Schemas.CoverBlock
export type CoverLayout = Schemas.CoverLayout
export type CoverBlockKey = keyof CoverLayout

export const COVER_BLOCK_KEYS: CoverBlockKey[] = [
	'logo',
	'company',
	'place',
	'title',
	'meta',
	'photo',
]

export const COVER_BLOCK_LABELS: Record<CoverBlockKey, string> = {
	logo: 'Logo',
	company: 'Infos entreprise',
	place: 'Lieu',
	title: 'Bandeau titre',
	meta: 'Bloc rédacteur / date / indice',
	photo: 'Photo principale',
}

export const COVER_MARGIN_MM = 20
export const COVER_BOTTOM_MARGIN_MM = 20
export const COVER_TITLE_MIN_HEIGHT_MM = 50

const EPSILON = 1e-6
const CONTENT_WIDTH_MM = PAGE_WIDTH_MM - 2 * COVER_MARGIN_MM

function frame(
	x: number,
	y: number,
	width: number,
	height: number,
): CoverBlock {
	return { x, y, width, height, visible: true }
}

export function defaultCoverLayout(): CoverLayout {
	const titleTop = 90
	const titleHeight = 55
	const metaTop = titleTop + titleHeight + 7
	const metaHeight = 36
	const photoTop = metaTop + metaHeight + 8
	return {
		logo: frame(COVER_MARGIN_MM, 24, 50, 25),
		company: frame(130, 24, 60, 25),
		place: frame(COVER_MARGIN_MM, 56, CONTENT_WIDTH_MM, 14),
		title: frame(COVER_MARGIN_MM, titleTop, CONTENT_WIDTH_MM, titleHeight),
		meta: frame(COVER_MARGIN_MM, metaTop, CONTENT_WIDTH_MM, metaHeight),
		photo: frame(
			COVER_MARGIN_MM,
			photoTop,
			CONTENT_WIDTH_MM,
			PAGE_HEIGHT_MM - COVER_BOTTOM_MARGIN_MM - photoTop,
		),
	}
}

export function blockRect(value: CoverBlock): Rect {
	return { x: value.x, y: value.y, width: value.width, height: value.height }
}

export function validateCoverBlock(value: CoverBlock): string | undefined {
	const { x, y, width, height } = value
	if (![x, y, width, height].every(Number.isFinite)) {
		return 'Position et taille doivent être des nombres.'
	}
	if (width <= 0 || height <= 0) {
		return 'La largeur et la hauteur doivent être supérieures à 0.'
	}
	if (x < -EPSILON || y < -EPSILON) {
		return 'La position doit rester dans la page.'
	}
	if (
		x + width > PAGE_WIDTH_MM + EPSILON ||
		y + height > PAGE_HEIGHT_MM + EPSILON
	) {
		return 'Le bloc dépasse de la page (210 × 297 mm).'
	}
	return undefined
}

export function coverBlockErrors(
	layout: CoverLayout,
): Partial<Record<CoverBlockKey, string>> {
	const errors: Partial<Record<CoverBlockKey, string>> = {}
	for (const key of COVER_BLOCK_KEYS) {
		const message = validateCoverBlock(layout[key])
		if (message) errors[key] = message
	}
	return errors
}

export function validateCoverLayout(
	layout: CoverLayout | null | undefined,
): string | undefined {
	if (!layout) return undefined
	const errors = coverBlockErrors(layout)
	const key = COVER_BLOCK_KEYS.find((candidate) => errors[candidate])
	if (!key) return undefined
	return `${COVER_BLOCK_LABELS[key]} : ${errors[key]}`
}

export function setBlockRect(
	layout: CoverLayout,
	key: CoverBlockKey,
	rect: Rect,
): CoverLayout {
	return { ...layout, [key]: { ...layout[key], ...rect } }
}

export function setBlockVisible(
	layout: CoverLayout,
	key: CoverBlockKey,
	visible: boolean,
): CoverLayout {
	if (layout[key].visible === visible) return layout
	return { ...layout, [key]: { ...layout[key], visible } }
}

export function moveBlock(
	layout: CoverLayout,
	key: CoverBlockKey,
	dxMm: number,
	dyMm: number,
): CoverLayout {
	return setBlockRect(layout, key, moveRect(blockRect(layout[key]), dxMm, dyMm))
}

export function resizeBlock(
	layout: CoverLayout,
	key: CoverBlockKey,
	handle: ResizeHandle,
	dxMm: number,
	dyMm: number,
): CoverLayout {
	return setBlockRect(
		layout,
		key,
		resizeRect(blockRect(layout[key]), handle, dxMm, dyMm),
	)
}

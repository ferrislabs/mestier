import type { Schemas } from '#/api/api.client'

export type Decoration = Schemas.DoeDecoration
export type DecorationPages = NonNullable<Decoration['pages']>
export type DecorationLayer = NonNullable<Decoration['layer']>
export type DecorationKind = 'image' | 'rectangle' | 'line'

export interface Rect {
	x: number
	y: number
	width: number
	height: number
}

export type ResizeHandle = 'n' | 's' | 'e' | 'w' | 'ne' | 'nw' | 'se' | 'sw'

export const PAGE_WIDTH_MM = 210
export const PAGE_HEIGHT_MM = 297
export const MAX_DECORATIONS = 20
export const MIN_SIZE_MM = 1
export const DUPLICATE_OFFSET_MM = 5
export const KEY_STEP_MM = 1
export const KEY_STEP_SHIFT_MM = 10
export const LINE_THICKNESS_MM = 1
export const IMAGE_DEFAULT_WIDTH_MM = 80
const IMAGE_FALLBACK_HEIGHT_MM = 100

const EPSILON = 1e-6
const HEX_PATTERN = /^#[0-9a-f]{6}$/i

export const PAGES_LABELS: Record<DecorationPages, string> = {
	cover: 'Couverture',
	inner: 'Pages intérieures',
	all: 'Toutes les pages',
}

export const LAYER_LABELS: Record<DecorationLayer, string> = {
	behind: 'Derrière le contenu',
	front: 'Devant le contenu',
}

export const KIND_LABELS: Record<'image' | 'rectangle', string> = {
	image: 'Image',
	rectangle: 'Rectangle',
}

export function roundMm(value: number): number {
	return Math.round(value * 10) / 10
}

export function pagesOf(decoration: Decoration): DecorationPages {
	return decoration.pages ?? 'all'
}

export function layerOf(decoration: Decoration): DecorationLayer {
	return decoration.layer ?? 'behind'
}

export function rectOf(decoration: Decoration): Rect {
	return {
		x: decoration.x,
		y: decoration.y,
		width: decoration.width,
		height: decoration.height,
	}
}

export function isLine(decoration: Decoration): boolean {
	return (
		decoration.kind === 'rectangle' &&
		Math.min(decoration.width, decoration.height) <= LINE_THICKNESS_MM &&
		Math.max(decoration.width, decoration.height) > 10 * LINE_THICKNESS_MM
	)
}

export function decorationLabel(decoration: Decoration): string {
	if (decoration.kind === 'image') return 'Image'
	return isLine(decoration) ? 'Ligne' : 'Rectangle'
}

export function appliesToPage(
	decoration: Decoration,
	pageIndex: number,
): boolean {
	const pages = pagesOf(decoration)
	if (pages === 'all') return true
	return pageIndex === 0 ? pages === 'cover' : pages === 'inner'
}

export function createDecoration(
	kind: DecorationKind,
	color: string,
): Decoration {
	if (kind === 'image') {
		return {
			kind: 'image',
			key: '',
			fit: 'contain',
			x: 65,
			y: 98.5,
			width: 80,
			height: 100,
			pages: 'all',
			layer: 'behind',
		}
	}
	if (kind === 'line') {
		return {
			kind: 'rectangle',
			fill: color,
			x: 20,
			y: 148,
			width: 170,
			height: LINE_THICKNESS_MM,
			pages: 'all',
			layer: 'behind',
		}
	}
	return {
		kind: 'rectangle',
		fill: color,
		x: 65,
		y: 123.5,
		width: 80,
		height: 50,
		pages: 'all',
		layer: 'behind',
	}
}

export function createImageDecoration(
	key: string,
	aspectRatio: number,
): Decoration {
	const usable = Number.isFinite(aspectRatio) && aspectRatio > 0
	let width = IMAGE_DEFAULT_WIDTH_MM
	let height = usable ? width / aspectRatio : IMAGE_FALLBACK_HEIGHT_MM
	if (height > PAGE_HEIGHT_MM) {
		width = (width * PAGE_HEIGHT_MM) / height
		height = PAGE_HEIGHT_MM
	}
	width = Math.max(roundMm(width), MIN_SIZE_MM)
	height = Math.max(roundMm(height), MIN_SIZE_MM)
	return {
		kind: 'image',
		key,
		fit: 'contain',
		x: roundMm((PAGE_WIDTH_MM - width) / 2),
		y: roundMm((PAGE_HEIGHT_MM - height) / 2),
		width,
		height,
		pages: 'all',
		layer: 'behind',
	}
}

export function appendDecoration(
	list: Decoration[],
	decoration: Decoration,
): Decoration[] {
	if (list.length >= MAX_DECORATIONS) return list
	return [...list, decoration]
}

export function addDecoration(
	list: Decoration[],
	kind: DecorationKind,
	color: string,
): Decoration[] {
	if (list.length >= MAX_DECORATIONS) return list
	return [...list, createDecoration(kind, color)]
}

export function removeDecoration(
	list: Decoration[],
	index: number,
): Decoration[] {
	if (index < 0 || index >= list.length) return list
	return list.filter((_, position) => position !== index)
}

export function updateDecoration(
	list: Decoration[],
	index: number,
	next: Decoration,
): Decoration[] {
	return list.map((item, position) => (position === index ? next : item))
}

export function duplicateDecoration(
	list: Decoration[],
	index: number,
): Decoration[] {
	const source = list[index]
	if (!source || list.length >= MAX_DECORATIONS) return list
	const moved = moveRect(
		rectOf(source),
		DUPLICATE_OFFSET_MM,
		DUPLICATE_OFFSET_MM,
	)
	const copy: Decoration = { ...source, ...moved }
	return [...list.slice(0, index + 1), copy, ...list.slice(index + 1)]
}

export function reorderDecoration(
	list: Decoration[],
	index: number,
	direction: 'forward' | 'backward',
): Decoration[] {
	const target = direction === 'forward' ? index + 1 : index - 1
	if (index < 0 || index >= list.length) return list
	if (target < 0 || target >= list.length) return list
	const next = [...list]
	const current = next[index] as Decoration
	next[index] = next[target] as Decoration
	next[target] = current
	return next
}

export function reorderedIndex(
	list: Decoration[],
	index: number,
	direction: 'forward' | 'backward',
): number {
	const target = direction === 'forward' ? index + 1 : index - 1
	return target < 0 || target >= list.length ? index : target
}

export function mmPerPixel(displayedWidthPx: number): number {
	return displayedWidthPx > 0 ? PAGE_WIDTH_MM / displayedWidthPx : 0
}

export function pxToMm(px: number, displayedWidthPx: number): number {
	return px * mmPerPixel(displayedWidthPx)
}

export function mmToPx(mm: number, displayedWidthPx: number): number {
	return (mm * displayedWidthPx) / PAGE_WIDTH_MM
}

function clamp(value: number, min: number, max: number): number {
	return Math.min(Math.max(value, min), max)
}

export function clampRect(rect: Rect): Rect {
	const width = clamp(roundMm(rect.width), MIN_SIZE_MM, PAGE_WIDTH_MM)
	const height = clamp(roundMm(rect.height), MIN_SIZE_MM, PAGE_HEIGHT_MM)
	return {
		width,
		height,
		x: roundMm(clamp(rect.x, 0, PAGE_WIDTH_MM - width)),
		y: roundMm(clamp(rect.y, 0, PAGE_HEIGHT_MM - height)),
	}
}

export function moveRect(rect: Rect, dxMm: number, dyMm: number): Rect {
	return clampRect({ ...rect, x: rect.x + dxMm, y: rect.y + dyMm })
}

export function resizeRect(
	rect: Rect,
	handle: ResizeHandle,
	dxMm: number,
	dyMm: number,
): Rect {
	let left = rect.x
	let top = rect.y
	let right = rect.x + rect.width
	let bottom = rect.y + rect.height
	if (handle.includes('w')) {
		left = clamp(rect.x + dxMm, 0, right - MIN_SIZE_MM)
	}
	if (handle.includes('e')) {
		right = clamp(right + dxMm, left + MIN_SIZE_MM, PAGE_WIDTH_MM)
	}
	if (handle.includes('n')) {
		top = clamp(rect.y + dyMm, 0, bottom - MIN_SIZE_MM)
	}
	if (handle.includes('s')) {
		bottom = clamp(bottom + dyMm, top + MIN_SIZE_MM, PAGE_HEIGHT_MM)
	}
	return clampRect({
		x: left,
		y: top,
		width: right - left,
		height: bottom - top,
	})
}

export function keyStep(shift: boolean): number {
	return shift ? KEY_STEP_SHIFT_MM : KEY_STEP_MM
}

export function nudgeRect(rect: Rect, key: string, shift: boolean): Rect {
	const step = keyStep(shift)
	switch (key) {
		case 'ArrowLeft':
			return moveRect(rect, -step, 0)
		case 'ArrowRight':
			return moveRect(rect, step, 0)
		case 'ArrowUp':
			return moveRect(rect, 0, -step)
		case 'ArrowDown':
			return moveRect(rect, 0, step)
		default:
			return rect
	}
}

function inRange(value: number, min: number, max: number): boolean {
	return (
		Number.isFinite(value) && value >= min - EPSILON && value <= max + EPSILON
	)
}

export function validateDecoration(decoration: Decoration): string | undefined {
	const { x, y, width, height } = decoration
	if (![x, y, width, height].every(Number.isFinite)) {
		return 'Position et taille doivent être des nombres.'
	}
	if (width <= 0 || height <= 0) {
		return 'La largeur et la hauteur doivent être supérieures à 0.'
	}
	if (!inRange(x, 0, PAGE_WIDTH_MM) || !inRange(y, 0, PAGE_HEIGHT_MM)) {
		return 'La position doit rester dans la page.'
	}
	if (
		x + width > PAGE_WIDTH_MM + EPSILON ||
		y + height > PAGE_HEIGHT_MM + EPSILON
	) {
		return 'L’élément dépasse de la page (210 × 297 mm).'
	}
	if (decoration.opacity !== undefined && !inRange(decoration.opacity, 0, 1)) {
		return 'L’opacité doit être comprise entre 0 et 1.'
	}
	if (decoration.kind === 'image') {
		if (decoration.key.trim() === '') return 'Choisissez une image.'
		return undefined
	}
	if (!HEX_PATTERN.test(decoration.fill.trim())) {
		return 'Couleur attendue au format #rrggbb.'
	}
	if (
		decoration.radius !== undefined &&
		!inRange(decoration.radius, 0, PAGE_HEIGHT_MM)
	) {
		return 'Le rayon doit être compris entre 0 et 297 mm.'
	}
	return undefined
}

export function decorationErrors(
	list: Decoration[],
): Array<string | undefined> {
	return list.map(validateDecoration)
}

export function validateDecorations(list: Decoration[]): string | undefined {
	if (list.length > MAX_DECORATIONS) {
		return `${MAX_DECORATIONS} éléments au plus.`
	}
	const index = decorationErrors(list).findIndex(Boolean)
	if (index === -1) return undefined
	return `Élément ${index + 1} : ${validateDecoration(list[index] as Decoration)}`
}

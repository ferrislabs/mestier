import type { Schemas } from '#/api/api.client'

export type DoeStyle = Schemas.DoeStyle
export type DoeFont = NonNullable<DoeStyle['heading_font']>
export type LogoPosition = NonNullable<DoeStyle['cover_logo_position']>
export type LogoSize = NonNullable<DoeStyle['cover_logo_size']>

export type CompleteDoeStyle = Required<Omit<DoeStyle, 'logo_key'>> & {
	logo_key: string | null
}

export type StyleErrors = Partial<Record<keyof CompleteDoeStyle, string>>

export const DEFAULT_DOE_STYLE: CompleteDoeStyle = {
	primary: '#0f3d36',
	secondary: '#0f3d36',
	on_primary: '#ffffff',
	heading_font: 'inter',
	body_font: 'inter',
	logo_key: null,
	cover_logo_position: 'left',
	cover_logo_size: 'medium',
	cover_photo: true,
	header_logo: false,
	header_text: 'DOE – {titre}',
	footer_text: '{date}',
	cover_title: 'DOSSIER DES OUVRAGES EXÉCUTÉS (D.O.E.)',
	parties_title: 'INTERVENANTS',
	contents_title: 'SOMMAIRE',
	supply_table_title: 'Identification de la fourniture',
}

export const FONT_LABELS: Record<DoeFont, string> = {
	inter: 'Inter',
	montserrat: 'Montserrat',
	source_serif: 'Source Serif',
	barlow: 'Barlow',
}

export const FONTS: DoeFont[] = [
	'inter',
	'montserrat',
	'source_serif',
	'barlow',
]

export const TEMPLATE_VARIABLES = [
	'{titre}',
	'{lieu}',
	'{lot}',
	'{indice}',
	'{date}',
]

export const MAX_LABEL_LENGTH = 120
export const MAX_TEMPLATE_LENGTH = 200

const HEX_PATTERN = /^#[0-9a-f]{6}$/i
const VARIABLE_PATTERN = /\{([^{}]*)\}/g

export function isValidHex(value: string): boolean {
	return HEX_PATTERN.test(value.trim())
}

export function completeStyle(
	partial: DoeStyle | null | undefined,
): CompleteDoeStyle {
	const merged: CompleteDoeStyle = { ...DEFAULT_DOE_STYLE }
	if (!partial) return merged
	const target = merged as Record<string, unknown>
	for (const [key, value] of Object.entries(partial)) {
		if (value !== undefined) target[key] = value
	}
	return merged
}

function colorError(value: string): string | undefined {
	return isValidHex(value)
		? undefined
		: 'Format attendu : #rrggbb, par exemple #0f3d36.'
}

function labelError(value: string): string | undefined {
	if (value.trim() === '') return 'Ce libellé ne peut pas être vide.'
	if ([...value].length > MAX_LABEL_LENGTH) {
		return `${MAX_LABEL_LENGTH} caractères au plus.`
	}
	return undefined
}

function templateError(value: string): string | undefined {
	if ([...value].length > MAX_TEMPLATE_LENGTH) {
		return `${MAX_TEMPLATE_LENGTH} caractères au plus.`
	}
	for (const match of value.matchAll(VARIABLE_PATTERN)) {
		if (!TEMPLATE_VARIABLES.includes(match[0])) {
			return `Variable inconnue : ${match[0]}. Variables permises : ${TEMPLATE_VARIABLES.join(', ')}.`
		}
	}
	return undefined
}

export function validateStyle(style: CompleteDoeStyle): StyleErrors {
	const candidates: StyleErrors = {
		primary: colorError(style.primary),
		secondary: colorError(style.secondary),
		on_primary: colorError(style.on_primary),
		header_text: templateError(style.header_text),
		footer_text: templateError(style.footer_text),
		cover_title: labelError(style.cover_title),
		parties_title: labelError(style.parties_title),
		contents_title: labelError(style.contents_title),
		supply_table_title: labelError(style.supply_table_title),
	}
	const errors: StyleErrors = {}
	for (const [key, message] of Object.entries(candidates)) {
		if (message) errors[key as keyof CompleteDoeStyle] = message
	}
	return errors
}

export function isStyleValid(style: CompleteDoeStyle): boolean {
	return Object.keys(validateStyle(style)).length === 0
}

export function normalizeStyle(style: CompleteDoeStyle): CompleteDoeStyle {
	return {
		...style,
		primary: style.primary.trim().toLowerCase(),
		secondary: style.secondary.trim().toLowerCase(),
		on_primary: style.on_primary.trim().toLowerCase(),
		cover_title: style.cover_title.trim(),
		parties_title: style.parties_title.trim(),
		contents_title: style.contents_title.trim(),
		supply_table_title: style.supply_table_title.trim(),
	}
}

export function sameStyle(a: CompleteDoeStyle, b: CompleteDoeStyle): boolean {
	return JSON.stringify(normalizeStyle(a)) === JSON.stringify(normalizeStyle(b))
}

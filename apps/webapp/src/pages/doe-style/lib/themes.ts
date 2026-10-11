import type { CompleteDoeStyle } from '#/pages/doe-style/lib/style'

export type ThemeLook = Pick<
	CompleteDoeStyle,
	| 'primary'
	| 'secondary'
	| 'on_primary'
	| 'heading_font'
	| 'body_font'
	| 'cover_logo_position'
	| 'cover_logo_size'
	| 'cover_photo'
>

export interface DoeTheme {
	id: string
	name: string
	description: string
	look: ThemeLook
}

const BASE = {
	on_primary: '#ffffff',
	cover_logo_position: 'left',
	cover_logo_size: 'medium',
	cover_photo: true,
} as const

export const DOE_THEMES: DoeTheme[] = [
	{
		id: 'sobre',
		name: 'Sobre',
		description: 'Vert profond et Inter, le rendu par défaut de Mestier.',
		look: {
			...BASE,
			primary: '#0f3d36',
			secondary: '#0f3d36',
			heading_font: 'inter',
			body_font: 'inter',
		},
	},
	{
		id: 'nature',
		name: 'Nature',
		description: 'Verts végétaux, titres Montserrat et texte à empattements.',
		look: {
			...BASE,
			primary: '#2f5d3a',
			secondary: '#6b8e23',
			heading_font: 'montserrat',
			body_font: 'source_serif',
		},
	},
	{
		id: 'terre',
		name: 'Terre',
		description: 'Terre cuite et ocre, en Barlow.',
		look: {
			...BASE,
			primary: '#8b4a2b',
			secondary: '#c47a3a',
			heading_font: 'barlow',
			body_font: 'barlow',
		},
	},
	{
		id: 'corporate',
		name: 'Corporate',
		description: 'Bleus institutionnels, logo centré.',
		look: {
			...BASE,
			primary: '#1f3a5f',
			secondary: '#3b6ea5',
			heading_font: 'inter',
			body_font: 'inter',
			cover_logo_position: 'center',
		},
	},
	{
		id: 'elegant',
		name: 'Élégant',
		description: 'Anthracite et or ancien, tout en Source Serif.',
		look: {
			...BASE,
			primary: '#2b2b2b',
			secondary: '#8a6d3b',
			on_primary: '#f5efe1',
			heading_font: 'source_serif',
			body_font: 'source_serif',
		},
	},
]

export function applyTheme(
	style: CompleteDoeStyle,
	theme: DoeTheme,
): CompleteDoeStyle {
	return { ...style, ...theme.look }
}

export function activeThemeId(style: CompleteDoeStyle): string | null {
	const found = DOE_THEMES.find((theme) =>
		(Object.keys(theme.look) as Array<keyof ThemeLook>).every((key) => {
			const current = style[key]
			return typeof current === 'string'
				? current.trim().toLowerCase() === theme.look[key]
				: current === theme.look[key]
		}),
	)
	return found?.id ?? null
}

export const ENTRY = {
	projects: 'projects',
	models: 'models',
	overview: 'overview',
	missions: 'missions',
	doe: 'doe',
	settings: 'settings',
} as const

export type ProjectSection = 'overview' | 'missions' | 'doe' | 'settings'

const NOT_A_PROJECT = new Set(['new', 'doe-models'])

const SECTION_SEGMENT: Record<ProjectSection, string> = {
	overview: '',
	missions: '/missions',
	doe: '/doe',
	settings: '/settings',
}

export function projectIdOf(modulePath: string): string | null {
	const match = /^\/projet\/([^/]+)/.exec(modulePath)
	if (!match?.[1] || NOT_A_PROJECT.has(match[1])) return null
	return decodeURIComponent(match[1])
}

export function projectSectionOf(modulePath: string): ProjectSection | null {
	if (!projectIdOf(modulePath)) return null
	const segment = /^\/projet\/[^/]+\/([^/]+)/.exec(modulePath)?.[1]
	switch (segment) {
		case 'missions':
			return 'missions'
		case 'doe':
			return 'doe'
		case 'settings':
			return 'settings'
		default:
			return 'overview'
	}
}

export function activeSidebarEntries(modulePath: string): ReadonlySet<string> {
	if (modulePath.startsWith('/projet/doe-models')) {
		return new Set([ENTRY.models])
	}
	const section = projectSectionOf(modulePath)
	return new Set([section ? ENTRY[section] : ENTRY.projects])
}

export function switchTarget(
	projectId: string,
	section: ProjectSection,
): string {
	return `/projet/${encodeURIComponent(projectId)}${SECTION_SEGMENT[section]}`
}

export function doeModelIdOf(modulePath: string): string | null {
	const match = /^\/projet\/doe-models\/([^/]+)/.exec(modulePath)
	return match?.[1] ? decodeURIComponent(match[1]) : null
}

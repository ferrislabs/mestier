const RULES: readonly (readonly [legacy: string, current: string])[] = [
	['team', 'membres'],
	['work-time', 'temps-de-travail'],
	['equipment', 'materiel'],
	['rights', 'droits'],
	['absences/new', 'absences/nouvelle'],
	['extra/new', 'extra/nouvelle'],
	['absences', 'absences'],
	['extra', 'extra'],
	['planning', 'planning'],
]

const SEARCH_KEYS: Readonly<Record<string, string>> = {
	tab: 'onglet',
	status: 'statut',
	member: 'membre',
	account: 'compte',
	view: 'vue',
	month: 'mois',
}

const MEMBER_WORK_TIME = /\/work-time$/

/** Current path of a pre-rework HR URL; unknown splats land on the module root. */
export function legacyHrPath(splat: string): string {
	for (const [legacy, current] of RULES) {
		if (splat === legacy || splat.startsWith(`${legacy}/`)) {
			const rest = splat.slice(legacy.length)
			return `/rh/${current}${legacy === 'team' ? rest.replace(MEMBER_WORK_TIME, '/temps-de-travail') : rest}`
		}
	}
	return '/rh'
}

/** The legacy search keys renamed to the French ones. */
export function legacyHrSearch(
	search: Readonly<Record<string, unknown>>,
): Record<string, unknown> {
	return Object.fromEntries(
		Object.entries(search).map(([key, value]) => [
			SEARCH_KEYS[key] ?? key,
			value,
		]),
	)
}

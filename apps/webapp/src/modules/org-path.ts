/** Segment qui porte le tenant dans l'URL : `/o/<slug>/…`. */
export const ORG_PATH_SEGMENT = '/o'

/** Gabarit de route TanStack correspondant au préfixe d'organisation. */
export const ORG_ROUTE_PREFIX = `${ORG_PATH_SEGMENT}/$organizationSlug`

/**
 * Préfixe un chemin relatif à l'organisation.
 *
 * Les chemins du registre de modules restent relatifs (`/crm/customers`) : le
 * tenant n'est ajouté qu'au moment de construire un lien, pour qu'un module
 * n'ait jamais à connaître la forme du préfixe.
 */
export function buildOrgPath(organizationSlug: string, to: string): string {
	const base = `${ORG_PATH_SEGMENT}/${organizationSlug}`
	if (to === '/') return base

	return `${base}${to}`
}

interface OrgPathParts {
	organizationSlug: string | null
	/** Chemin relatif à l'organisation, toujours préfixé par `/`. */
	path: string
}

/**
 * Sépare le tenant du reste du chemin. Sans préfixe d'organisation, le chemin
 * est renvoyé tel quel — c'est le cas des routes hors organisation.
 */
export function splitOrgPath(pathname: string): OrgPathParts {
	if (!pathname.startsWith(`${ORG_PATH_SEGMENT}/`)) {
		return { organizationSlug: null, path: pathname }
	}

	const rest = pathname.slice(ORG_PATH_SEGMENT.length + 1)
	const separator = rest.indexOf('/')
	if (separator === -1) {
		return { organizationSlug: decodeURIComponent(rest), path: '/' }
	}

	return {
		organizationSlug: decodeURIComponent(rest.slice(0, separator)),
		path: rest.slice(separator) || '/',
	}
}

/** Segment carrying the tenant in the URL: `/o/<slug>/…`. */
export const ORG_PATH_SEGMENT = '/o'

/** TanStack route template matching the organization prefix. */
export const ORG_ROUTE_PREFIX = `${ORG_PATH_SEGMENT}/$organizationSlug`

/**
 * Prefixes a path relative to the organization.
 *
 * Module registry paths stay relative (`/crm/customers`): the tenant is only
 * added when building a link, so a module never has to know the prefix's shape.
 */
export function buildOrgPath(organizationSlug: string, to: string): string {
	const base = `${ORG_PATH_SEGMENT}/${organizationSlug}`
	if (to === '/') return base

	return `${base}${to}`
}

interface OrgPathParts {
	organizationSlug: string | null
	/** Path relative to the organization, always prefixed with `/`. */
	path: string
}

/**
 * Splits the tenant off the rest of the path. With no organization prefix the
 * path is returned as is — the case for routes outside any organization.
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

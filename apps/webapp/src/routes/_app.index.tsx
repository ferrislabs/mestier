import { createFileRoute, Navigate } from '@tanstack/react-router'

import {
	readLastOrganizationSlug,
	useOrganizationList,
} from '#/hooks/use-active-organization'
import { buildOrgPath } from '#/modules/org-path'

export const Route = createFileRoute('/_app/')({
	component: OrganizationEntry,
})

/**
 * Tenant-less entry point: we resume on the last visited organization, or the
 * first one. `OrgGate` guarantees there is at least one.
 */
function OrganizationEntry() {
	const organizations = useOrganizationList()
	const lastSlug = readLastOrganizationSlug()
	const target =
		organizations.find((organization) => organization.slug === lastSlug) ??
		organizations[0]

	if (!target) return null

	return <Navigate to={buildOrgPath(target.slug, '/')} replace />
}

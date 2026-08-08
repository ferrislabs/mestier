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
 * Point d'entrée sans tenant : on repart sur la dernière organisation visitée,
 * à défaut la première. `OrgGate` garantit qu'il y en a au moins une.
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

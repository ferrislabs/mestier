import { useLocation, useNavigate } from '@tanstack/react-router'
import { useMemo } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers } from '#/hooks/use-customers'
import { useProjects } from '#/hooks/use-projects'
import { buildOrgPath, splitOrgPath } from '#/modules/org-path'
import {
	projectIdOf,
	projectSectionOf,
	switchTarget,
} from '#/pages/projet/lib/project-path'
import { switcherOptions } from '#/pages/projet/lib/switcher-options'
import { ProjetCrumbs } from '#/pages/projet/ui/projet-crumbs'

export function ProjetCrumbsFeature() {
	const { activeOrganization } = useActiveOrganization()
	const navigate = useNavigate()
	const pathname = useLocation({ select: (location) => location.pathname })
	const modulePath = splitOrgPath(pathname).path
	const projectId = projectIdOf(modulePath)
	const section = projectSectionOf(modulePath)

	const projects = useProjects(activeOrganization.id, {
		includeArchived: true,
	})
	const customers = useCustomers(activeOrganization.id)

	const options = useMemo(
		() =>
			switcherOptions(projects.data?.data ?? [], customers.data?.data ?? []),
		[projects.data, customers.data],
	)

	if (!projectId || !section) return null

	const current = options.find((option) => option.id === projectId)
	const currentName =
		current?.name ?? (projects.isLoading ? 'Chargement…' : 'Projet introuvable')

	return (
		<ProjetCrumbs
			organizationSlug={activeOrganization.slug}
			section={section}
			onPreview={modulePath.endsWith('/doe/preview')}
			currentId={projectId}
			currentName={currentName}
			currentOwnerName={current?.ownerName ?? null}
			options={options}
			onSelect={(nextId) =>
				navigate({
					to: buildOrgPath(
						activeOrganization.slug,
						switchTarget(nextId, section),
					),
				})
			}
		/>
	)
}

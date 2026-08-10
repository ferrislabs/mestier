import { buildOrgPath, splitOrgPath } from '#/modules/org-path'
import { resolveModule } from '#/modules/resolve-module'
import { resolveSection } from '#/modules/scope'
import type { NavTarget } from '#/modules/types'

export interface BreadcrumbItemDefinition {
	id: string
	label: string
	to?: string
}

interface BuildBreadcrumbInput {
	pathname: string
	organizationName: string
	organizationSlug: string
	detailLabel?: string
}

export function buildBreadcrumbItems({
	pathname,
	organizationName,
	organizationSlug,
	detailLabel,
}: BuildBreadcrumbInput): BreadcrumbItemDefinition[] {
	const modulePath = splitOrgPath(pathname).path
	const activeModule = resolveModule(modulePath)
	const items: BreadcrumbItemDefinition[] = [
		{
			id: 'organization',
			label: organizationName,
			to: buildOrgPath(organizationSlug, '/'),
		},
	]

	if (activeModule.basePath !== '/') {
		items.push({
			id: `module-${activeModule.id}`,
			label: activeModule.label,
			to: buildOrgPath(organizationSlug, activeModule.basePath),
		})
	}

	for (const section of matchingTargets(
		activeModule.sections,
		activeModule.basePath,
		modulePath,
	)) {
		items.push({
			id: `section-${section.id}`,
			label: section.label,
			to: buildOrgPath(organizationSlug, section.to),
		})
	}

	const activeSection = resolveSection(activeModule, modulePath)
	for (const tab of matchingTargets(
		activeSection?.tabs ?? [],
		activeSection?.to ?? '',
		modulePath,
	)) {
		items.push({
			id: `tab-${tab.id}`,
			label: tab.label,
			to: buildOrgPath(organizationSlug, tab.to),
		})
	}

	if (detailLabel) {
		items.push({ id: 'detail', label: detailLabel })
	}

	return items
}

/**
 * Navigation targets whose path is a prefix of the current one, from the most
 * general to the most precise. A target that doubles its parent is dropped, so
 * the same link never appears twice.
 */
export function matchingTargets<T extends NavTarget>(
	targets: T[],
	parentPath: string,
	pathname: string,
): T[] {
	return targets
		.filter(
			(target) => target.status !== 'coming-soon' && target.to !== parentPath,
		)
		.filter(
			(target) =>
				pathname === target.to || pathname.startsWith(`${target.to}/`),
		)
		.sort((a, b) => a.to.length - b.to.length)
}

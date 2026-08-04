import type { NavItem } from '#/components/nav-main'
import { buildSidebarGroups } from '#/modules/nav-groups'
import { resolveModule } from '#/modules/resolve-module'

export interface BreadcrumbItemDefinition {
	id: string
	label: string
	to?: string
}

interface BuildBreadcrumbInput {
	pathname: string
	organizationName: string
	detailLabel?: string
}

export function buildBreadcrumbItems({
	pathname,
	organizationName,
	detailLabel,
}: BuildBreadcrumbInput): BreadcrumbItemDefinition[] {
	const activeModule = resolveModule(pathname)
	const items: BreadcrumbItemDefinition[] = [
		{ id: 'organization', label: organizationName, to: '/' },
	]

	if (activeModule.basePath !== '/') {
		items.push({
			id: `module-${activeModule.id}`,
			label: activeModule.label,
			to: activeModule.basePath,
		})
	}

	const navItems = buildSidebarGroups(pathname).flatMap((group) => group.items)
	for (const item of matchingNavItems(
		navItems,
		activeModule.basePath,
		pathname,
	)) {
		items.push({ id: `nav-${item.to}`, label: item.title, to: item.to })
	}

	if (detailLabel) {
		items.push({ id: 'detail', label: detailLabel })
	}

	return items
}

export function matchingNavItems(
	items: NavItem[],
	basePath: string,
	pathname: string,
): NavItem[] {
	return items
		.filter((item) => !item.disabled && item.to !== basePath)
		.filter(
			(item) => pathname === item.to || pathname.startsWith(`${item.to}/`),
		)
		.sort((a, b) => a.to.length - b.to.length)
}

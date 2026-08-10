import { moduleLandingPath } from '#/modules/landing'
import { splitOrgPath } from '#/modules/org-path'
import { resolveModule } from '#/modules/resolve-module'
import type { AppModule, ModuleSection, ModuleTab } from '#/modules/types'

export interface Scope {
	/** Label of the active scope: the section when there is one, else the module. */
	label: string
	/** Second level of the active scope, rendered as horizontal tabs. */
	tabs: ModuleTab[]
}

/**
 * Side-nav section covering a given path: the deepest one whose path is a
 * prefix of the current path.
 */
export function resolveSection(
	module: AppModule,
	path: string,
): ModuleSection | undefined {
	let best: ModuleSection | undefined

	for (const section of module.sections) {
		if (section.status === 'coming-soon') continue
		if (path !== section.to && !path.startsWith(`${section.to}/`)) continue
		if (!best || section.to.length > best.to.length) best = section
	}

	return best
}

export function resolveScope(pathname: string): Scope {
	const path = splitOrgPath(pathname).path
	const module = resolveModule(path)
	const section = resolveSection(module, path)

	return {
		label: section?.label ?? module.label,
		tabs: section?.tabs ?? [],
	}
}

/**
 * A path's equivalent in another organization.
 *
 * A list screen transposes as is; an entity screen does not — a customer's id
 * does not exist in the target organization. We then climb back to the deepest
 * section that stays valid.
 */
export function crossOrganizationPath(path: string): string {
	const module = resolveModule(path)
	const section = resolveSection(module, path)

	if (!section) {
		return module.hasOverview ? module.basePath : moduleLandingPath(module.id)
	}

	return path === section.to ? path : section.to
}

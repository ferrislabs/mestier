import { MODULES } from '#/modules/registry'
import type { ModuleId, ModuleNavGroup } from '#/modules/types'

export function firstLandingTarget(
	nav: ModuleNavGroup[],
	basePath: string,
): string | undefined {
	const item = nav
		.flatMap((group) => group.items)
		.find((item) => !item.disabled && item.to !== basePath)

	return item?.to
}

export function moduleLandingPath(moduleId: ModuleId): string {
	const module = MODULES.find((module) => module.id === moduleId)
	if (!module) throw new Error(`registry: module ${moduleId} manquant`)

	return firstLandingTarget(module.nav, module.basePath) ?? module.basePath
}

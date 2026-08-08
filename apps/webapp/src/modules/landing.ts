import { MODULES } from '#/modules/registry'
import type { ModuleId, ModuleSection } from '#/modules/types'

export function firstLandingTarget(
	sections: ModuleSection[],
	basePath: string,
): string | undefined {
	const section = sections.find(
		(section) => section.status !== 'coming-soon' && section.to !== basePath,
	)

	return section?.to
}

export function moduleLandingPath(moduleId: ModuleId): string {
	const module = MODULES.find((module) => module.id === moduleId)
	if (!module) throw new Error(`registry: module ${moduleId} manquant`)
	if (module.hasOverview) return module.basePath

	return firstLandingTarget(module.sections, module.basePath) ?? module.basePath
}

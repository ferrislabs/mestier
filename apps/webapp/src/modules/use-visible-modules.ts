import { usePermissions } from '#/hooks/use-permissions'
import { MODULES } from '#/modules/registry'
import type { AppModule } from '#/modules/types'
import { useSectionVisibility } from '#/modules/use-section-visibility'

function isGated(module: AppModule): boolean {
	return (
		module.sections.length > 0 &&
		module.sections.every(
			(section) => section.requiredPermission || section.visibleWhen,
		)
	)
}

export function useVisibleModules(): AppModule[] {
	const { data, isSuccess } = usePermissions()
	const isSectionVisible = useSectionVisibility()
	const granted = data?.data.permissions ?? []

	return MODULES.filter(
		(module) =>
			module.status !== 'hidden' &&
			(!module.requiredPermission ||
				(isSuccess && granted.includes(module.requiredPermission))) &&
			(!isGated(module) || module.sections.some(isSectionVisible)),
	)
}

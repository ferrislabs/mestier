import { useFeatures } from '#/hooks/use-features'
import { usePermissions } from '#/hooks/use-permissions'
import { isFeatureVisible } from '#/modules/feature-gate'
import { MODULES } from '#/modules/registry'
import type { AppModule } from '#/modules/types'

export function useVisibleModules(): AppModule[] {
	const { data, isSuccess } = usePermissions()
	const granted = data?.data.permissions ?? []
	const { features, loaded } = useFeatures()

	return MODULES.filter(
		(module) =>
			module.status !== 'hidden' &&
			(!module.requiredPermission ||
				(isSuccess && granted.includes(module.requiredPermission))) &&
			isFeatureVisible(module, features, loaded),
	)
}

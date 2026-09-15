import { usePermissions } from '#/hooks/use-permissions'
import { MODULES } from '#/modules/registry'
import type { AppModule } from '#/modules/types'

export function useVisibleModules(): AppModule[] {
	const { data, isSuccess } = usePermissions()
	const granted = data?.data.permissions ?? []

	return MODULES.filter(
		(module) =>
			module.status !== 'hidden' &&
			(!module.requiredPermission ||
				(isSuccess && granted.includes(module.requiredPermission))),
	)
}

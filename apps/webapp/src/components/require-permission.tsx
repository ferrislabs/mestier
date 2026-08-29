import type { ReactNode } from 'react'
import type { PermissionName } from '#/hooks/use-permissions'
import { useHasPermission } from '#/hooks/use-permissions'

interface RequirePermissionProps {
	permission: PermissionName
	children: ReactNode
}

/**
 * Hides `children` outright when the caller does not hold `permission` in
 * the active organization — a write control the caller cannot use is
 * absent, not disabled-and-mysterious (#307).
 *
 * Presentation only: the API refuses regardless of whether this hides the
 * control, see `hooks/use-permissions.ts`'s own doc comment. A permission
 * can still be revoked between load and click, so this never replaces the
 * 403 handling on the mutation itself.
 */
export function RequirePermission({
	permission,
	children,
}: RequirePermissionProps) {
	const hasPermission = useHasPermission(permission)
	if (!hasPermission) return null
	return <>{children}</>
}

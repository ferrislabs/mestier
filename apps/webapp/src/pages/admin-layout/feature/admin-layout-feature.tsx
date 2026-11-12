import { Navigate, Outlet } from '@tanstack/react-router'
import { useAdminAccess } from '#/hooks/use-admin'
import { adminGuard } from '#/pages/admin-layout/lib/admin-sections'
import { AdminShell } from '#/pages/admin-layout/ui/admin-shell'

export function AdminLayoutFeature() {
	const access = useAdminAccess()
	const guard = adminGuard(access.isPending, access.data?.data.can_access)

	if (guard === 'loading') return null
	if (guard === 'denied') return <Navigate to="/" replace />

	return (
		<AdminShell>
			<Outlet />
		</AdminShell>
	)
}

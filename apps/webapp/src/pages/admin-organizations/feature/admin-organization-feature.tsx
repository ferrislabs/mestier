import {
	useAdminOrganization,
	useSetOrganizationFeature,
} from '#/hooks/use-admin'
import { mutationErrorMessage } from '#/lib/api-error'
import { AdminOrganization } from '#/pages/admin-organizations/ui/admin-organization'

interface AdminOrganizationFeatureProps {
	organizationId: string
}

export function AdminOrganizationFeature({
	organizationId,
}: AdminOrganizationFeatureProps) {
	const organization = useAdminOrganization(organizationId)
	const setFeature = useSetOrganizationFeature()

	return (
		<AdminOrganization
			organization={organization.data?.data}
			loading={organization.isPending}
			error={
				mutationErrorMessage(organization.error) ??
				mutationErrorMessage(setFeature.error)
			}
			pendingFlag={
				setFeature.isPending ? (setFeature.variables?.path.flag ?? null) : null
			}
			onToggle={(flag, enabled) =>
				setFeature.mutate({
					path: { organization_id: organizationId, flag },
					body: { enabled },
				})
			}
		/>
	)
}

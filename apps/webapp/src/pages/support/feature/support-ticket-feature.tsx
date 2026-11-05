import { useActiveOrganization } from '#/hooks/use-active-organization'
import { SupportTicketDetailFeature } from '#/pages/support/feature/support-ticket-detail-feature'
import { SupportOrganizationUnavailable } from '#/pages/support/feature/use-support-screen'
import { useSupportTicketVM } from '#/pages/support/feature/use-support-ticket'
import type { SupportTicketScreenProps } from '#/pages/support/types'
import { SupportTicketPageUI } from '#/pages/support/ui/support-ticket-page-ui'

export function SupportTicketFeature({
	ticketId,
	onBack,
}: SupportTicketScreenProps) {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) return <SupportOrganizationUnavailable />

	return (
		<SupportTicketContent
			key={`${activeOrganization.id}:${ticketId}`}
			organizationId={activeOrganization.id}
			ticketId={ticketId}
			onBack={onBack}
		/>
	)
}

interface SupportTicketContentProps extends SupportTicketScreenProps {
	organizationId: string
}

function SupportTicketContent({
	organizationId,
	ticketId,
	onBack,
}: SupportTicketContentProps) {
	const { query, vm } = useSupportTicketVM(organizationId, ticketId)

	return (
		<SupportTicketPageUI
			ticket={vm}
			isLoading={query.isLoading}
			error={vm ? null : (query.error?.message ?? null)}
			onRetry={() => void query.refetch()}
			onBack={onBack}
			detail={
				<SupportTicketDetailFeature
					organizationId={organizationId}
					ticketId={ticketId}
					variant="page"
				/>
			}
		/>
	)
}

import { createFileRoute } from '@tanstack/react-router'
import { SupportTicketFeature } from '#/pages/support/feature/support-ticket-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/support/tickets/$ticketId',
)({
	component: SupportTicketPage,
})

function SupportTicketPage() {
	const { organizationSlug, ticketId } = Route.useParams()
	const navigate = Route.useNavigate()

	return (
		<SupportTicketFeature
			ticketId={ticketId}
			onBack={() =>
				void navigate({
					to: '/o/$organizationSlug/support',
					params: { organizationSlug },
				})
			}
		/>
	)
}

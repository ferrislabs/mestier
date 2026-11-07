import { createFileRoute } from '@tanstack/react-router'
import { AutomationCredentialsFeature } from '#/pages/automation/feature/automation-credentials-feature'
import { parseIdentificationsSearch } from '#/pages/automation/lib/oauth-return'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/identifications',
)({
	validateSearch: parseIdentificationsSearch,
	component: IdentificationsPage,
})

function IdentificationsPage() {
	const { oauth } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<AutomationCredentialsFeature
			oauthOutcome={oauth}
			onOauthOutcomeShown={() => void navigate({ search: {}, replace: true })}
		/>
	)
}

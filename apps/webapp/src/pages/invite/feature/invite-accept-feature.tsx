import { useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { useEffect, useRef, useState } from 'react'
import { useAcceptInvitation } from '#/hooks/use-invitations'
import { buildOrgPath } from '#/modules/org-path'
import { InviteAcceptUI } from '#/pages/invite/ui/invite-accept-ui'

const MY_ORGS_PATH = '/api/v1/users/@me/organizations'

interface InviteAcceptFeatureProps {
	token: string
}

export function InviteAcceptFeature({ token }: InviteAcceptFeatureProps) {
	const navigate = useNavigate()
	const queryClient = useQueryClient()
	const acceptInvitation = useAcceptInvitation()
	// Guards the mutation to exactly one call for this component's lifetime.
	// The backend's own accept isn't idempotent — a second call on an
	// already-accepted token comes back as "not found" (see
	// `InvitationService::accept_invitation`) — and React 18 StrictMode
	// double-invokes effects in dev, which would otherwise show a spurious
	// "invalid link" error right after a successful join.
	const hasFired = useRef(false)
	const [errorMessage, setErrorMessage] = useState<string | null>(null)

	useEffect(() => {
		if (hasFired.current) return
		hasFired.current = true

		void (async () => {
			try {
				const accepted = await acceptInvitation.mutateAsync({
					path: { token },
				})

				// A plain `invalidateQueries` would not refetch here: nothing
				// under this route observes the org-list query (there is no
				// `OrgGate` on `/invite/$token`), so invalidating it alone
				// would leave a stale cache for whoever mounts `OrgGate` next.
				// `fetchQuery` both refreshes the cache and hands back the
				// freshly joined organization directly.
				const organizations = await queryClient.fetchQuery(
					window.tanstackApi.get(MY_ORGS_PATH).queryOptions,
				)
				const organization = organizations.data.find(
					(candidate) => candidate.id === accepted.data.organization_id,
				)

				if (!organization) {
					setErrorMessage(
						'Invitation acceptée, mais impossible de retrouver l’organisation. Rechargez la page.',
					)
					return
				}

				// No manual `localStorage` write for the "last visited" org:
				// landing on `/o/$organizationSlug` mounts `ActiveOrganizationProvider`,
				// which already persists it on every visit — see
				// `use-active-organization.tsx`.
				void navigate({ to: buildOrgPath(organization.slug, '/') })
			} catch (error) {
				setErrorMessage(describeError(error))
			}
		})()
	}, [acceptInvitation, navigate, queryClient, token])

	return (
		<InviteAcceptUI
			status={errorMessage ? 'error' : 'pending'}
			errorMessage={errorMessage}
		/>
	)
}

function errorStatus(error: unknown): number | undefined {
	if (error && typeof error === 'object' && 'status' in error) {
		const status = (error as { status?: unknown }).status
		return typeof status === 'number' ? status : undefined
	}
	return undefined
}

/**
 * The backend deliberately returns the same 404 for an unknown, expired, or
 * already-consumed token (see `InvitationService::accept_invitation`'s doc
 * comment) so that none of the three is distinguishable from the others —
 * this copy stays equally generic on purpose, rather than guessing which one
 * happened. `409` is the one case the backend *does* single out.
 */
function describeError(error: unknown): string {
	const status = errorStatus(error)
	if (status === 409) {
		return 'Vous êtes déjà membre de cette organisation.'
	}
	if (status === 404) {
		return 'Ce lien d’invitation n’est plus valide. Il a peut-être expiré ou déjà été utilisé — demandez-en un nouveau à un administrateur.'
	}
	return 'Une erreur est survenue lors de l’acceptation de l’invitation.'
}

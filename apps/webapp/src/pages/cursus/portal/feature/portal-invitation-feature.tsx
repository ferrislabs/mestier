import { useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { useEffect, useRef, useState } from 'react'
import { useAcceptCursusInvitation } from '#/pages/cursus/me/feature/use-my-cursus'
import { navigateToSpace } from '#/pages/cursus/portal/lib/space-target'
import { PortalInvitationUI } from '#/pages/cursus/portal/ui/portal-invitation-ui'

const GONE_MESSAGE =
	'Ce lien n’est plus valide, demandez un nouveau lien à votre centre de formation.'

function errorStatus(error: unknown): number | undefined {
	if (error && typeof error === 'object' && 'status' in error) {
		const status = (error as { status?: unknown }).status
		return typeof status === 'number' ? status : undefined
	}
	return undefined
}

function describeError(error: unknown): string {
	const status = errorStatus(error)
	if (status === 410) return GONE_MESSAGE
	if (status === 409 && error instanceof Error) return error.message
	return 'Une erreur est survenue lors de l’acceptation de l’invitation.'
}

export function PortalInvitationFeature({ token }: { token: string }) {
	const navigate = useNavigate()
	const queryClient = useQueryClient()
	const accept = useAcceptCursusInvitation()
	const hasFired = useRef(false)
	const [errorMessage, setErrorMessage] = useState<string | null>(null)

	useEffect(() => {
		if (hasFired.current) return
		hasFired.current = true

		void (async () => {
			try {
				const accepted = await accept.mutateAsync({ path: { token } })
				await queryClient.invalidateQueries({
					predicate: (query) => {
						const meta = query.queryKey[0]
						return (
							typeof meta === 'object' &&
							meta !== null &&
							(meta as { _id?: unknown })._id === '/api/v1/cursus/me/spaces'
						)
					},
				})
				await navigateToSpace(navigate, accepted.data, true)
			} catch (error) {
				setErrorMessage(describeError(error))
			}
		})()
	}, [accept, navigate, queryClient, token])

	return <PortalInvitationUI errorMessage={errorMessage} />
}

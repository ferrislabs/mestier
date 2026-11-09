import { useNavigate } from '@tanstack/react-router'
import { useEffect } from 'react'
import { useMyCursusSpaces } from '#/pages/cursus/me/feature/use-my-cursus'
import { navigateToSpace } from '#/pages/cursus/portal/lib/space-target'
import { PortalSpacesList } from '#/pages/cursus/portal/ui/portal-spaces-list'

export function PortalSpacesFeature() {
	const navigate = useNavigate()
	const spaces = useMyCursusSpaces()
	const items = spaces.data?.data ?? []
	const soleParticipantSpace =
		items.length === 1 && !items[0]?.is_member ? items[0] : undefined

	useEffect(() => {
		if (soleParticipantSpace) {
			void navigateToSpace(navigate, soleParticipantSpace, true)
		}
	}, [navigate, soleParticipantSpace])

	return (
		<PortalSpacesList
			spaces={items}
			isLoading={spaces.isLoading || soleParticipantSpace !== undefined}
			onOpen={(space) => void navigateToSpace(navigate, space)}
		/>
	)
}

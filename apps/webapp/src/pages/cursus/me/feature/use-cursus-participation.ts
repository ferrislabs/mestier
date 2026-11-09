import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useMyCursusSpaces } from '#/pages/cursus/me/feature/use-my-cursus'

export function useIsCursusParticipant(): boolean {
	const { activeOrganizationId } = useActiveOrganization()
	const spaces = useMyCursusSpaces()

	return (
		spaces.data?.data.some(
			(space) => space.organization_id === activeOrganizationId,
		) ?? false
	)
}

export function useHasPortalSpace(): boolean {
	const spaces = useMyCursusSpaces()

	return spaces.data?.data.some((space) => !space.is_member) ?? false
}

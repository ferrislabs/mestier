import { useMemo } from 'react'
import { useAuth } from 'react-oidc-context'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import {
	currentMemberOf,
	type Person,
	personOf,
} from '#/pages/customers/lib/people'
import { hueOf } from '#/pages/customers/ui/avatars'

export function useOrganizationPeople(organizationId: string) {
	const { members } = useReferenceCatalog(organizationId, {
		members: true,
		employeeProfiles: false,
		equipment: false,
		serviceRates: false,
		products: false,
		customUnits: false,
	})
	const auth = useAuth()
	const email = auth.user?.profile?.email
	const rows = members.data?.data
	const people = useMemo<readonly Person[]>(
		() => (rows ?? []).map((member) => personOf(member, hueOf)),
		[rows],
	)
	const currentPersonId = useMemo(
		() => currentMemberOf(rows ?? [], email)?.id ?? null,
		[rows, email],
	)
	return { people, currentPersonId, isLoading: members.isLoading }
}

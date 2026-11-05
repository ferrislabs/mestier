import { useMemo } from 'react'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import type { Person } from '#/pages/projects/lib/model'

function hueOf(id: string): number {
	let hash = 0
	for (const char of id) hash = (hash * 31 + char.charCodeAt(0)) % 360
	return hash
}

export function usePeople(organizationId: string): {
	data: Person[]
	isLoading: boolean
} {
	const { members } = useReferenceCatalog(organizationId, {
		members: true,
		employeeProfiles: false,
		equipment: false,
		serviceRates: false,
		products: false,
		customUnits: false,
	})
	const data = useMemo(
		() =>
			(members.data?.data ?? []).map(
				(member): Person => ({
					id: member.id,
					name: member.display_name,
					role: '',
					hue: hueOf(member.id),
				}),
			),
		[members.data],
	)
	return { data, isLoading: members.isLoading }
}

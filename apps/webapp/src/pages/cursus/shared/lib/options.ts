import type { Schemas } from '#/api/api.client'

export interface SelectOption {
	value: string
	label: string
}

export function participantName(
	participant: Pick<Schemas.ParticipantResponse, 'first_name' | 'last_name'>,
): string {
	return [participant.first_name, participant.last_name]
		.filter(Boolean)
		.join(' ')
}

export function apprenticeOptions(
	items: Schemas.ParticipantListItemResponse[],
): SelectOption[] {
	return items.flatMap((item) =>
		item.apprentice_id && item.status !== 'inactive'
			? [{ value: item.apprentice_id, label: participantName(item) }]
			: [],
	)
}

export function companyTutorOptions(
	items: Schemas.ParticipantListItemResponse[],
	hostCompanyId: string,
): SelectOption[] {
	return items
		.filter((item) => item.status !== 'inactive')
		.flatMap((item) =>
			item.company_tutors
				.filter((tutor) => tutor.host_company_id === hostCompanyId)
				.map((tutor) => ({
					value: tutor.id,
					label: participantName(item),
				})),
		)
}

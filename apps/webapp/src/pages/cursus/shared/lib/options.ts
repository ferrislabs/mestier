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
	details: Schemas.ParticipantDetailResponse[],
): SelectOption[] {
	return details
		.filter((detail) => detail.apprentice && detail.status !== 'inactive')
		.map((detail) => ({
			value: detail.apprentice?.id as string,
			label: participantName(detail),
		}))
}

export function companyTutorOptions(
	details: Schemas.ParticipantDetailResponse[],
	hostCompanyId: string,
): SelectOption[] {
	return details
		.filter((detail) => detail.status !== 'inactive')
		.flatMap((detail) =>
			detail.company_tutors
				.filter((tutor) => tutor.host_company_id === hostCompanyId)
				.map((tutor) => ({
					value: tutor.id,
					label: participantName(detail),
				})),
		)
}

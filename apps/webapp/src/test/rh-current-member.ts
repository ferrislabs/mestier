export const MY_MEMBER_PATH =
	'/api/v1/organizations/{organization_id}/members/me'

export function myMemberResponse(memberId = 'u-1') {
	return {
		data: {
			member_id: memberId,
			organization_id: 'org-1',
			display_name: 'Nova Alix',
		},
		pagination: null,
	}
}

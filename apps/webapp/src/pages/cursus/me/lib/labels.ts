import type { Schemas } from '#/api/api.client'

export const ROLE_LABELS: Record<Schemas.TriadRoleResponse, string> = {
	apprentice: 'Apprenti',
	main_tutor: 'Tuteur principal',
	tutor: 'Tuteur',
}

export function readOnlySince(
	access: Schemas.TriadAccessResponse,
): string | null {
	return access.kind === 'read_only' ? access.since : null
}

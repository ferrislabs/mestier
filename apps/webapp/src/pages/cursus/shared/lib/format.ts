import type { Schemas } from '#/api/api.client'

export const CONTRACT_KIND_LABELS: Record<Schemas.ContractKind, string> = {
	APPRENTICESHIP: 'Apprentissage',
	PROFESSIONALIZATION: 'Professionnalisation',
}

export const CONTRACT_KINDS: Schemas.ContractKind[] = [
	'APPRENTICESHIP',
	'PROFESSIONALIZATION',
]

export function formatDate(value: string | null | undefined): string {
	if (!value) return '—'
	const [year, month, day] = value.slice(0, 10).split('-')
	if (!year || !month || !day) return value
	return `${day}/${month}/${year}`
}

export function todayIso(): string {
	return new Date().toISOString().slice(0, 10)
}

export function memberLabel(member: Schemas.MemberResponse): string {
	return member.display_name
}

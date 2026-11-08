import { DEFAULT_ACCOUNT_ID } from '#/pages/hr/lib/rh/balances'

export const ENTRY = {
	members: 'members',
	planning: 'planning',
	requests: 'requests',
	pending: 'pending',
	mine: 'mine',
	types: 'types',
	account: (accountId: string) => `account:${accountId}`,
	extraRequests: 'extra-requests',
	extraPending: 'extra-pending',
	extraMine: 'extra-mine',
	extraSurcharges: 'extra-surcharges',
	workTime: 'work-time',
	equipment: 'equipment',
} as const

function text(value: unknown): string | null {
	return typeof value === 'string' && value !== '' ? value : null
}

export function activeSidebarEntries(
	modulePath: string,
	search: Readonly<Record<string, unknown>>,
	currentMemberId: string,
): ReadonlySet<string> {
	if (modulePath.startsWith('/rh/membres')) return new Set([ENTRY.members])
	if (modulePath.startsWith('/rh/planning')) return new Set([ENTRY.planning])
	if (modulePath.startsWith('/rh/temps-de-travail'))
		return new Set([ENTRY.workTime])
	if (modulePath.startsWith('/rh/materiel')) return new Set([ENTRY.equipment])
	if (modulePath.startsWith('/rh/absences')) {
		if (modulePath.startsWith('/rh/absences/nouvelle'))
			return new Set([ENTRY.requests])
		if (search.onglet === 'motifs') return new Set([ENTRY.types])
		if (search.statut === 'pending') return new Set([ENTRY.pending])
		if (currentMemberId !== '' && search.membre === currentMemberId)
			return new Set([ENTRY.mine])
		return new Set([ENTRY.requests])
	}
	if (modulePath.startsWith('/rh/extra')) {
		if (modulePath.startsWith('/rh/extra/nouvelle'))
			return new Set([ENTRY.extraRequests])
		if (search.onglet === 'majorations') return new Set([ENTRY.extraSurcharges])
		if (search.statut === 'pending') return new Set([ENTRY.extraPending])
		if (currentMemberId !== '' && search.membre === currentMemberId)
			return new Set([ENTRY.extraMine])
		return new Set([ENTRY.extraRequests])
	}
	if (modulePath.startsWith('/rh/droits'))
		return new Set([ENTRY.account(text(search.compte) ?? DEFAULT_ACCOUNT_ID)])
	return new Set()
}

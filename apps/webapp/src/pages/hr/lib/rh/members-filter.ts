import { currentContract } from '#/pages/hr/lib/rh/contracts'
import type { ContractType, IsoDate, Member } from '#/pages/hr/lib/rh/types'

export type StatusFilter = 'active' | 'archived' | 'all'

export type ContractFilter = ContractType | 'all'

export type MemberFilters = {
	query: string
	status: StatusFilter
	contract: ContractFilter
}

export const DEFAULT_FILTERS: MemberFilters = {
	query: '',
	status: 'active',
	contract: 'all',
}

export function normalizeText(text: string): string {
	return text.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase().trim()
}

function compactDigits(text: string): string {
	return text.replace(/[\s.-]/g, '')
}

function matches(
	member: Member,
	filters: MemberFilters,
	query: string,
	today: IsoDate,
) {
	if (filters.status !== 'all' && member.status !== filters.status) return false
	if (
		filters.contract !== 'all' &&
		currentContract(member, today).type !== filters.contract
	)
		return false
	if (query === '') return true
	const haystack = normalizeText(
		[
			member.firstName,
			member.lastName,
			member.jobTitle,
			member.workEmail,
			member.personalEmail,
			member.mobile,
		].join(' '),
	)
	if (haystack.includes(query)) return true
	return (
		/\d/.test(query) && compactDigits(haystack).includes(compactDigits(query))
	)
}

export function filterMembers(
	members: readonly Member[],
	filters: MemberFilters,
	today: IsoDate,
): Member[] {
	const query = normalizeText(filters.query)
	const kept = members.filter((member) =>
		matches(member, filters, query, today),
	)
	return [
		...kept.filter((member) => member.status === 'active'),
		...kept.filter((member) => member.status === 'archived'),
	]
}

export function membersSummary(members: readonly Member[]): string {
	const active = members.filter((member) => member.status === 'active').length
	const archived = members.length - active
	const base =
		active === 0
			? 'Aucun membre actif'
			: `${active} ${active > 1 ? 'membres actifs' : 'membre actif'}`
	return archived === 0
		? base
		: `${base}, ${archived} ${archived > 1 ? 'archivés' : 'archivé'}`
}

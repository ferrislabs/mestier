import type { SupportPriority } from '#/pages/support/types'

export const SUPPORT_PRIORITIES: ReadonlyArray<{
	id: SupportPriority
	label: string
}> = [
	{ id: 'urgent', label: 'Urgente' },
	{ id: 'high', label: 'Haute' },
	{ id: 'normal', label: 'Normale' },
	{ id: 'low', label: 'Basse' },
]

export const DEFAULT_SUPPORT_PRIORITY: SupportPriority = 'normal'

export function priorityRank(priority: SupportPriority): number {
	return SUPPORT_PRIORITIES.findIndex((entry) => entry.id === priority)
}

export function priorityLabel(priority: SupportPriority): string {
	return (
		SUPPORT_PRIORITIES.find((entry) => entry.id === priority)?.label ?? priority
	)
}

export function isSupportPriority(value: unknown): value is SupportPriority {
	return SUPPORT_PRIORITIES.some((entry) => entry.id === value)
}

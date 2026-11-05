import type { AppModule } from '#/modules/types'

export type ActivityState = 'late' | 'failed' | 'today' | 'progress' | 'waiting'

export interface HomeActivity {
	id: string
	title: string
	context?: string
	state: ActivityState
	when?: string
	time?: string
	progress?: number
	to: string
	module: AppModule
}

export interface ActivityGroup {
	id: string
	title: string
	states: ActivityState[]
}

export const ACTIVITY_GROUPS: ActivityGroup[] = [
	{ id: 'now', title: 'À traiter maintenant', states: ['late', 'failed'] },
	{ id: 'today', title: 'Aujourd’hui', states: ['today'] },
	{ id: 'progress', title: 'En cours', states: ['progress'] },
	{ id: 'waiting', title: 'En attente', states: ['waiting'] },
]

export function itemsOfGroup(
	items: HomeActivity[],
	group: ActivityGroup,
): HomeActivity[] {
	return items.filter((item) => group.states.includes(item.state))
}

export function formatToday(date: Date): string {
	const formatted = new Intl.DateTimeFormat('fr-FR', {
		weekday: 'long',
		day: 'numeric',
		month: 'long',
	}).format(date)
	return formatted.charAt(0).toUpperCase() + formatted.slice(1)
}

export function greeting(date: Date): string {
	return date.getHours() >= 18 ? 'Bonsoir' : 'Bonjour'
}

export function firstNameOf(
	profile: { given_name?: string; name?: string } | undefined,
): string | undefined {
	const given = profile?.given_name?.trim()
	if (given) return given
	const first = profile?.name?.trim().split(/\s+/)[0]
	return first || undefined
}

export function activityCountLabel(count: number): string {
	return count > 0 ? `${count} à suivre` : 'Rien à suivre'
}

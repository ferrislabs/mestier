import type { PlanningEntry } from '#/hooks/use-planning'
import { usePlanning } from '#/hooks/use-planning'
import {
	TodayPlanningUI,
	type TodayTaskRow,
} from '#/pages/home/ui/today-planning-ui'
import { todayIsoDate } from '#/pages/planning/types'

interface TodayPlanningFeatureProps {
	organizationId: string
	organizationSlug: string
}

const MAX_ROWS = 5

/**
 * Today's team-wide agenda for the homepage — a `usePlanning` read scoped to
 * a single day, distinct from `MyTasksTodayFeature`'s `useMyFieldTasks`
 * (the caller's own tasks, for pointage).
 */
export function TodayPlanningFeature({
	organizationId,
	organizationSlug,
}: TodayPlanningFeatureProps) {
	const today = todayIsoDate()
	const planning = usePlanning(organizationId, { from: today, to: today })

	const response = planning.data?.data
	const nameByMemberId = new Map(
		(response?.resources ?? []).map((resource) => [
			resource.member_id,
			resource.display_name,
		]),
	)

	const entries: TodayTaskRow[] = (response?.entries ?? [])
		.filter(isTaskEntry)
		.sort((a, b) => a.starts_at.localeCompare(b.starts_at))
		.slice(0, MAX_ROWS)
		.map((task) => ({
			id: task.id,
			timeWindow: task.all_day
				? 'Toute la journée'
				: formatTimeWindow(task.starts_at, task.ends_at),
			title: task.title,
			subtitle:
				task.customer_name ?? assigneeNames(task.member_ids, nameByMemberId),
		}))

	return (
		<TodayPlanningUI
			organizationSlug={organizationSlug}
			entries={entries}
			isLoading={planning.isLoading}
			error={planning.error?.message ?? null}
		/>
	)
}

function isTaskEntry(
	entry: PlanningEntry,
): entry is Extract<PlanningEntry, { kind: 'task' }> {
	return entry.kind === 'task'
}

function assigneeNames(
	memberIds: string[],
	nameByMemberId: Map<string, string>,
): string | null {
	const names = memberIds
		.map((id) => nameByMemberId.get(id))
		.filter((name): name is string => Boolean(name))
	return names.length > 0 ? names.join(', ') : null
}

function formatTimeWindow(startsAt: string, endsAt: string): string {
	const formatter = new Intl.DateTimeFormat('fr-FR', {
		hour: '2-digit',
		minute: '2-digit',
	})
	return `${formatter.format(new Date(startsAt))}–${formatter.format(new Date(endsAt))}`
}

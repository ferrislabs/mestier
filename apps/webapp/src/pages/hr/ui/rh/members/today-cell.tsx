import { findType } from '#/pages/hr/lib/rh/absences'
import { formatWorkDay } from '#/pages/hr/lib/rh/schedule'
import { todayStatus } from '#/pages/hr/lib/rh/today-status'
import type { IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'
import { TypeChip } from '#/pages/hr/ui/rh/shared/type-chip'

const PART_LABELS = { morning: 'matin', afternoon: 'après-midi' } as const

export function TodayCell({
	state,
	member,
	today,
}: {
	state: RhState
	member: Member
	today: IsoDate
}) {
	const status = todayStatus(state, member, today)
	switch (status.kind) {
		case 'inactive':
			return (
				<span className="text-muted-foreground">
					<span aria-hidden>—</span>
					<span className="sr-only">Inactif</span>
				</span>
			)
		case 'work':
			return (
				<span className="block">
					<span className="inline-flex items-center gap-1.5">
						<span
							aria-hidden
							className="size-1.5 shrink-0 rounded-full bg-state-progress"
						/>
						Au travail
					</span>
					<span className="tabular block pl-3 text-[12px] text-muted-foreground">
						{formatWorkDay(status)}
					</span>
				</span>
			)
		case 'off':
			return (
				<span className="text-muted-foreground">{status.label ?? 'Repos'}</span>
			)
		case 'unplanned':
			return (
				<span className="inline-flex h-5 items-center rounded-sm border border-muted-foreground/50 border-dashed px-1.5 font-medium text-[11px] text-muted-foreground">
					À planifier
				</span>
			)
		case 'holiday':
			return (
				<span className="text-muted-foreground" title={status.name}>
					Férié
				</span>
			)
		case 'absence': {
			const type = findType(state, status.typeId) ?? {
				name: 'Absence',
				hue: 250,
			}
			const suffix = [
				status.part === 'full' ? null : PART_LABELS[status.part],
				status.pending ? 'à valider' : null,
			]
				.filter(Boolean)
				.join(' · ')
			return <TypeChip type={type} suffix={suffix || undefined} />
		}
	}
}

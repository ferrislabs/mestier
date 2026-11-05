import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import { weekly } from '#/pages/hr/lib/rh/contract-schedule'
import {
	convertSchedule,
	SCHEDULE_MODES,
	type ScheduleMode,
} from '#/pages/hr/lib/rh/schedule-mode'
import type { ContractSchedule, IsoDate } from '#/pages/hr/lib/rh/types'
import { RotationEditor } from '#/pages/hr/ui/rh/members/rotation-editor'
import { ScheduleEditor } from '#/pages/hr/ui/rh/members/schedule-editor'
import { VariableEditor } from '#/pages/hr/ui/rh/members/variable-editor'

const HINTS: Record<ScheduleMode, string> = {
	weekly: 'Les mêmes jours et horaires chaque semaine.',
	rotation:
		'Des séquences de semaines ou de mois qui se répètent : alternance, semaine A / semaine B…',
	variable: 'Pas de semaine type : les jours se planifient dans le calendrier.',
}

export function ScheduleModeEditor({
	idPrefix,
	value,
	onChange,
	contractStart,
	today,
}: {
	idPrefix: string
	value: ContractSchedule
	onChange: (schedule: ContractSchedule) => void
	contractStart: IsoDate
	today: IsoDate
}) {
	return (
		<div className="grid gap-4">
			<div className="flex flex-wrap items-center gap-3">
				<ToggleGroup
					type="single"
					variant="outline"
					size="sm"
					value={value.kind}
					onValueChange={(mode) =>
						mode &&
						onChange(
							convertSchedule(value, mode as ScheduleMode, contractStart),
						)
					}
					aria-label="Type de rythme de travail"
				>
					{SCHEDULE_MODES.map((mode) => (
						<ToggleGroupItem key={mode.id} value={mode.id}>
							{mode.label}
						</ToggleGroupItem>
					))}
				</ToggleGroup>
				<p className="text-[12px] text-muted-foreground">{HINTS[value.kind]}</p>
			</div>
			{value.kind === 'weekly' && (
				<ScheduleEditor
					idPrefix={`${idPrefix}-week`}
					value={value.week}
					onChange={(week) => onChange(weekly(week))}
				/>
			)}
			{value.kind === 'rotation' && (
				<RotationEditor
					idPrefix={`${idPrefix}-rotation`}
					value={value}
					onChange={onChange}
					from={contractStart > today ? contractStart : today}
					today={today}
				/>
			)}
			{value.kind === 'variable' && (
				<VariableEditor
					idPrefix={`${idPrefix}-variable`}
					value={value.defaultDay}
					onChange={(defaultDay) => onChange({ kind: 'variable', defaultDay })}
				/>
			)}
		</div>
	)
}

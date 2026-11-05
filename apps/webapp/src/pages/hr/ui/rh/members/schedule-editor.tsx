import { Copy } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Switch } from '#/components/ui/switch'
import { cn } from '#/lib/utils'
import { WEEKDAY_LABELS } from '#/pages/hr/lib/rh/dates'
import { dayHoursLabel } from '#/pages/hr/lib/rh/format'
import { formatHours, weeklyHours } from '#/pages/hr/lib/rh/members'
import {
	errorTargetsLunch,
	scheduleErrors,
	withDay,
	withDayApplied,
	withDayToggled,
	withLunch,
	withLunchToggled,
} from '#/pages/hr/lib/rh/schedule-draft'
import type { WeekSchedule } from '#/pages/hr/lib/rh/types'

const INDENT = 'pl-[calc(28px+6rem+1.5rem)]'

function TimeRange({
	label,
	start,
	end,
	invalid,
	describedBy,
	onStart,
	onEnd,
}: {
	label: string
	start: string
	end: string
	invalid: boolean
	describedBy?: string
	onStart: (value: string) => void
	onEnd: (value: string) => void
}) {
	return (
		<>
			<Input
				type="time"
				value={start}
				onChange={(event) => onStart(event.target.value)}
				aria-label={`Début ${label}`}
				aria-invalid={invalid ? true : undefined}
				aria-describedby={describedBy}
				className="h-8 w-36 shrink-0 bg-card"
			/>
			<span aria-hidden className="text-muted-foreground">
				–
			</span>
			<Input
				type="time"
				value={end}
				onChange={(event) => onEnd(event.target.value)}
				aria-label={`Fin ${label}`}
				aria-invalid={invalid ? true : undefined}
				aria-describedby={describedBy}
				className="h-8 w-36 shrink-0 bg-card"
			/>
		</>
	)
}

export function ScheduleEditor({
	idPrefix,
	value,
	onChange,
}: {
	idPrefix: string
	value: WeekSchedule
	onChange: (schedule: WeekSchedule) => void
}) {
	const errors = scheduleErrors(value)
	const workedCount = value.filter((day) => day !== null).length
	return (
		<div className="grid gap-2">
			<ul className="grid gap-3">
				{value.map((day, index) => {
					const label = WEEKDAY_LABELS[index]
					const lower = label.toLowerCase()
					const error = errors[index]
					const invalid = error !== null
					const lunchInvalid = errorTargetsLunch(error)
					const dayInvalid = invalid && !lunchInvalid
					const switchId = `${idPrefix}-worked-${index}`
					const lunchId = `${idPrefix}-lunch-${index}`
					const errorId = `${idPrefix}-error-${index}`
					const describedBy = invalid ? errorId : undefined
					return (
						<li key={label} className="grid gap-1.5">
							<div className="flex min-h-8 items-center gap-3">
								<Switch
									id={switchId}
									size="default"
									checked={day !== null}
									onCheckedChange={(worked) =>
										onChange(withDayToggled(value, index, worked))
									}
									aria-label={`${label} travaillé`}
								/>
								<Label
									htmlFor={switchId}
									className={cn(
										'w-24 shrink-0 font-normal',
										day === null && 'text-muted-foreground',
									)}
								>
									{label}
								</Label>
								{day ? (
									<>
										<TimeRange
											label={lower}
											start={day.start}
											end={day.end}
											invalid={dayInvalid}
											describedBy={describedBy}
											onStart={(start) =>
												onChange(withDay(value, index, { start }))
											}
											onEnd={(end) => onChange(withDay(value, index, { end }))}
										/>
										<span className="tabular ml-auto w-14 text-right text-[13px] text-muted-foreground">
											{dayHoursLabel(day)}
										</span>
										{workedCount > 1 ? (
											<Button
												type="button"
												variant="ghost"
												size="icon-xs"
												aria-label={`Appliquer les horaires du ${lower} à tous les jours travaillés`}
												title="Appliquer à tous les jours travaillés"
												onClick={() => onChange(withDayApplied(value, index))}
											>
												<Copy aria-hidden />
											</Button>
										) : (
											<span aria-hidden className="size-6 shrink-0" />
										)}
									</>
								) : (
									<span className="text-[13px] text-muted-foreground">
										Repos
									</span>
								)}
							</div>
							{day && (
								<div className="flex min-h-8 items-center gap-3">
									<div className="flex w-[calc(28px+6rem+0.75rem)] shrink-0 items-center justify-end gap-2">
										<Label
											htmlFor={lunchId}
											className="font-normal text-[13px] text-muted-foreground"
										>
											Pause
										</Label>
										<Switch
											id={lunchId}
											size="sm"
											checked={Boolean(day.lunch)}
											onCheckedChange={(enabled) =>
												onChange(withLunchToggled(value, index, enabled))
											}
											aria-label={`Pause ${lower}`}
										/>
									</div>
									{day.lunch ? (
										<TimeRange
											label={`de pause ${lower}`}
											start={day.lunch.start}
											end={day.lunch.end}
											invalid={lunchInvalid}
											describedBy={describedBy}
											onStart={(start) =>
												onChange(withLunch(value, index, { start }))
											}
											onEnd={(end) =>
												onChange(withLunch(value, index, { end }))
											}
										/>
									) : (
										<span className="text-[13px] text-muted-foreground">
											Sans pause
										</span>
									)}
								</div>
							)}
							{error && (
								<p
									id={errorId}
									className={cn('text-[12px] text-state-alert', INDENT)}
								>
									{error}
								</p>
							)}
						</li>
					)
				})}
			</ul>
			<div className="flex items-baseline justify-between gap-4 border-t pt-2.5">
				<p className="text-[12px] text-muted-foreground">
					Les durées sont nettes de la pause déjeuner.
				</p>
				<p className="shrink-0 text-sm">
					<span className="text-muted-foreground">Total hebdo net </span>
					<output className="tabular font-semibold">
						{formatHours(weeklyHours(value))}
					</output>
				</p>
			</div>
		</div>
	)
}

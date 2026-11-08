import { Link } from '@tanstack/react-router'
import { CalendarRange, ChevronLeft, ChevronRight } from 'lucide-react'
import { type KeyboardEvent, useRef, useState } from 'react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import {
	cellModel,
	isSameMonth,
	monthLabel,
	monthWeeks,
	shiftMonth,
} from '#/pages/hr/lib/rh/calendar-model'
import {
	HATCH_INACTIVE,
	TODAY_SOFT,
	WEEKEND_SOFT,
} from '#/pages/hr/lib/rh/calendar-style'
import { currentContract } from '#/pages/hr/lib/rh/contracts'
import {
	addDays,
	startOfMonth,
	startOfWeek,
	WEEKDAY_SHORT,
	weekdayOf,
} from '#/pages/hr/lib/rh/dates'
import type { IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'
import { CalendarCell } from '#/pages/hr/ui/rh/members/calendar-cell'
import { CalendarLegend } from '#/pages/hr/ui/rh/members/calendar-legend'
import { DetailCard } from '#/pages/hr/ui/rh/members/detail-card'
import { PlanDaysDialog } from '#/pages/hr/ui/rh/members/plan-days-dialog'
import { CALENDAR_ANCHOR } from '#/pages/hr/ui/rh/members/rhythm-card'
import { useRhOrgPath } from '#/pages/hr/ui/rh/shared/rh-context'

const STEPS: Record<string, number> = {
	ArrowLeft: -1,
	ArrowRight: 1,
	ArrowUp: -7,
	ArrowDown: 7,
}

export function MemberCalendar({
	state,
	member,
	today,
	month,
	onMonthChange,
}: {
	state: RhState
	member: Member
	today: IsoDate
	month: IsoDate
	onMonthChange: (month: IsoDate | null) => void
}) {
	const orgPath = useRhOrgPath()
	const gridRef = useRef<HTMLTableElement>(null)
	const [focused, setFocused] = useState<IsoDate>(today)
	const first = startOfMonth(month)
	const weeks = monthWeeks(first)
	const days = weeks.flat()
	const activeDate = days.includes(focused)
		? focused
		: days.includes(today)
			? today
			: first
	const viewingToday = isSameMonth(first, today)
	const variable = currentContract(member, today).schedule.kind === 'variable'

	function onKeyDown(event: KeyboardEvent<HTMLButtonElement>, date: IsoDate) {
		const step = STEPS[event.key]
		let target: IsoDate | null = null
		if (step !== undefined) target = addDays(date, step)
		else if (event.key === 'Home') target = startOfWeek(date)
		else if (event.key === 'End') target = addDays(startOfWeek(date), 6)
		if (target === null) return
		event.preventDefault()
		if (!days.includes(target)) return
		setFocused(target)
		gridRef.current
			?.querySelector<HTMLButtonElement>(`[data-date="${target}"]`)
			?.focus()
	}

	return (
		<DetailCard
			id={CALENDAR_ANCHOR}
			title="Calendrier"
			action={
				<Link
					to={orgPath('/rh/planning')}
					search={{ vue: 'month', date: first }}
					className="rounded-sm text-[13px] text-primary underline-offset-4 outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
				>
					Ouvrir dans le planning
				</Link>
			}
		>
			<div className="grid gap-3">
				<div className="flex flex-wrap items-center gap-2">
					<h4 aria-live="polite" className="mr-auto font-semibold text-base">
						{monthLabel(first)}
					</h4>
					{member.status === 'active' && (
						<PlanDaysDialog
							member={member}
							today={today}
							month={first}
							trigger={
								<Button variant={variable ? 'default' : 'outline'} size="sm">
									<CalendarRange aria-hidden /> Planifier des jours
								</Button>
							}
						/>
					)}
					<Button
						variant="outline"
						size="sm"
						disabled={viewingToday}
						onClick={() => onMonthChange(null)}
					>
						Aujourd’hui
					</Button>
					<Button
						variant="outline"
						size="icon-sm"
						aria-label="Mois précédent"
						onClick={() => onMonthChange(shiftMonth(first, -1))}
					>
						<ChevronLeft aria-hidden />
					</Button>
					<Button
						variant="outline"
						size="icon-sm"
						aria-label="Mois suivant"
						onClick={() => onMonthChange(shiftMonth(first, 1))}
					>
						<ChevronRight aria-hidden />
					</Button>
				</div>
				<div className="overflow-hidden rounded-lg border">
					<table
						ref={gridRef}
						className="w-full table-fixed border-separate border-spacing-0"
					>
						<caption className="sr-only">
							{`Emploi du temps de ${member.firstName} en ${monthLabel(first)}`}
						</caption>
						<thead>
							<tr>
								{WEEKDAY_SHORT.map((label, index) => (
									<th
										key={label}
										scope="col"
										className={cn(
											'border-b bg-muted/40 py-1.5 font-medium text-[11px] text-muted-foreground uppercase',
											index < 6 && 'border-r',
										)}
									>
										{label}
									</th>
								))}
							</tr>
						</thead>
						<tbody>
							{weeks.map((week, row) => (
								<tr key={week[0]}>
									{week.map((date) => {
										const cell = cellModel(state, member, date)
										const column = weekdayOf(date)
										return (
											<td
												key={date}
												className={cn(
													'p-0 align-top',
													row < weeks.length - 1 && 'border-b',
													column < 6 && 'border-r',
													column >= 5 && WEEKEND_SOFT,
													date === today && TODAY_SOFT,
													cell.morning.kind === 'inactive' &&
														cell.afternoon.kind === 'inactive' &&
														HATCH_INACTIVE,
												)}
											>
												<CalendarCell
													state={state}
													member={member}
													cell={cell}
													isToday={date === today}
													inMonth={isSameMonth(date, first)}
													tabIndex={date === activeDate ? 0 : -1}
													onFocusCell={() => setFocused(date)}
													onKeyDown={(event) => onKeyDown(event, date)}
												/>
											</td>
										)
									})}
								</tr>
							))}
						</tbody>
					</table>
				</div>
				<CalendarLegend types={state.absenceTypes} />
			</div>
		</DetailCard>
	)
}

import { cn } from '#/lib/utils'
import {
	HATCH_INACTIVE,
	HATCH_PENDING,
	HOLIDAY_FILL,
	hueStyle,
	TODAY_SOFT,
	TODAY_SOLID,
	UNPLANNED_FACE,
	WEEKEND_SOFT,
	WEEKEND_SOLID,
} from '#/pages/hr/lib/rh/cell-style'
import { formatLong, WEEKDAY_SHORT, weekdayOf } from '#/pages/hr/lib/rh/dates'
import { extrasOn } from '#/pages/hr/lib/rh/extra'
import { holidayName } from '#/pages/hr/lib/rh/holidays'
import { type HalfDay, resolveDay } from '#/pages/hr/lib/rh/planning'
import {
	extraTotals,
	isWeekend,
	presentCount,
	restExtraLabel,
	sameHalf,
	type TypeIndex,
	typeInfo,
} from '#/pages/hr/lib/rh/planning-model'
import type { ExtraPeriod } from '#/pages/hr/lib/rh/types'
import { DayCell } from '#/pages/hr/ui/rh/planning/day-cell'
import {
	EmptyRow,
	type GridProps,
	MemberLabel,
} from '#/pages/hr/ui/rh/planning/grid-parts'
import { useRovingCells } from '#/pages/hr/ui/rh/planning/use-roving-cells'
import { TYPE_SOLID, TYPE_TONE } from '#/pages/hr/ui/rh/shared/type-chip'

const MEMBER_COLUMN = 184
const DAY_COLUMN = 26

const CELL =
	'relative flex h-9 w-full items-center justify-center outline-none transition-colors hover:bg-accent/60 focus-visible:z-10 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset aria-expanded:bg-accent'

function halfTone(half: HalfDay): string {
	switch (half.kind) {
		case 'work':
			return 'bg-muted-foreground/20'
		case 'off':
			return half.label ? 'bg-muted-foreground/12' : ''
		case 'unplanned':
			return ''
		case 'holiday':
			return ''
		case 'absence':
			return half.pending ? `${TYPE_TONE} ${HATCH_PENDING}` : TYPE_SOLID
		case 'inactive':
			return HATCH_INACTIVE
	}
}

function MonthHalf({
	half,
	types,
	worked,
	wholeUnplanned,
}: {
	half: HalfDay
	types: TypeIndex
	worked: boolean
	wholeUnplanned: boolean
}) {
	return (
		<span
			style={
				half.kind === 'absence'
					? hueStyle(typeInfo(types, half.typeId).hue)
					: undefined
			}
			className={cn(
				'block min-h-0 flex-1',
				worked ? 'bg-muted-foreground/20' : halfTone(half),
				half.kind === 'unplanned' && !wholeUnplanned && UNPLANNED_FACE,
			)}
		/>
	)
}

function ExtraMark({ extras }: { extras: readonly ExtraPeriod[] }) {
	const totals = extraTotals(extras)
	if (totals.approved === 0 && totals.pending === 0) return null
	return (
		<span
			aria-hidden
			className={cn(
				'absolute right-0.5 bottom-0.5 size-2 rounded-full',
				totals.approved > 0
					? 'bg-foreground ring-1 ring-card'
					: 'border border-foreground bg-card',
			)}
		/>
	)
}

function dayTint(date: string, today: string, wholeHoliday: boolean): string {
	if (date === today) return TODAY_SOFT
	if (wholeHoliday) return HOLIDAY_FILL
	return isWeekend(date) ? WEEKEND_SOFT : ''
}

function columnBorder(date: string): string {
	return weekdayOf(date) === 0
		? 'border-l border-l-border'
		: 'border-l border-l-border/40'
}

export function MonthGrid({
	state,
	types,
	members,
	team,
	days,
	today,
	query,
	onRequestAbsence,
	onRequestExtra,
}: GridProps) {
	const roving = useRovingCells(
		members.length,
		days.length,
		Math.max(days.indexOf(today), 0),
	)
	const filtered = query.trim() !== ''
	return (
		<div className="min-h-0 overflow-auto rounded-lg border bg-card [scroll-padding:52px_0_48px_188px]">
			<table
				ref={roving.tableRef}
				className="w-full table-fixed border-separate border-spacing-0"
				style={{ minWidth: MEMBER_COLUMN + days.length * DAY_COLUMN }}
			>
				<caption className="sr-only">
					Planning du mois : une ligne par membre, une colonne par jour.
				</caption>
				<colgroup>
					<col style={{ width: MEMBER_COLUMN }} />
					{days.map((date) => (
						<col key={date} />
					))}
				</colgroup>
				<thead>
					<tr>
						<th
							scope="col"
							className="sticky top-0 left-0 z-30 border-r border-b bg-card px-3 py-2 text-left font-medium text-muted-foreground text-xs"
						>
							Membre
						</th>
						{days.map((date) => {
							const holiday = holidayName(date)
							const isToday = date === today
							return (
								<th
									key={date}
									scope="col"
									title={holiday}
									aria-label={formatLong(date)}
									className={cn(
										'sticky top-0 z-20 border-b bg-card p-0 font-normal',
										columnBorder(date),
										isWeekend(date) && WEEKEND_SOLID,
										isToday && TODAY_SOLID,
									)}
								>
									<span className="flex flex-col items-center py-1.5 leading-tight">
										<span className="text-[10px] text-muted-foreground uppercase">
											{WEEKDAY_SHORT[weekdayOf(date)].charAt(0)}
										</span>
										<span
											className={cn(
												'font-semibold text-xs tabular-nums',
												isToday && 'text-state-today',
											)}
										>
											{Number(date.slice(8, 10))}
										</span>
										<span
											aria-hidden
											className={cn(
												'mt-0.5 size-1 rounded-full',
												holiday ? 'bg-muted-foreground' : 'bg-transparent',
											)}
										/>
									</span>
								</th>
							)
						})}
					</tr>
				</thead>
				<tbody>
					{members.length === 0 && (
						<EmptyRow colSpan={days.length + 1} query={query} />
					)}
					{members.map((member, row) => (
						<tr key={member.id}>
							<th
								scope="row"
								className="sticky left-0 z-10 border-r border-b bg-card px-3 py-1.5 text-left font-normal"
							>
								<MemberLabel member={member} compact />
							</th>
							{days.map((date, col) => {
								const status = resolveDay(state, member, date)
								const extras = extrasOn(state, member.id, date)
								const wholeUnplanned =
									status.morning.kind === 'unplanned' &&
									status.afternoon.kind === 'unplanned'
								const wholeHoliday =
									status.morning.kind === 'holiday' &&
									status.afternoon.kind === 'holiday'
								const worked =
									sameHalf(status.morning, status.afternoon) &&
									(status.morning.kind === 'off' ||
										status.morning.kind === 'holiday') &&
									restExtraLabel(extras) !== null
								return (
									<td
										key={date}
										className={cn(
											'border-b p-0',
											columnBorder(date),
											dayTint(date, today, wholeHoliday),
										)}
									>
										<DayCell
											member={member}
											status={status}
											types={types}
											extras={extras}
											nav={roving.navFor(row, col)}
											className={CELL}
											onRequestAbsence={onRequestAbsence}
											onRequestExtra={onRequestExtra}
										>
											<span
												className={cn(
													'relative flex h-6 w-5 flex-col overflow-hidden rounded-[4px]',
													wholeUnplanned && UNPLANNED_FACE,
												)}
											>
												<MonthHalf
													half={status.morning}
													types={types}
													worked={worked}
													wholeUnplanned={wholeUnplanned}
												/>
												<MonthHalf
													half={status.afternoon}
													types={types}
													worked={worked}
													wholeUnplanned={wholeUnplanned}
												/>
												<ExtraMark extras={extras} />
											</span>
											{status.override && !status.variable && (
												<span
													aria-hidden
													className="absolute top-0.5 right-0.5 size-1.5 rounded-full bg-state-today ring-1 ring-card"
												/>
											)}
										</DayCell>
									</td>
								)
							})}
						</tr>
					))}
				</tbody>
				<tfoot>
					<tr>
						<th
							scope="row"
							className="sticky bottom-0 left-0 z-30 border-t border-r bg-card px-3 py-2 text-left"
						>
							<span className="block font-medium text-sm leading-tight">
								Présents
							</span>
							{filtered && (
								<span className="block font-normal text-muted-foreground text-xs leading-tight">
									Toute l’équipe
								</span>
							)}
						</th>
						{days.map((date) => {
							const count = presentCount(state, team, date)
							return (
								<td
									key={date}
									className={cn(
										'sticky bottom-0 z-20 border-t bg-card py-2 text-center font-semibold text-[11px] tabular-nums',
										columnBorder(date),
										isWeekend(date) && WEEKEND_SOLID,
										date === today && TODAY_SOLID,
										count === 0 && 'font-normal text-muted-foreground/60',
									)}
								>
									{count}
								</td>
							)
						})}
					</tr>
				</tfoot>
			</table>
		</div>
	)
}

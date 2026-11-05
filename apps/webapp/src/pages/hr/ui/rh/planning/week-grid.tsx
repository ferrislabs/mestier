import { cn } from '#/lib/utils'
import {
	EXTRA_BADGE,
	EXTRA_PENDING,
	EXTRA_SOLID,
	HATCH_INACTIVE,
	HATCH_PENDING,
	HOLIDAY_FILL,
	hueStyle,
	TODAY_SOFT,
	TODAY_SOLID,
	TYPE_BORDER,
	UNPLANNED_FACE,
	WEEKEND_SOFT,
	WEEKEND_SOLID,
} from '#/pages/hr/lib/rh/cell-style'
import { WEEKDAY_SHORT, weekdayOf } from '#/pages/hr/lib/rh/dates'
import { extrasOn, formatDuration } from '#/pages/hr/lib/rh/extra'
import { holidayName } from '#/pages/hr/lib/rh/holidays'
import { type HalfDay, resolveDay } from '#/pages/hr/lib/rh/planning'
import {
	extraTotals,
	type HalfScope,
	isWeekend,
	presentCount,
	restExtraLabel,
	sameHalf,
	shortWork,
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
import { TYPE_TONE } from '#/pages/hr/ui/rh/shared/type-chip'

const CELL =
	'relative block h-[52px] w-full overflow-hidden rounded-md text-left outline-none transition-colors hover:bg-accent/50 focus-visible:ring-2 focus-visible:ring-ring aria-expanded:ring-2 aria-expanded:ring-ring/70'

function AbsenceFace({
	half,
	scope,
	types,
}: {
	half: Extract<HalfDay, { kind: 'absence' }>
	scope: HalfScope
	types: TypeIndex
}) {
	const type = typeInfo(types, half.typeId)
	const sub = [
		scope === 'morning' ? 'matin' : scope === 'afternoon' ? 'après-midi' : null,
		half.pending ? 'à valider' : null,
	]
		.filter(Boolean)
		.join(' · ')
	return (
		<span
			style={hueStyle(type.hue)}
			className={cn(
				'flex size-full min-w-0 flex-col justify-center overflow-hidden px-2 leading-tight',
				TYPE_TONE,
				TYPE_BORDER,
				half.pending
					? `border border-dashed ${HATCH_PENDING}`
					: 'border-l-[3px]',
			)}
		>
			<span className="truncate font-medium text-xs">{type.name}</span>
			{sub && <span className="truncate text-[10px] opacity-80">{sub}</span>}
		</span>
	)
}

function HalfFace({
	half,
	scope,
	types,
}: {
	half: HalfDay
	scope: HalfScope
	types: TypeIndex
}) {
	switch (half.kind) {
		case 'work':
			return (
				<span className="flex size-full items-center justify-center text-muted-foreground text-xs tabular-nums">
					{shortWork(half, scope)}
				</span>
			)
		case 'off':
			return half.label ? (
				<span className="flex size-full items-center justify-center bg-muted/70 px-1 text-[11px] text-muted-foreground">
					<span className="truncate">{half.label}</span>
				</span>
			) : (
				<span className="flex size-full items-center justify-center text-[11px] text-muted-foreground/45">
					Repos
				</span>
			)
		case 'unplanned':
			return (
				<span
					className={cn(
						'flex size-full items-center justify-center rounded-[inherit] text-[11px] italic',
						UNPLANNED_FACE,
					)}
				>
					À planifier
				</span>
			)
		case 'holiday':
			return (
				<span
					className={cn(
						'flex size-full items-center justify-center text-[11px] text-muted-foreground italic',
					)}
				>
					Férié
				</span>
			)
		case 'absence':
			return <AbsenceFace half={half} scope={scope} types={types} />
		case 'inactive':
			return <span className={cn('block size-full', HATCH_INACTIVE)} />
	}
}

function ExtraRestFace({ label }: { label: string }) {
	return (
		<span className="flex size-full items-start justify-center bg-foreground/[0.06] px-1 pt-2">
			<span className="truncate font-medium text-foreground text-xs tabular-nums">
				{label}
			</span>
		</span>
	)
}

function ExtraBadges({ extras }: { extras: readonly ExtraPeriod[] }) {
	const totals = extraTotals(extras)
	if (totals.approved === 0 && totals.pending === 0) return null
	return (
		<span
			aria-hidden
			className="absolute right-1 bottom-1 flex items-center gap-0.5"
		>
			{totals.approved > 0 && (
				<span className={cn(EXTRA_BADGE, EXTRA_SOLID)}>
					+{formatDuration(totals.approved)}
				</span>
			)}
			{totals.pending > 0 && (
				<span className={cn(EXTRA_BADGE, EXTRA_PENDING)}>
					+{formatDuration(totals.pending)}
				</span>
			)}
		</span>
	)
}

function WeekCellFace({
	morning,
	afternoon,
	types,
	extras,
}: {
	morning: HalfDay
	afternoon: HalfDay
	types: TypeIndex
	extras: readonly ExtraPeriod[]
}) {
	if (sameHalf(morning, afternoon)) {
		const label =
			morning.kind === 'off' || morning.kind === 'holiday'
				? restExtraLabel(extras)
				: null
		if (label) return <ExtraRestFace label={label} />
		return <HalfFace half={morning} scope="day" types={types} />
	}
	return (
		<span className="flex size-full flex-col gap-0.5">
			<span className="min-h-0 flex-1 overflow-hidden rounded-[5px]">
				<HalfFace half={morning} scope="morning" types={types} />
			</span>
			<span className="min-h-0 flex-1 overflow-hidden rounded-[5px]">
				<HalfFace half={afternoon} scope="afternoon" types={types} />
			</span>
		</span>
	)
}

function dayTint(date: string, today: string, wholeHoliday: boolean): string {
	if (date === today) return TODAY_SOFT
	if (wholeHoliday) return HOLIDAY_FILL
	return isWeekend(date) ? WEEKEND_SOFT : ''
}

export function WeekGrid({
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
		<div className="min-h-0 overflow-auto rounded-lg border bg-card [scroll-padding:64px_0_48px_212px]">
			<table
				ref={roving.tableRef}
				className="w-full min-w-[58rem] table-fixed border-separate border-spacing-0"
			>
				<caption className="sr-only">
					Planning de la semaine : une ligne par membre, une colonne par jour.
				</caption>
				<colgroup>
					<col className="w-[212px]" />
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
									className={cn(
										'sticky top-0 z-20 border-b border-l bg-card p-0 align-top font-normal',
										isWeekend(date) && WEEKEND_SOLID,
										isToday && TODAY_SOLID,
									)}
								>
									<span className="flex flex-col items-center gap-0.5 px-1 py-2 leading-tight">
										<span className="text-[11px] text-muted-foreground uppercase tracking-wide">
											{WEEKDAY_SHORT[weekdayOf(date)]}
										</span>
										<span
											className={cn(
												'font-semibold text-base tabular-nums',
												isToday && 'text-state-today',
											)}
										>
											{Number(date.slice(8, 10))}
										</span>
										{holiday && (
											<span
												title={holiday}
												className="max-w-full truncate text-[10px] text-muted-foreground"
											>
												{holiday}
											</span>
										)}
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
								className="sticky left-0 z-10 border-r border-b bg-card px-3 py-2 text-left font-normal"
							>
								<MemberLabel member={member} />
							</th>
							{days.map((date, col) => {
								const status = resolveDay(state, member, date)
								const wholeHoliday =
									status.morning.kind === 'holiday' &&
									status.afternoon.kind === 'holiday'
								const extras = extrasOn(state, member.id, date)
								return (
									<td
										key={date}
										className={cn(
											'border-b border-l px-1 py-0.5',
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
											<WeekCellFace
												morning={status.morning}
												afternoon={status.afternoon}
												types={types}
												extras={extras}
											/>
											<ExtraBadges extras={extras} />
											{status.override && !status.variable && (
												<span
													aria-hidden
													className="absolute top-1 right-1 size-1.5 rounded-full bg-state-today ring-2 ring-card"
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
										'sticky bottom-0 z-20 border-t border-l bg-card py-2 text-center font-semibold text-sm tabular-nums',
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

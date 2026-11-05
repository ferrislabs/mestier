import { type KeyboardEvent, useState } from 'react'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { cn } from '#/lib/utils'
import { findType } from '#/pages/hr/lib/rh/absences'
import {
	type CellModel,
	cellLabel,
	extraBadge,
	type HalfModel,
} from '#/pages/hr/lib/rh/calendar-model'
import {
	EXTRA_BADGE,
	EXTRA_PENDING,
	EXTRA_SOLID,
	HATCH_INACTIVE,
	HATCH_PENDING,
	HOLIDAY_FILL,
	hueStyle,
	TYPE_BORDER,
} from '#/pages/hr/lib/rh/calendar-style'
import type { Member, RhState } from '#/pages/hr/lib/rh/types'
import { CalendarDayPopover } from '#/pages/hr/ui/rh/members/calendar-popover'
import { TYPE_TONE } from '#/pages/hr/ui/rh/shared/type-chip'

const BLOCK =
	'flex min-h-0 flex-1 items-center rounded-[4px] px-1.5 text-[11px] leading-tight'

function AbsenceBlock({
	state,
	half,
	label,
	showPending,
}: {
	state: RhState
	half: Extract<HalfModel, { kind: 'absence' }>
	label: string | null
	showPending: boolean
}) {
	const type = findType(state, half.typeId)
	return (
		<span
			style={hueStyle(type?.hue ?? 250)}
			className={cn(
				BLOCK,
				'flex-col items-start justify-center font-medium',
				TYPE_TONE,
				half.pending && `border border-dashed ${TYPE_BORDER} ${HATCH_PENDING}`,
			)}
		>
			<span className="w-full truncate">
				{label ?? type?.name ?? 'Absence'}
			</span>
			{showPending && half.pending && (
				<span className="font-normal text-[10px] opacity-80">à valider</span>
			)}
		</span>
	)
}

function HalfBlock({
	state,
	half,
	part,
	split,
}: {
	state: RhState
	half: HalfModel
	part: 'matin' | 'après-midi'
	split: boolean
}) {
	switch (half.kind) {
		case 'work':
			return (
				<span
					className={cn(
						BLOCK,
						'text-[10px] text-muted-foreground tabular-nums',
					)}
				>
					{half.range}
				</span>
			)
		case 'off':
			return half.label ? (
				<span
					className={cn(
						BLOCK,
						'bg-muted/70 font-medium text-[10px] text-muted-foreground',
					)}
				>
					<span className="truncate">{half.label}</span>
				</span>
			) : (
				<span className={cn(BLOCK, 'text-muted-foreground/50')}>Repos</span>
			)
		case 'unplanned':
			return (
				<span
					className={cn(
						BLOCK,
						'justify-center border border-muted-foreground/50 border-dashed text-[10px] text-muted-foreground',
					)}
				>
					À planifier
				</span>
			)
		case 'holiday':
			return (
				<span title={half.name} className={cn(BLOCK, 'text-muted-foreground')}>
					Férié
				</span>
			)
		case 'inactive':
			return <span aria-hidden className={cn(BLOCK, HATCH_INACTIVE)} />
		case 'absence':
			return (
				<AbsenceBlock
					state={state}
					half={half}
					label={split ? part : null}
					showPending={!split}
				/>
			)
	}
}

function CellBody({ state, cell }: { state: RhState; cell: CellModel }) {
	if (cell.split)
		return (
			<>
				<HalfBlock state={state} half={cell.morning} part="matin" split />
				<HalfBlock
					state={state}
					half={cell.afternoon}
					part="après-midi"
					split
				/>
			</>
		)
	if (cell.morning.kind === 'work')
		return (
			<span
				className={cn(
					BLOCK,
					'items-end text-[11px] text-muted-foreground tabular-nums',
				)}
			>
				{cell.compactRange}
			</span>
		)
	return (
		<HalfBlock state={state} half={cell.morning} part="matin" split={false} />
	)
}

export function CalendarCell({
	state,
	member,
	cell,
	isToday,
	inMonth,
	tabIndex,
	onFocusCell,
	onKeyDown,
}: {
	state: RhState
	member: Member
	cell: CellModel
	isToday: boolean
	inMonth: boolean
	tabIndex: number
	onFocusCell: () => void
	onKeyDown: (event: KeyboardEvent<HTMLButtonElement>) => void
}) {
	const [open, setOpen] = useState(false)
	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<button
					type="button"
					data-date={cell.date}
					aria-label={cellLabel(state, cell)}
					tabIndex={tabIndex}
					onFocus={onFocusCell}
					onKeyDown={onKeyDown}
					className={cn(
						'@container flex h-full min-h-[5.75rem] w-full flex-col items-stretch gap-1 p-1 text-left outline-none transition-colors hover:bg-accent/60 focus-visible:z-10 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset aria-expanded:bg-accent',
						cell.morning.kind === 'holiday' &&
							cell.afternoon.kind === 'holiday' &&
							HOLIDAY_FILL,
					)}
				>
					<span
						className={cn(
							'flex shrink-0 items-start justify-between gap-1',
							!inMonth && 'opacity-50',
						)}
					>
						<span className="flex items-center gap-1">
							<span
								className={cn(
									'flex h-5 min-w-5 items-center justify-center rounded-full px-1 font-semibold text-xs tabular-nums',
									isToday && 'bg-state-today text-card',
								)}
							>
								{Number(cell.date.slice(8, 10))}
							</span>
							{cell.overridden && (
								<span
									aria-hidden
									className="size-1.5 rounded-full bg-state-today ring-1 ring-card"
								/>
							)}
						</span>
						<span className="flex flex-wrap justify-end gap-0.5">
							{cell.extraApproved > 0 && (
								<span aria-hidden className={cn(EXTRA_BADGE, EXTRA_SOLID)}>
									{extraBadge(cell.extraApproved)}
								</span>
							)}
							{cell.extraPending > 0 && (
								<span aria-hidden className={cn(EXTRA_BADGE, EXTRA_PENDING)}>
									{extraBadge(cell.extraPending)}
								</span>
							)}
						</span>
					</span>
					<span
						className={cn(
							'flex min-h-0 flex-1 flex-col gap-0.5',
							!inMonth && 'opacity-50',
						)}
					>
						<CellBody state={state} cell={cell} />
					</span>
				</button>
			</PopoverTrigger>
			<PopoverContent
				align="start"
				collisionPadding={8}
				className="max-h-(--radix-popover-content-available-height) w-[21rem] overflow-y-auto p-0"
			>
				<CalendarDayPopover
					state={state}
					member={member}
					date={cell.date}
					onClose={() => setOpen(false)}
				/>
			</PopoverContent>
		</Popover>
	)
}

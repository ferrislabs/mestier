import { type ReactNode, useState } from 'react'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from '#/components/ui/tooltip'
import { formatLong } from '#/pages/hr/lib/rh/dates'
import { fullName } from '#/pages/hr/lib/rh/members'
import type { DayStatus } from '#/pages/hr/lib/rh/planning'
import {
	type AbsenceInitial,
	cellLabel,
	type ExtraInitial,
	extraLine,
	lineTitle,
	overrideNote,
	type TypeIndex,
	tooltipLines,
} from '#/pages/hr/lib/rh/planning-model'
import type { ExtraPeriod, Member } from '#/pages/hr/lib/rh/types'
import { DayPopoverBody } from '#/pages/hr/ui/rh/planning/day-popover'
import type { CellNav } from '#/pages/hr/ui/rh/planning/use-roving-cells'

export type DayCellProps = {
	member: Member
	status: DayStatus
	types: TypeIndex
	extras: readonly ExtraPeriod[]
	nav: CellNav
	className: string
	onRequestAbsence: (initial: AbsenceInitial) => void
	onRequestExtra: (initial: ExtraInitial) => void
	children: ReactNode
}

function TooltipBody({
	member,
	status,
	types,
	extras,
}: Pick<DayCellProps, 'member' | 'status' | 'types' | 'extras'>) {
	const note = overrideNote(status)
	return (
		<div className="grid gap-0.5">
			<p className="font-medium">
				{fullName(member)} · {formatLong(status.date)}
			</p>
			{tooltipLines(status, types, member).map((line) => (
				<p key={line.label}>
					{line.label === 'Journée'
						? line.text
						: `${lineTitle(line)} : ${line.text}`}
				</p>
			))}
			{extras.map((extra) => (
				<p key={extra.id}>Heures extra : {extraLine(extra)}</p>
			))}
			{note && <p className="opacity-80">{note}</p>}
		</div>
	)
}

export function DayCell({
	member,
	status,
	types,
	extras,
	nav,
	className,
	onRequestAbsence,
	onRequestExtra,
	children,
}: DayCellProps) {
	const [open, setOpen] = useState(false)
	const [tipOpen, setTipOpen] = useState(false)
	return (
		<Popover open={open} onOpenChange={setOpen}>
			<Tooltip
				open={!open && tipOpen}
				onOpenChange={setTipOpen}
				disableHoverableContent
			>
				<TooltipTrigger asChild>
					<PopoverTrigger asChild>
						<button
							type="button"
							aria-label={cellLabel(status, types, member, extras)}
							data-cell={nav.cell}
							tabIndex={nav.tabIndex}
							onFocus={nav.onFocus}
							onKeyDown={nav.onKeyDown}
							className={className}
						>
							{children}
						</button>
					</PopoverTrigger>
				</TooltipTrigger>
				<TooltipContent side="top" sideOffset={4}>
					<TooltipBody
						member={member}
						status={status}
						types={types}
						extras={extras}
					/>
				</TooltipContent>
			</Tooltip>
			<PopoverContent
				align="start"
				collisionPadding={8}
				className="max-h-(--radix-popover-content-available-height) w-[21rem] overflow-y-auto p-0"
			>
				<DayPopoverBody
					member={member}
					status={status}
					types={types}
					extras={extras}
					onRequestAbsence={onRequestAbsence}
					onRequestExtra={onRequestExtra}
					onClose={() => setOpen(false)}
				/>
			</PopoverContent>
		</Popover>
	)
}

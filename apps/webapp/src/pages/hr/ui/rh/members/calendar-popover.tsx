import { useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import { absenceDays, findType } from '#/pages/hr/lib/rh/absences'
import { dayDetails, type HalfDetail } from '#/pages/hr/lib/rh/calendar-model'
import { formatDays, formatRange } from '#/pages/hr/lib/rh/dates'
import {
	describeSpan,
	equivalentMinutes,
	extraMinutes,
	formatDuration,
	formatRate,
} from '#/pages/hr/lib/rh/extra'
import { overrideFailureOf } from '#/pages/hr/lib/rh/override-wire'
import { dayPlanOf } from '#/pages/hr/lib/rh/plan-draft'
import { formatWorkDay } from '#/pages/hr/lib/rh/schedule'
import type {
	ExtraPeriod,
	IsoDate,
	Member,
	RhState,
	WorkDay,
} from '#/pages/hr/lib/rh/types'
import { AbsenceStatusBadge } from '#/pages/hr/ui/rh/shared/absence-status-badge'
import { useRhOverrideActions } from '#/pages/hr/ui/rh/shared/rh-context'
import { TypeChip } from '#/pages/hr/ui/rh/shared/type-chip'

function HalfLine({
	state,
	member,
	detail,
	dayRange,
}: {
	state: RhState
	member: Member
	detail: HalfDetail
	dayRange: string | null
}) {
	const { model, absence } = detail
	const type = absence ? findType(state, absence.typeId) : undefined
	return (
		<div className="grid gap-1">
			<p className="text-muted-foreground text-xs">{detail.part}</p>
			{model.kind === 'work' && (
				<p className="tabular-nums">
					Au travail{' '}
					<span className="font-medium">
						{detail.part === 'Journée' && dayRange ? dayRange : model.range}
					</span>
				</p>
			)}
			{model.kind === 'off' && (
				<p className="text-muted-foreground">
					{model.label ? `${model.label} · séquence sans travail` : 'Repos'}
				</p>
			)}
			{model.kind === 'unplanned' && (
				<p className="text-muted-foreground">
					À planifier : aucun horaire n’est encore défini pour ce jour.
				</p>
			)}
			{model.kind === 'holiday' && (
				<p>
					Férié <span className="text-muted-foreground">· {model.name}</span>
				</p>
			)}
			{model.kind === 'inactive' && (
				<p className="text-muted-foreground">Hors contrat</p>
			)}
			{model.kind === 'absence' && (
				<div className="grid gap-1">
					<div className="flex flex-wrap items-center gap-2">
						<TypeChip type={type ?? { name: 'Absence', hue: 250 }} />
						{absence && <AbsenceStatusBadge status={absence.status} />}
					</div>
					{absence && (
						<p className="text-muted-foreground text-xs">
							{formatRange(absence.startDate, absence.endDate)}
							{type && ` · ${formatDays(absenceDays(absence, member, type))}`}
						</p>
					)}
				</div>
			)}
		</div>
	)
}

function ExtraLine({ extra }: { extra: ExtraPeriod }) {
	return (
		<li
			className={cn(
				'grid gap-0.5 rounded-md border bg-muted/40 px-2.5 py-1.5',
				extra.status === 'pending' && 'border-dashed',
			)}
		>
			<div className="flex items-center justify-between gap-2">
				<span className="font-medium tabular-nums">
					{describeSpan(extra)}
					<span className="font-normal text-muted-foreground">
						{' '}
						· {formatDuration(extraMinutes(extra))}
					</span>
				</span>
				<AbsenceStatusBadge status={extra.status} />
			</div>
			<p className="text-muted-foreground text-xs">
				{formatRate(extra.rate)}
				{extra.rate > 0 &&
					` · équivalent payé ${formatDuration(equivalentMinutes(extra))}`}
			</p>
			<p className="line-clamp-2 text-muted-foreground text-xs">
				{extra.reason}
			</p>
		</li>
	)
}

function PlanningActions({
	member,
	date,
	defaultDay,
	planned,
	current,
	onClose,
}: {
	member: Member
	date: IsoDate
	defaultDay: WorkDay
	planned: boolean
	current: 'work' | 'off' | null
	onClose: () => void
}) {
	const overrides = useRhOverrideActions()
	const [refusal, setRefusal] = useState<string | null>(null)
	async function commit(write: () => Promise<void>) {
		setRefusal(null)
		try {
			await write()
			onClose()
		} catch (error) {
			setRefusal(overrideFailureOf(error).message)
		}
	}
	function plan(kind: 'work' | 'off') {
		void commit(() =>
			overrides.plan(dayPlanOf(member.id, date, kind, defaultDay)),
		)
	}
	return (
		<section aria-label="Planification du jour" className="grid gap-1.5">
			<p className="font-medium text-xs">Planification du jour</p>
			<div className="flex flex-wrap gap-1.5">
				<Button
					type="button"
					size="xs"
					variant={current === 'work' ? 'default' : 'outline'}
					aria-pressed={current === 'work'}
					onClick={() => plan('work')}
				>
					Travaillé
				</Button>
				<Button
					type="button"
					size="xs"
					variant={current === 'off' ? 'default' : 'outline'}
					aria-pressed={current === 'off'}
					onClick={() => plan('off')}
				>
					Repos
				</Button>
				{planned && (
					<Button
						type="button"
						size="xs"
						variant="ghost"
						onClick={() => {
							void commit(() => overrides.clear(member.id, date))
						}}
					>
						Retirer du planning
					</Button>
				)}
			</div>
			{refusal && (
				<p role="alert" className="text-state-alert text-xs">
					{refusal}
				</p>
			)}
			<p className="text-[11px] text-muted-foreground">
				Travaillé reprend les horaires par défaut ({formatWorkDay(defaultDay)}).
			</p>
		</section>
	)
}

export function CalendarDayPopover({
	state,
	member,
	date,
	onClose,
}: {
	state: RhState
	member: Member
	date: IsoDate
	onClose: () => void
}) {
	const details = dayDetails(state, member, date)
	const first = details.halves[0].model
	return (
		<div className="grid gap-3 p-3 text-sm">
			<div>
				<p className="font-medium first-letter:uppercase">{details.title}</p>
				{details.sequence && (
					<p className="text-muted-foreground text-xs">
						Séquence : {details.sequence}
					</p>
				)}
			</div>
			{details.halves.map((detail) => (
				<HalfLine
					key={detail.part}
					state={state}
					member={member}
					detail={detail}
					dayRange={details.dayRange}
				/>
			))}
			{details.lunch && (
				<p className="text-muted-foreground text-xs tabular-nums">
					Pause déjeuner {details.lunch}
				</p>
			)}
			{details.note && (
				<p className="rounded-md bg-state-today/10 px-2.5 py-1.5 text-xs">
					{details.note}
				</p>
			)}
			{details.variable && member.status === 'active' && (
				<RequirePermission permission="MANAGE_PLANNING">
					<PlanningActions
						member={member}
						date={date}
						defaultDay={details.variable.defaultDay}
						planned={details.variable.planned}
						current={
							details.variable.planned && first.kind !== 'unplanned'
								? first.kind === 'work'
									? 'work'
									: first.kind === 'off'
										? 'off'
										: null
								: null
						}
						onClose={onClose}
					/>
				</RequirePermission>
			)}
			{details.extras.length > 0 && (
				<section aria-label="Heures extra" className="grid gap-1.5">
					<p className="font-medium text-xs">Heures extra</p>
					<ul className="grid gap-1.5">
						{details.extras.map((extra) => (
							<ExtraLine key={extra.id} extra={extra} />
						))}
					</ul>
				</section>
			)}
		</div>
	)
}

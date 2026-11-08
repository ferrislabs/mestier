import { AlarmClockPlus, CalendarPlus, RotateCcw } from 'lucide-react'
import { useId, useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Separator } from '#/components/ui/separator'
import { Switch } from '#/components/ui/switch'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import { cn } from '#/lib/utils'
import { formatLong } from '#/pages/hr/lib/rh/dates'
import {
	describeSpan,
	equivalentMinutes,
	extraMinutes,
	formatDuration,
	formatRate,
} from '#/pages/hr/lib/rh/extra'
import { overrideFailureOf } from '#/pages/hr/lib/rh/override-wire'
import type { DayStatus, HalfDay } from '#/pages/hr/lib/rh/planning'
import {
	type AbsenceInitial,
	dayFormError,
	dayFormFault,
	dayLines,
	type ExtraInitial,
	initialDayForm,
	isDayFormDirty,
	overrideNote,
	type TypeIndex,
	toOverride,
	typeInfo,
} from '#/pages/hr/lib/rh/planning-model'
import type { ExtraPeriod, Member } from '#/pages/hr/lib/rh/types'
import { AbsenceStatusBadge } from '#/pages/hr/ui/rh/shared/absence-status-badge'
import { useRhOverrideActions } from '#/pages/hr/ui/rh/shared/rh-context'
import { TypeDot } from '#/pages/hr/ui/rh/shared/type-chip'

type DayPopoverProps = {
	member: Member
	status: DayStatus
	types: TypeIndex
	extras: readonly ExtraPeriod[]
	onRequestAbsence: (initial: AbsenceInitial) => void
	onRequestExtra: (initial: ExtraInitial) => void
	onClose: () => void
}

function ExtraList({ extras }: { extras: readonly ExtraPeriod[] }) {
	return (
		<section aria-label="Heures extra" className="grid gap-1.5">
			<p className="font-medium text-xs">Heures extra</p>
			<ul className="grid gap-1.5">
				{extras.map((extra) => (
					<li
						key={extra.id}
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
				))}
			</ul>
		</section>
	)
}

function HalfDot({ half, types }: { half: HalfDay; types: TypeIndex }) {
	if (half.kind === 'absence')
		return <TypeDot hue={typeInfo(types, half.typeId).hue} />
	return (
		<span
			aria-hidden
			className={cn(
				'inline-block size-2 shrink-0 rounded-full',
				half.kind === 'work' ? 'bg-muted-foreground/45' : 'bg-border',
			)}
		/>
	)
}

function TimeRange({
	startLabel,
	endLabel,
	start,
	end,
	invalid,
	describedBy,
	onStartChange,
	onEndChange,
}: {
	startLabel: string
	endLabel: string
	start: string
	end: string
	invalid: boolean
	describedBy: string | undefined
	onStartChange: (value: string) => void
	onEndChange: (value: string) => void
}) {
	return (
		<div className="flex items-center gap-2 text-muted-foreground text-xs">
			<span>de</span>
			<Input
				type="time"
				value={start}
				onChange={(event) => onStartChange(event.target.value)}
				aria-label={startLabel}
				aria-invalid={invalid}
				aria-describedby={invalid ? describedBy : undefined}
				className="h-8 flex-1 px-2"
			/>
			<span>à</span>
			<Input
				type="time"
				value={end}
				onChange={(event) => onEndChange(event.target.value)}
				aria-label={endLabel}
				aria-invalid={invalid}
				aria-describedby={invalid ? describedBy : undefined}
				className="h-8 flex-1 px-2"
			/>
		</div>
	)
}

function DayEditor({
	member,
	status,
	onClose,
}: Pick<DayPopoverProps, 'member' | 'status' | 'onClose'>) {
	const overrides = useRhOverrideActions()
	const [refusal, setRefusal] = useState<string | null>(null)
	const [pending, setPending] = useState(false)
	const id = useId()
	const initial = initialDayForm(member, status.date, status.override)
	const [form, setForm] = useState(initial)
	const error = dayFormError(form)
	const fault = dayFormFault(form)
	const valid = error === null
	const planning = status.variable
	const dirty =
		(planning && status.override === null) || isDayFormDirty(initial, form)

	async function commit(write: () => Promise<void>) {
		setPending(true)
		setRefusal(null)
		try {
			await write()
			onClose()
		} catch (error) {
			setRefusal(overrideFailureOf(error).message)
		} finally {
			setPending(false)
		}
	}

	function save() {
		void commit(() => overrides.put(toOverride(member.id, status.date, form)))
	}

	function restore() {
		void commit(() => overrides.clear(member.id, status.date))
	}

	return (
		<form
			className="grid gap-2.5 p-3"
			onSubmit={(event) => {
				event.preventDefault()
				if (valid && dirty && !pending) save()
			}}
		>
			<p className="font-medium text-xs">
				{planning ? 'Planifier ce jour' : 'Modifier ce jour'}
			</p>
			<ToggleGroup
				type="single"
				variant="outline"
				size="sm"
				value={form.kind}
				onValueChange={(kind) => {
					if (kind === 'work' || kind === 'off') setForm({ ...form, kind })
				}}
				aria-label="État du jour"
				className="w-full"
			>
				<ToggleGroupItem value="work" className="flex-1">
					Travaille
				</ToggleGroupItem>
				<ToggleGroupItem value="off" className="flex-1">
					Repos
				</ToggleGroupItem>
			</ToggleGroup>
			{form.kind === 'work' && (
				<>
					<TimeRange
						startLabel="Heure de début"
						endLabel="Heure de fin"
						start={form.start}
						end={form.end}
						invalid={fault === 'day'}
						describedBy={`${id}-error`}
						onStartChange={(start) => setForm({ ...form, start })}
						onEndChange={(end) => setForm({ ...form, end })}
					/>
					<div className="flex items-center gap-2">
						<Switch
							id={`${id}-lunch`}
							checked={form.lunchOn}
							onCheckedChange={(lunchOn) => setForm({ ...form, lunchOn })}
						/>
						<Label htmlFor={`${id}-lunch`} className="font-normal text-xs">
							Pause déjeuner
						</Label>
					</div>
					{form.lunchOn && (
						<TimeRange
							startLabel="Début de la pause"
							endLabel="Fin de la pause"
							start={form.lunchStart}
							end={form.lunchEnd}
							invalid={fault === 'lunch'}
							describedBy={`${id}-error`}
							onStartChange={(lunchStart) => setForm({ ...form, lunchStart })}
							onEndChange={(lunchEnd) => setForm({ ...form, lunchEnd })}
						/>
					)}
				</>
			)}
			{error && (
				<p id={`${id}-error`} className="text-state-alert text-xs">
					{error}
				</p>
			)}
			{refusal && (
				<p role="alert" className="text-state-alert text-xs">
					{refusal}
				</p>
			)}
			<Input
				value={form.note}
				onChange={(event) => setForm({ ...form, note: event.target.value })}
				placeholder="Note (facultatif)"
				aria-label="Note"
				maxLength={120}
				className="h-8"
			/>
			<div className="flex flex-wrap items-center gap-2">
				<Button type="submit" size="sm" disabled={!valid || !dirty || pending}>
					Enregistrer
				</Button>
				{status.override && (
					<Button
						type="button"
						variant="ghost"
						size="sm"
						disabled={pending}
						onClick={restore}
					>
						<RotateCcw />
						{planning ? 'Retirer du planning' : 'Rétablir la semaine type'}
					</Button>
				)}
			</div>
		</form>
	)
}

export function DayPopoverBody({
	member,
	status,
	types,
	extras,
	onRequestAbsence,
	onRequestExtra,
	onClose,
}: DayPopoverProps) {
	const lines = dayLines(status, types, member)
	const halves = [status.morning, status.afternoon]
	const inactive = status.morning.kind === 'inactive'
	return (
		<div className="flex flex-col text-sm">
			<div className="grid gap-2.5 p-3">
				<div>
					<p className="font-semibold">
						{member.firstName} · {formatLong(status.date)}
					</p>
					<p className="text-muted-foreground text-xs">{member.jobTitle}</p>
				</div>
				<ul className="grid gap-1.5">
					{lines.map((line, index) => {
						const half = halves[index]
						return (
							<li key={line.label} className="flex items-baseline gap-3">
								<span className="w-[4.5rem] shrink-0 text-muted-foreground text-xs">
									{line.label}
								</span>
								<span className="flex min-w-0 items-center gap-1.5">
									<HalfDot half={half} types={types} />
									{line.text}
								</span>
							</li>
						)
					})}
				</ul>
				{overrideNote(status) && (
					<p className="text-muted-foreground text-xs">
						{overrideNote(status)}
					</p>
				)}
				{extras.length > 0 && <ExtraList extras={extras} />}
				{!inactive && (
					<div className="flex flex-wrap gap-2">
						<RequirePermission permission="MANAGE_REFERENCE">
							<Button
								type="button"
								variant="outline"
								size="sm"
								onClick={() => {
									onClose()
									onRequestAbsence({
										memberId: member.id,
										startDate: status.date,
									})
								}}
							>
								<CalendarPlus />
								Poser une absence
							</Button>
						</RequirePermission>
						<Button
							type="button"
							variant="outline"
							size="sm"
							onClick={() => {
								onClose()
								onRequestExtra({ memberId: member.id, date: status.date })
							}}
						>
							<AlarmClockPlus />
							Déclarer des heures extra
						</Button>
					</div>
				)}
			</div>
			{!inactive && (
				<RequirePermission permission="MANAGE_PLANNING">
					<Separator />
					<DayEditor member={member} status={status} onClose={onClose} />
				</RequirePermission>
			)}
		</div>
	)
}

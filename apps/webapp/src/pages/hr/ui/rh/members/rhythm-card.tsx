import { Pencil } from 'lucide-react'
import { type FormEvent, useId, useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from '#/components/ui/dialog'
import { contractDraftOf } from '#/pages/hr/lib/rh/contract-draft'
import {
	describeCycle,
	validateSchedule,
	weekHours,
} from '#/pages/hr/lib/rh/contract-schedule'
import { CONTRACT_LABELS, currentContract } from '#/pages/hr/lib/rh/contracts'
import { formatMonth, WEEKDAY_LABELS } from '#/pages/hr/lib/rh/dates'
import {
	dayHoursLabel,
	formatDate,
	scheduleCaption,
} from '#/pages/hr/lib/rh/format'
import { contractFailureOf } from '#/pages/hr/lib/rh/member-wire'
import { formatHours, fullName } from '#/pages/hr/lib/rh/members'
import { blockLengthLabel } from '#/pages/hr/lib/rh/rotation'
import { formatWorkDay } from '#/pages/hr/lib/rh/schedule'
import {
	previewLines,
	type RotationSchedule,
	scheduleVolume,
	variableMonthSummary,
	weekSummary,
} from '#/pages/hr/lib/rh/schedule-mode'
import type {
	Contract,
	ContractSchedule,
	IsoDate,
	Member,
	RhState,
	WeekSchedule,
	WorkDay,
} from '#/pages/hr/lib/rh/types'
import { DetailCard } from '#/pages/hr/ui/rh/members/detail-card'
import { PlanDaysDialog } from '#/pages/hr/ui/rh/members/plan-days-dialog'
import { ScheduleModeEditor } from '#/pages/hr/ui/rh/members/schedule-mode-editor'
import { useRhContractActions } from '#/pages/hr/ui/rh/shared/rh-context'

export const CALENDAR_ANCHOR = 'member-calendar'

function RhythmForm({
	member,
	contract,
	today,
	onClose,
}: {
	member: Member
	contract: Contract
	today: IsoDate
	onClose: () => void
}) {
	const actions = useRhContractActions()
	const idPrefix = useId()
	const [schedule, setDraft] = useState<ContractSchedule>(contract.schedule)
	const [failure, setFailure] = useState<string | null>(null)
	const [isPending, setIsPending] = useState(false)
	const errors = validateSchedule(schedule)
	const blocked = errors.length > 0
	const extra = errors.filter(
		(message) =>
			!message.startsWith('Renseigne la date de début du cycle') &&
			!message.startsWith('Pour des séquences en mois') &&
			!message.startsWith('Ajoute au moins') &&
			!message.startsWith('Un cycle compte') &&
			!message.startsWith('Une séquence dure'),
	)

	async function submit(event: FormEvent) {
		event.preventDefault()
		if (blocked || isPending) return
		setIsPending(true)
		setFailure(null)
		try {
			await actions.update(member.id, contract.id, {
				...contractDraftOf(contract),
				schedule,
			})
			onClose()
		} catch (error) {
			setFailure(contractFailureOf(error).message)
		} finally {
			setIsPending(false)
		}
	}

	return (
		<form onSubmit={submit} noValidate className="flex min-h-0 flex-1 flex-col">
			<DialogHeader className="px-6 pt-6 pb-4">
				<DialogTitle>Rythme de travail</DialogTitle>
				<DialogDescription>
					Jours et horaires de {fullName(member)} pour son contrat{' '}
					{CONTRACT_LABELS[contract.type]}, repris dans le planning.
				</DialogDescription>
			</DialogHeader>
			<div className="grid min-h-0 flex-1 gap-4 overflow-y-auto border-t px-6 py-5">
				<ScheduleModeEditor
					idPrefix={idPrefix}
					value={schedule}
					onChange={(next) => {
						setFailure(null)
						setDraft(next)
					}}
					contractStart={contract.startDate}
					today={today}
				/>
				{failure && (
					<p role="alert" className="text-[13px] text-state-alert">
						{failure}
					</p>
				)}
				{extra.length > 0 && (
					<ul className="grid gap-1 text-[13px] text-state-alert">
						{extra.map((message) => (
							<li key={message}>{message}</li>
						))}
					</ul>
				)}
			</div>
			<DialogFooter className="border-t px-6 py-3">
				<DialogClose asChild>
					<Button type="button" variant="outline">
						Annuler
					</Button>
				</DialogClose>
				<Button type="submit" disabled={blocked || isPending}>
					Enregistrer
				</Button>
			</DialogFooter>
		</form>
	)
}

function WeekLines({ week }: { week: WeekSchedule }) {
	return (
		<dl className="grid gap-1.5 text-[13px]">
			{week.map((day, index) => (
				<div
					key={WEEKDAY_LABELS[index]}
					className="flex items-baseline justify-between gap-4"
				>
					<dt className={day ? undefined : 'text-muted-foreground'}>
						{WEEKDAY_LABELS[index]}
					</dt>
					<dd className="tabular text-right">
						{day ? (
							<>
								{formatWorkDay(day)}
								<span className="ml-2 inline-block w-12 text-muted-foreground">
									{dayHoursLabel(day)}
								</span>
							</>
						) : (
							<span className="text-muted-foreground">Repos</span>
						)}
					</dd>
				</div>
			))}
		</dl>
	)
}

function WeeklyView({ week }: { week: WeekSchedule }) {
	return (
		<>
			<WeekLines week={week} />
			<p className="mt-3 flex items-baseline justify-between border-t pt-2.5 text-[13px]">
				<span className="text-muted-foreground">Total hebdomadaire net</span>
				<span className="tabular font-semibold">
					{formatHours(weekHours(week))}
				</span>
			</p>
		</>
	)
}

function RotationView({
	schedule,
	contract,
	today,
}: {
	schedule: RotationSchedule
	contract: Contract
	today: IsoDate
}) {
	const from = contract.startDate > today ? contract.startDate : today
	return (
		<div className="grid gap-4">
			<div className="grid gap-0.5">
				<p className="font-medium text-[13px]">
					{describeCycle(schedule.blocks)}
				</p>
				<p className="text-[13px] text-muted-foreground">
					{scheduleVolume(schedule, true)}
				</p>
				<p className="text-[12px] text-muted-foreground">
					Cycle démarré le {formatDate(schedule.anchor)}
				</p>
			</div>
			<section aria-label="Séquences du cycle" className="grid gap-1.5">
				<h4 className="text-[12px] text-muted-foreground">Séquences</h4>
				<ul className="grid divide-y rounded-lg border">
					{schedule.blocks.map((block, index) => (
						<li
							key={`${block.label}-${block.length}-${block.unit}-${weekSummary(block.week)}-${index.toString()}`}
						>
							<details className="group">
								<summary className="flex cursor-pointer list-none flex-wrap items-baseline gap-x-3 gap-y-0.5 px-3 py-2 text-[13px] outline-none focus-visible:bg-accent">
									<span className="font-medium">
										{block.label.trim() || `Séquence ${index + 1}`}
									</span>
									<span className="text-muted-foreground">
										{blockLengthLabel(block)}
									</span>
									<span className="ml-auto text-muted-foreground">
										{weekSummary(block.week)}
									</span>
								</summary>
								<div className="border-t px-3 py-3">
									{block.week.every((day) => day === null) ? (
										<p className="text-[13px] text-muted-foreground">
											Pas de travail pendant cette séquence.
										</p>
									) : (
										<WeekLines week={block.week} />
									)}
								</div>
							</details>
						</li>
					))}
				</ul>
			</section>
			<section aria-label="Prochaines séquences" className="grid gap-1.5">
				<h4 className="text-[12px] text-muted-foreground">
					Prochaines séquences
				</h4>
				<ol className="grid divide-y rounded-lg border">
					{previewLines(schedule, from, 6).map((line, position) => (
						<li
							key={line.key}
							className="flex flex-wrap items-center gap-x-3 gap-y-0.5 px-3 py-2 text-[13px]"
						>
							<span className="min-w-24 font-medium">{line.label}</span>
							<span className="tabular text-muted-foreground">
								{formatDate(line.start)} au {formatDate(line.end)}
							</span>
							<span className="ml-auto text-muted-foreground">
								{line.summary}
							</span>
							{position === 0 && from === today && (
								<span className="inline-flex h-5 items-center rounded-sm bg-state-progress/12 px-1.5 font-medium text-[11px] text-state-progress">
									En cours
								</span>
							)}
						</li>
					))}
				</ol>
			</section>
		</div>
	)
}

function VariableView({
	state,
	member,
	today,
	month,
	defaultDay,
}: {
	state: RhState
	member: Member
	today: IsoDate
	month: IsoDate
	defaultDay: WorkDay
}) {
	const summary = variableMonthSummary(state, member.id, today)
	const hours = formatHours(Math.round((summary.minutes / 60) * 10) / 10)
	return (
		<div className="grid gap-3">
			<p className="rounded-md bg-muted/60 px-3 py-2 text-[13px]">
				Pas de semaine type : les jours travaillés se planifient dans le
				calendrier, jour par jour ou sur une période.
			</p>
			<dl className="grid gap-1.5 text-[13px]">
				<div className="flex items-baseline justify-between gap-4">
					<dt className="text-muted-foreground">Horaires par défaut</dt>
					<dd className="tabular">{formatWorkDay(defaultDay)}</dd>
				</div>
				<div className="flex items-baseline justify-between gap-4">
					<dt className="text-muted-foreground">
						Planifié en {formatMonth(today)}
					</dt>
					<dd className="tabular">
						{summary.days === 0
							? 'Aucun jour'
							: `${summary.days} ${summary.days > 1 ? 'jours' : 'jour'} · ${hours}`}
					</dd>
				</div>
			</dl>
			<div className="flex flex-wrap items-center gap-2 border-t pt-3">
				<PlanDaysDialog
					member={member}
					today={today}
					month={month}
					trigger={<Button size="sm">Planifier des jours</Button>}
				/>
				<Button
					type="button"
					variant="outline"
					size="sm"
					onClick={() =>
						document
							.getElementById(CALENDAR_ANCHOR)
							?.scrollIntoView({ behavior: 'smooth', block: 'start' })
					}
				>
					Voir le calendrier
				</Button>
			</div>
		</div>
	)
}

export function RhythmCard({
	state,
	member,
	today,
	month,
}: {
	state: RhState
	member: Member
	today: IsoDate
	month: IsoDate
}) {
	const [editing, setEditing] = useState(false)
	const contract = currentContract(member, today)
	const schedule = contract.schedule
	return (
		<Dialog open={editing} onOpenChange={setEditing}>
			<DetailCard
				title="Rythme de travail"
				action={
					member.contracts.length > 0 ? (
						<RequirePermission permission="MANAGE_MEMBERS">
							<DialogTrigger asChild>
								<Button variant="outline" size="sm">
									<Pencil aria-hidden /> Modifier le rythme
								</Button>
							</DialogTrigger>
						</RequirePermission>
					) : undefined
				}
			>
				<p className="mb-3 text-[12px] text-muted-foreground">
					{scheduleCaption(contract, today)}
				</p>
				{schedule.kind === 'weekly' && <WeeklyView week={schedule.week} />}
				{schedule.kind === 'rotation' && (
					<RotationView schedule={schedule} contract={contract} today={today} />
				)}
				{schedule.kind === 'variable' && (
					<VariableView
						state={state}
						member={member}
						today={today}
						month={month}
						defaultDay={schedule.defaultDay}
					/>
				)}
			</DetailCard>
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-2xl">
				<RhythmForm
					member={member}
					contract={contract}
					today={today}
					onClose={() => setEditing(false)}
				/>
			</DialogContent>
		</Dialog>
	)
}

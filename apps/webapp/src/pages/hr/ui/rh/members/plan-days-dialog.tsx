import { type FormEvent, type ReactNode, useId, useState } from 'react'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import { currentContract } from '#/pages/hr/lib/rh/contracts'
import { WEEKDAY_LABELS, WEEKDAY_SHORT } from '#/pages/hr/lib/rh/dates'
import {
	clearPlanning,
	type DayPlan,
	planDays,
	validatePlan,
} from '#/pages/hr/lib/rh/day-planning'
import { formatDate } from '#/pages/hr/lib/rh/format'
import { fullName } from '#/pages/hr/lib/rh/members'
import {
	countPlanned,
	defaultPlanPeriod,
	emptyPlan,
	fromWeekdays,
	planSummary,
	toWeekdays,
} from '#/pages/hr/lib/rh/plan-draft'
import { DEFAULT_LUNCH, validateWorkDay } from '#/pages/hr/lib/rh/schedule'
import type {
	DayOverrideKind,
	IsoDate,
	Member,
	WorkDay,
} from '#/pages/hr/lib/rh/types'
import { WorkDayFields } from '#/pages/hr/ui/rh/members/work-day-fields'
import { useRhState, useRhUpdate } from '#/pages/hr/ui/rh/shared/rh-context'

const FALLBACK_DAY: WorkDay = {
	start: '08:00',
	end: '17:00',
	lunch: DEFAULT_LUNCH,
}

function planDay(plan: DayPlan): WorkDay {
	return { start: plan.start, end: plan.end, lunch: plan.lunch }
}

export function PlanDaysDialog({
	member,
	today,
	month,
	trigger,
}: {
	member: Member
	today: IsoDate
	month: IsoDate
	trigger: ReactNode
}) {
	const [open, setOpen] = useState(false)
	return (
		<Dialog open={open} onOpenChange={setOpen}>
			<DialogTrigger asChild>{trigger}</DialogTrigger>
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-xl">
				<PlanForm
					member={member}
					today={today}
					month={month}
					onClose={() => setOpen(false)}
				/>
			</DialogContent>
		</Dialog>
	)
}

function PlanForm({
	member,
	today,
	month,
	onClose,
}: {
	member: Member
	today: IsoDate
	month: IsoDate
	onClose: () => void
}) {
	const updateRh = useRhUpdate()
	const state = useRhState()
	const baseId = useId()
	const schedule = currentContract(member, today).schedule
	const defaultDay =
		schedule.kind === 'variable' ? schedule.defaultDay : FALLBACK_DAY
	const [plan, setPlan] = useState<DayPlan>(() =>
		emptyPlan(member.id, defaultPlanPeriod(month, today), defaultDay),
	)
	const [confirmClear, setConfirmClear] = useState(false)
	const errors = validatePlan(state, plan)
	const dayError = plan.kind === 'work' ? validateWorkDay(planDay(plan)) : null
	const blocked = errors.length > 0
	const planned = countPlanned(state, member.id, plan.startDate, plan.endDate)
	const id = (field: string) => `${baseId}-${field}`

	function patch(next: Partial<DayPlan>) {
		setPlan((current) => ({ ...current, ...next }))
		setConfirmClear(false)
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		if (blocked) return
		updateRh((current, context) => planDays(current, context, plan))
		onClose()
	}

	function clear() {
		updateRh((current) =>
			clearPlanning(current, member.id, plan.startDate, plan.endDate),
		)
		onClose()
	}

	return (
		<form onSubmit={submit} noValidate className="flex min-h-0 flex-1 flex-col">
			<DialogHeader className="px-6 pt-6 pb-4">
				<DialogTitle>Planifier des jours</DialogTitle>
				<DialogDescription>
					Définis les jours travaillés ou de repos de {fullName(member)} sur une
					période. Les jours déjà planifiés sont remplacés.
				</DialogDescription>
			</DialogHeader>
			<div className="grid min-h-0 flex-1 gap-5 overflow-y-auto border-t px-6 py-5">
				<div className="grid gap-3 sm:grid-cols-2">
					<FormField id={id('start')} label="Du" required>
						<Input
							id={id('start')}
							type="date"
							value={plan.startDate}
							onChange={(event) => patch({ startDate: event.target.value })}
						/>
					</FormField>
					<FormField id={id('end')} label="Au" required>
						<Input
							id={id('end')}
							type="date"
							value={plan.endDate}
							onChange={(event) => patch({ endDate: event.target.value })}
						/>
					</FormField>
				</div>
				<div className="grid gap-1.5">
					<p className="font-medium text-sm">Jours de la semaine</p>
					<ToggleGroup
						type="multiple"
						variant="outline"
						size="sm"
						value={fromWeekdays(plan.weekdays)}
						onValueChange={(values) => patch({ weekdays: toWeekdays(values) })}
						aria-label="Jours de la semaine"
						className="flex-wrap"
					>
						{WEEKDAY_SHORT.map((label, index) => (
							<ToggleGroupItem
								key={label}
								value={String(index)}
								aria-label={WEEKDAY_LABELS[index]}
							>
								{label}
							</ToggleGroupItem>
						))}
					</ToggleGroup>
				</div>
				<div className="grid gap-1.5">
					<p className="font-medium text-sm">Type de jour</p>
					<ToggleGroup
						type="single"
						variant="outline"
						size="sm"
						value={plan.kind}
						onValueChange={(kind) =>
							kind && patch({ kind: kind as DayOverrideKind })
						}
						aria-label="Type de jour"
					>
						<ToggleGroupItem value="work">Travaillé</ToggleGroupItem>
						<ToggleGroupItem value="off">Repos</ToggleGroupItem>
					</ToggleGroup>
				</div>
				{plan.kind === 'work' && (
					<div className="grid gap-1.5">
						<p className="font-medium text-sm">Horaires</p>
						<WorkDayFields
							idPrefix={id('day')}
							value={planDay(plan)}
							onChange={(day) =>
								patch({
									start: day.start,
									end: day.end,
									lunch: day.lunch ?? null,
								})
							}
						/>
					</div>
				)}
				<FormField id={id('note')} label="Note">
					<Input
						id={id('note')}
						value={plan.note}
						onChange={(event) => patch({ note: event.target.value })}
						placeholder="Mission, chantier, remplacement…"
						autoComplete="off"
					/>
				</FormField>
				<div className="grid gap-1 rounded-md bg-muted/60 px-3 py-2.5 text-[13px]">
					<output>{errors.length === 0 ? planSummary(state, plan) : ''}</output>
					{errors
						.filter((message) => message !== dayError)
						.map((message) => (
							<p key={message} className="text-state-alert">
								{message}
							</p>
						))}
				</div>
				<section
					aria-label="Effacer la planification"
					className="grid gap-2 border-t pt-4"
				>
					{confirmClear ? (
						<div className="grid gap-2">
							<p className="text-[13px]">
								{planned === 1
									? '1 jour planifié sera effacé'
									: `${planned} jours planifiés seront effacés`}{' '}
								du {formatDate(plan.startDate)} au {formatDate(plan.endDate)}.
							</p>
							<div className="flex gap-2">
								<Button
									type="button"
									variant="destructive"
									size="sm"
									onClick={clear}
								>
									Confirmer l’effacement
								</Button>
								<Button
									type="button"
									variant="outline"
									size="sm"
									onClick={() => setConfirmClear(false)}
								>
									Annuler
								</Button>
							</div>
						</div>
					) : (
						<div className="flex flex-wrap items-center gap-3">
							<Button
								type="button"
								variant="outline"
								size="sm"
								disabled={planned === 0 || !plan.startDate || !plan.endDate}
								onClick={() => setConfirmClear(true)}
							>
								Effacer la planification de cette période
							</Button>
							{planned === 0 && (
								<span className="text-[12px] text-muted-foreground">
									Aucun jour planifié sur cette période.
								</span>
							)}
						</div>
					)}
				</section>
			</div>
			<div className="flex items-center justify-end gap-2 border-t px-6 py-3">
				<DialogClose asChild>
					<Button type="button" variant="outline">
						Annuler
					</Button>
				</DialogClose>
				<Button type="submit" disabled={blocked}>
					Planifier
				</Button>
			</div>
		</form>
	)
}

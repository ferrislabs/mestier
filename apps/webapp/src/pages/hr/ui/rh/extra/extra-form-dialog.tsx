import { CircleAlert, Info } from 'lucide-react'
import { type FormEvent, useId, useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { isIsoDate, todayIso } from '#/pages/hr/lib/rh/dates'
import { createExtra, validateExtra } from '#/pages/hr/lib/rh/extra'
import {
	buildExtraDraft,
	CUSTOM_RATE,
	type ExtraForm,
	isRateInvalid,
	type Majoration,
	resolveDate,
	suggestedMajoration,
} from '#/pages/hr/lib/rh/extra-view'
import { fullName } from '#/pages/hr/lib/rh/members'
import { activePlanningMembers } from '#/pages/hr/lib/rh/planning'
import { DEFAULT_LUNCH } from '#/pages/hr/lib/rh/schedule'
import type { IsoDate } from '#/pages/hr/lib/rh/types'
import { ExtraRecap } from '#/pages/hr/ui/rh/extra/extra-recap'
import { MajorationFields } from '#/pages/hr/ui/rh/extra/majoration-fields'
import { SwitchRow } from '#/pages/hr/ui/rh/extra/switch-row'
import { MemberAvatar } from '#/pages/hr/ui/rh/shared/member-avatar'
import {
	useRhActorId,
	useRhState,
	useRhUpdate,
} from '#/pages/hr/ui/rh/shared/rh-context'

export type ExtraFormInitial = { memberId?: string; date?: IsoDate }

export type ExtraFormDialogProps = {
	open: boolean
	onOpenChange: (open: boolean) => void
	initial?: ExtraFormInitial
}

export function ExtraFormDialog({
	open,
	onOpenChange,
	initial,
}: ExtraFormDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[calc(100dvh-2rem)] overflow-y-auto sm:max-w-4xl">
				<ExtraFormBody initial={initial} onClose={() => onOpenChange(false)} />
			</DialogContent>
		</Dialog>
	)
}

const REASON_PLACEHOLDER = 'Raison'

function ExtraFormBody({
	initial,
	onClose,
}: {
	initial?: ExtraFormInitial
	onClose: () => void
}) {
	const updateRh = useRhUpdate()
	const actorId = useRhActorId()
	const state = useRhState()
	const baseId = useId()
	const members = activePlanningMembers(state)
	const surcharges = state.surcharges.filter((surcharge) => !surcharge.archived)
	const [form, setForm] = useState<ExtraForm>(() => {
		const date = resolveDate(initial?.date, todayIso())
		return {
			memberId: members.some((member) => member.id === initial?.memberId)
				? (initial?.memberId ?? actorId)
				: actorId,
			date,
			start: '',
			end: '',
			lunchOn: false,
			lunch: DEFAULT_LUNCH,
			reason: '',
			majoration: suggestedMajoration(state, date),
		}
	})
	const [approve, setApprove] = useState(false)
	const [dirty, setDirty] = useState(false)
	const [reasonTouched, setReasonTouched] = useState(false)
	const [majorationTouched, setMajorationTouched] = useState(false)

	const draft = buildExtraDraft(state, form)
	const errors = validateExtra(state, draft)
	const message = errors[0]
	const reasonEmpty = draft.reason.trim() === ''
	const others = reasonEmpty
		? validateExtra(state, { ...draft, reason: REASON_PLACEHOLDER })
		: errors
	const reasonShown = reasonEmpty && others.length === 0
	const urgent =
		dirty && message !== undefined && (!reasonShown || reasonTouched)
	const fieldError = urgent && !reasonShown
	const rateInvalid =
		fieldError &&
		form.majoration.enabled &&
		form.majoration.choice === CUSTOM_RATE &&
		isRateInvalid(draft.rate)
	const spanInvalid = fieldError && !rateInvalid

	function patch(partial: Partial<ExtraForm>) {
		setDirty(true)
		setForm((current) => ({ ...current, ...partial }))
	}

	function changeDate(date: string) {
		setDirty(true)
		setForm((current) => ({
			...current,
			date,
			majoration:
				majorationTouched || !isIsoDate(date)
					? current.majoration
					: suggestedMajoration(state, date),
		}))
	}

	function changeMajoration(partial: Partial<Majoration>) {
		setMajorationTouched(true)
		setDirty(true)
		setForm((current) => ({
			...current,
			majoration: { ...current.majoration, ...partial },
		}))
	}

	function submit(event: FormEvent<HTMLFormElement>) {
		event.preventDefault()
		setDirty(true)
		setReasonTouched(true)
		if (errors.length > 0) return
		updateRh((current, context) =>
			createExtra(current, context, draft, approve),
		)
		onClose()
	}

	const memberId = `${baseId}-member`
	const dateId = `${baseId}-date`
	const startId = `${baseId}-start`
	const endId = `${baseId}-end`
	const lunchStartId = `${baseId}-lunch-start`
	const lunchEndId = `${baseId}-lunch-end`
	const reasonId = `${baseId}-reason`
	const errorId = `${baseId}-error`
	const describedBy = message ? errorId : undefined

	return (
		<form onSubmit={submit} className="grid gap-5" noValidate>
			<DialogHeader>
				<DialogTitle>Déclarer des heures extra</DialogTitle>
				<DialogDescription>
					Heures faites en plus de l’horaire habituel, pour toi ou pour un
					membre de l’équipe. Le récapitulatif se met à jour pendant la saisie.
				</DialogDescription>
			</DialogHeader>
			<div className="grid gap-6 md:grid-cols-[minmax(0,1fr)_17rem]">
				<div className="grid content-start gap-4">
					<div className="grid gap-4 sm:grid-cols-2">
						<div className="grid gap-1.5">
							<Label htmlFor={memberId}>Membre</Label>
							<Select
								value={form.memberId}
								onValueChange={(value) => patch({ memberId: value })}
							>
								<SelectTrigger id={memberId} className="w-full">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									{members.map((entry) => (
										<SelectItem key={entry.id} value={entry.id}>
											<MemberAvatar member={entry} size="sm" />
											{fullName(entry)}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						</div>
						<div className="grid gap-1.5">
							<Label htmlFor={dateId}>Date</Label>
							<Input
								id={dateId}
								type="date"
								value={form.date}
								aria-invalid={spanInvalid || undefined}
								aria-describedby={describedBy}
								onChange={(event) => changeDate(event.target.value)}
							/>
						</div>
					</div>
					<div className="grid gap-4 sm:grid-cols-2">
						<div className="grid gap-1.5">
							<Label htmlFor={startId}>Début</Label>
							<Input
								id={startId}
								type="time"
								value={form.start}
								aria-invalid={spanInvalid || undefined}
								aria-describedby={describedBy}
								onChange={(event) => patch({ start: event.target.value })}
							/>
						</div>
						<div className="grid gap-1.5">
							<Label htmlFor={endId}>Fin</Label>
							<Input
								id={endId}
								type="time"
								value={form.end}
								aria-invalid={spanInvalid || undefined}
								aria-describedby={describedBy}
								onChange={(event) => patch({ end: event.target.value })}
							/>
						</div>
					</div>
					<div className="grid gap-3 rounded-lg border p-3">
						<SwitchRow
							label="Pause déjeuner"
							help={
								form.lunchOn
									? 'La pause est retirée de la durée comptée.'
									: 'Aucune pause : toute la plage est comptée.'
							}
							checked={form.lunchOn}
							onChange={(lunchOn) => patch({ lunchOn })}
						/>
						{form.lunchOn && (
							<div className="grid gap-4 border-t pt-3 sm:grid-cols-2">
								<div className="grid gap-1.5">
									<Label htmlFor={lunchStartId}>Début de la pause</Label>
									<Input
										id={lunchStartId}
										type="time"
										value={form.lunch.start}
										aria-invalid={spanInvalid || undefined}
										aria-describedby={describedBy}
										onChange={(event) =>
											patch({
												lunch: { ...form.lunch, start: event.target.value },
											})
										}
									/>
								</div>
								<div className="grid gap-1.5">
									<Label htmlFor={lunchEndId}>Fin de la pause</Label>
									<Input
										id={lunchEndId}
										type="time"
										value={form.lunch.end}
										aria-invalid={spanInvalid || undefined}
										aria-describedby={describedBy}
										onChange={(event) =>
											patch({
												lunch: { ...form.lunch, end: event.target.value },
											})
										}
									/>
								</div>
							</div>
						)}
					</div>
					<div className="grid gap-1.5">
						<Label htmlFor={reasonId}>Raison</Label>
						<textarea
							id={reasonId}
							rows={3}
							value={form.reason}
							aria-required
							aria-invalid={urgent && reasonShown ? true : undefined}
							aria-describedby={describedBy}
							onChange={(event) => patch({ reason: event.target.value })}
							onBlur={() => setReasonTouched(true)}
							placeholder="Par exemple : chantier à terminer avant la livraison."
							className="w-full resize-none rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-xs outline-none transition-[color,box-shadow] placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 aria-invalid:border-state-alert dark:bg-input/30"
						/>
					</div>
					<MajorationFields
						surcharges={surcharges}
						majoration={form.majoration}
						suggested={!majorationTouched}
						errorId={errorId}
						invalid={rateInvalid}
						onChange={changeMajoration}
					/>
					<SwitchRow
						label="Valider immédiatement"
						help="Sinon, la déclaration reste à valider."
						checked={approve}
						onChange={setApprove}
					/>
					{message &&
						(urgent ? (
							<div id={errorId} role="alert">
								<p className="flex items-start gap-2 text-sm text-state-alert">
									<CircleAlert className="mt-0.5 size-4 shrink-0" />
									{message}
								</p>
							</div>
						) : (
							<p
								id={errorId}
								className="flex items-start gap-2 text-muted-foreground text-sm"
							>
								<Info className="mt-0.5 size-4 shrink-0" />
								{message}
							</p>
						))}
				</div>
				<ExtraRecap draft={draft} approve={approve} />
			</div>
			<DialogFooter>
				<Button type="button" variant="outline" onClick={onClose}>
					Annuler
				</Button>
				<Button type="submit" disabled={errors.length > 0}>
					Déclarer
				</Button>
			</DialogFooter>
		</form>
	)
}

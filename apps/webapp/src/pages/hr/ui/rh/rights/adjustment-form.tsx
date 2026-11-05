import { useId, useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { addAdjustment, type BalancePeriod } from '#/pages/hr/lib/rh/balances'
import {
	formatAmount,
	parseDecimal,
	validateAdjustmentDraft,
} from '#/pages/hr/lib/rh/rights-model'
import type {
	AdjustmentDraft,
	AdjustmentKind,
	IsoDate,
	LeaveAccount,
	Member,
	RhState,
} from '#/pages/hr/lib/rh/types'
import { useRhUpdate } from '#/pages/hr/ui/rh/shared/rh-context'

const KIND_LABELS: Record<AdjustmentKind, string> = {
	adjustment: 'Régularisation',
	carry_over: 'Report',
}

export function AdjustmentForm({
	state,
	member,
	account,
	period,
	today,
	onDone,
	onCancel,
}: {
	state: RhState
	member: Member
	account: LeaveAccount
	period: BalancePeriod
	today: IsoDate
	onDone: (message: string) => void
	onCancel: () => void
}) {
	const updateRh = useRhUpdate()
	const kindId = useId()
	const daysId = useId()
	const dateId = useId()
	const reasonId = useId()
	const hintId = useId()
	const [kind, setKind] = useState<AdjustmentKind>('adjustment')
	const [daysText, setDaysText] = useState('')
	const [date, setDate] = useState(today)
	const [reason, setReason] = useState('')
	const [touched, setTouched] = useState(false)

	const draft: AdjustmentDraft = {
		memberId: member.id,
		accountId: account.id,
		date,
		days: parseDecimal(daysText),
		kind,
		reason,
	}
	const errors = validateAdjustmentDraft(state, draft, today)

	function edit(update: () => void) {
		update()
		setTouched(true)
	}

	function submit() {
		setTouched(true)
		if (errors.length > 0) return
		updateRh((current, context) => addAdjustment(current, context, draft))
		onDone(
			kind === 'carry_over'
				? 'Report ajouté au registre.'
				: 'Régularisation ajoutée au registre.',
		)
	}

	return (
		<form
			noValidate
			aria-label="Ajouter une régularisation"
			className="grid gap-3 rounded-lg border bg-muted/40 p-4"
			onSubmit={(event) => {
				event.preventDefault()
				submit()
			}}
		>
			<div className="grid grid-cols-[11rem_8rem_1fr] gap-3 max-sm:grid-cols-1">
				<div className="grid content-start gap-1.5">
					<Label htmlFor={kindId}>Type</Label>
					<Select
						value={kind}
						onValueChange={(next) =>
							edit(() => setKind(next as AdjustmentKind))
						}
					>
						<SelectTrigger id={kindId} className="w-full bg-background">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{(Object.keys(KIND_LABELS) as AdjustmentKind[]).map((option) => (
								<SelectItem key={option} value={option}>
									{KIND_LABELS[option]}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</div>
				<div className="grid content-start gap-1.5">
					<Label htmlFor={daysId}>Jours</Label>
					<Input
						id={daysId}
						type="number"
						inputMode="decimal"
						step="0.5"
						value={daysText}
						placeholder={kind === 'adjustment' ? 'ex. −1' : 'ex. 3'}
						aria-describedby={hintId}
						onChange={(event) => edit(() => setDaysText(event.target.value))}
						className="tabular bg-background"
					/>
				</div>
				<div className="grid content-start gap-1.5">
					<Label htmlFor={dateId}>Date</Label>
					<Input
						id={dateId}
						type="date"
						min={period.start}
						max={period.end}
						value={date}
						onChange={(event) => edit(() => setDate(event.target.value))}
						className="tabular w-full bg-background sm:w-44"
					/>
				</div>
			</div>
			<p id={hintId} className="-mt-1 text-[13px] text-muted-foreground">
				{kind === 'adjustment'
					? 'Négatif pour retirer des jours, positif pour en ajouter. Le motif est obligatoire.'
					: account.carryOverMax > 0
						? `Positif, au plus ${formatAmount(account.carryOverMax)} jours sur ce compte. Le motif est facultatif.`
						: 'Ce compte n’autorise aucun report.'}
			</p>
			<div className="grid gap-1.5">
				<Label htmlFor={reasonId}>
					Motif
					<span className="font-normal text-muted-foreground">
						{kind === 'adjustment' ? 'obligatoire' : 'facultatif'}
					</span>
				</Label>
				<Input
					id={reasonId}
					value={reason}
					placeholder={
						kind === 'adjustment'
							? 'ex. Jour récupéré après le chantier Dupont'
							: 'ex. Report des congés de la période précédente'
					}
					onChange={(event) => edit(() => setReason(event.target.value))}
					className="bg-background"
				/>
			</div>
			<div aria-live="polite" className="grid gap-1 empty:hidden">
				{touched &&
					errors.map((error) => (
						<p key={error} className="text-sm text-state-alert">
							{error}
						</p>
					))}
			</div>
			<div className="flex justify-end gap-2">
				<Button type="button" variant="outline" size="sm" onClick={onCancel}>
					Annuler
				</Button>
				<Button type="submit" size="sm" disabled={errors.length > 0}>
					{kind === 'carry_over'
						? 'Enregistrer le report'
						: 'Enregistrer la régularisation'}
				</Button>
			</div>
		</form>
	)
}

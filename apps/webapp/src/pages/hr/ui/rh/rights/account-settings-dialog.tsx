import { useId, useState } from 'react'
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
import { periodOf } from '#/pages/hr/lib/rh/balances'
import { accountFailureOf } from '#/pages/hr/lib/rh/leave-wire'
import {
	type AccountSettingsDraft,
	convertAcquisitionValue,
	describePeriod,
	parseDecimal,
	validateAccountSettings,
} from '#/pages/hr/lib/rh/rights-model'
import type {
	Acquisition,
	IsoDate,
	LeaveAccount,
} from '#/pages/hr/lib/rh/types'
import { useRhLeaveActions } from '#/pages/hr/ui/rh/shared/rh-context'

const MONTHS = [
	'Janvier',
	'Février',
	'Mars',
	'Avril',
	'Mai',
	'Juin',
	'Juillet',
	'Août',
	'Septembre',
	'Octobre',
	'Novembre',
	'Décembre',
]

const KIND_LABELS: Record<Acquisition['kind'], string> = {
	monthly: 'Mensuelle',
	annual: 'Annuelle',
}

function inputText(value: number): string {
	return String(Math.round(value * 100) / 100)
}

function rateOf(account: LeaveAccount): number {
	return account.acquisition.kind === 'monthly'
		? account.acquisition.perMonth
		: account.acquisition.days
}

export function AccountSettingsDialog({
	account,
	today,
	onClose,
}: {
	account: LeaveAccount
	today: IsoDate
	onClose: () => void
}) {
	const leave = useRhLeaveActions()
	const kindId = useId()
	const valueId = useId()
	const monthId = useId()
	const carryId = useId()
	const initialValue = rateOf(account)
	const initialText = inputText(initialValue)
	const [kind, setKind] = useState(account.acquisition.kind)
	const [valueText, setValueText] = useState(initialText)
	const [periodStartMonth, setPeriodStartMonth] = useState(
		account.periodStartMonth,
	)
	const [carryText, setCarryText] = useState(inputText(account.carryOverMax))
	const [pending, setPending] = useState(false)
	const [failure, setFailure] = useState<string | null>(null)

	const draft: AccountSettingsDraft = {
		kind,
		value:
			kind === account.acquisition.kind && valueText === initialText
				? initialValue
				: parseDecimal(valueText),
		periodStartMonth,
		carryOverMax: parseDecimal(carryText),
	}
	const errors = validateAccountSettings(draft)
	const period = periodOf({ ...account, periodStartMonth }, today)

	function changeKind(next: Acquisition['kind']) {
		setValueText(
			Number.isFinite(draft.value)
				? inputText(convertAcquisitionValue(kind, next, draft.value))
				: '',
		)
		setKind(next)
	}

	async function run(action: () => Promise<void>) {
		if (pending) return
		setPending(true)
		setFailure(null)
		try {
			await action()
			onClose()
		} catch (error) {
			setFailure(accountFailureOf(error).message)
			setPending(false)
		}
	}

	function save() {
		if (errors.length > 0) return
		void run(() => leave.saveAccount(account, draft))
	}

	function toggleArchive() {
		void run(() =>
			account.archived
				? leave.restoreAccount(account.id)
				: leave.archiveAccount(account.id),
		)
	}

	return (
		<Dialog
			open
			onOpenChange={(open) => {
				if (!open) onClose()
			}}
		>
			<DialogContent showCloseButton={false} className="sm:max-w-md">
				<DialogHeader>
					<DialogTitle>Paramétrer le compte {account.name}</DialogTitle>
					<DialogDescription>
						Règle d’acquisition, début de période et report maximal.
					</DialogDescription>
				</DialogHeader>
				<form
					noValidate
					className="grid gap-4"
					onSubmit={(event) => {
						event.preventDefault()
						save()
					}}
				>
					<div className="grid grid-cols-2 gap-3">
						<div className="grid gap-1.5">
							<Label htmlFor={kindId}>Type d’acquisition</Label>
							<Select
								value={kind}
								onValueChange={(next) =>
									changeKind(next as Acquisition['kind'])
								}
							>
								<SelectTrigger id={kindId} className="w-full">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									{(Object.keys(KIND_LABELS) as Acquisition['kind'][]).map(
										(option) => (
											<SelectItem key={option} value={option}>
												{KIND_LABELS[option]}
											</SelectItem>
										),
									)}
								</SelectContent>
							</Select>
						</div>
						<div className="grid gap-1.5">
							<Label htmlFor={valueId}>
								{kind === 'monthly' ? 'Jours par mois' : 'Jours par an'}
							</Label>
							<Input
								id={valueId}
								type="number"
								inputMode="decimal"
								min={0}
								step="0.01"
								value={valueText}
								onChange={(event) => setValueText(event.target.value)}
								className="tabular"
							/>
						</div>
					</div>
					<div className="grid gap-1.5">
						<Label htmlFor={monthId}>Mois de début de période</Label>
						<Select
							value={String(periodStartMonth)}
							onValueChange={(next) => setPeriodStartMonth(Number(next))}
						>
							<SelectTrigger id={monthId} className="w-full">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{MONTHS.map((label, index) => (
									<SelectItem key={label} value={String(index + 1)}>
										{label}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
						<p className="text-[13px] text-muted-foreground">
							Période en cours : {describePeriod(period)}.
						</p>
					</div>
					<div className="grid gap-1.5">
						<Label htmlFor={carryId}>Report maximal (jours)</Label>
						<Input
							id={carryId}
							type="number"
							inputMode="decimal"
							min={0}
							step="0.5"
							value={carryText}
							onChange={(event) => setCarryText(event.target.value)}
							className="tabular"
						/>
						<p className="text-[13px] text-muted-foreground">
							Indique 0 pour interdire tout report d’une période à l’autre.
						</p>
					</div>
					<p className="rounded-md bg-muted px-3 py-2 text-[13px] text-muted-foreground">
						À l’enregistrement, les soldes de tous les membres sont recalculés
						avec cette règle, à partir des absences et des régularisations déjà
						saisies.
					</p>
					<div aria-live="polite" className="grid gap-1 empty:hidden">
						{errors.map((error) => (
							<p key={error} className="text-sm text-state-alert">
								{error}
							</p>
						))}
						{failure && (
							<p role="alert" className="text-sm text-state-alert">
								{failure}
							</p>
						)}
					</div>
					<DialogFooter>
						<Button
							type="button"
							variant="outline"
							disabled={pending}
							onClick={toggleArchive}
							className="sm:mr-auto"
						>
							{account.archived ? 'Restaurer le compte' : 'Archiver le compte'}
						</Button>
						<Button type="button" variant="outline" onClick={onClose}>
							Annuler
						</Button>
						<Button type="submit" disabled={errors.length > 0 || pending}>
							Enregistrer
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}

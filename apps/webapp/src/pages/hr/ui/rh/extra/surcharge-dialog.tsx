import { CircleAlert } from 'lucide-react'
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
import {
	MAX_RATE,
	surchargeUsage,
	validateSurcharge,
} from '#/pages/hr/lib/rh/extra'
import {
	describeHourExample,
	faultyField,
	parseRate,
	SURCHARGE_KINDS,
} from '#/pages/hr/lib/rh/extra-view'
import {
	type SurchargeFailure,
	surchargeFailureOf,
} from '#/pages/hr/lib/rh/extra-wire'
import type { Surcharge, SurchargeKind } from '#/pages/hr/lib/rh/types'
import {
	useRhState,
	useRhSurchargeActions,
} from '#/pages/hr/ui/rh/shared/rh-context'

export function SurchargeDialog({
	open,
	onOpenChange,
	surcharge,
}: {
	open: boolean
	onOpenChange: (open: boolean) => void
	surcharge: Surcharge | null
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[calc(100dvh-2rem)] overflow-y-auto sm:max-w-lg">
				<SurchargeForm
					surcharge={surcharge}
					onClose={() => onOpenChange(false)}
				/>
			</DialogContent>
		</Dialog>
	)
}

function SurchargeForm({
	surcharge,
	onClose,
}: {
	surcharge: Surcharge | null
	onClose: () => void
}) {
	const actions = useRhSurchargeActions()
	const state = useRhState()
	const baseId = useId()
	const [failure, setFailure] = useState<SurchargeFailure | null>(null)
	const [isPending, setIsPending] = useState(false)
	const [touched, setTouched] = useState(false)
	const [name, setName] = useState(surcharge?.name ?? '')
	const [rateInput, setRateInput] = useState(
		surcharge ? String(surcharge.rate) : '',
	)
	const [kind, setKind] = useState<SurchargeKind>(surcharge?.kind ?? 'overtime')
	const rate = parseRate(rateInput)
	const draft = { name, rate, kind }
	const errors = validateSurcharge(state, draft, surcharge?.id)
	const shown = failure?.message ?? (touched ? errors[0] : undefined)
	const fault = failure?.field ?? faultyField(name, rate, shown !== undefined)
	const usage = surcharge ? surchargeUsage(state, surcharge.id) : 0
	const nameId = `${baseId}-name`
	const rateId = `${baseId}-rate`
	const kindId = `${baseId}-kind`
	const errorId = `${baseId}-error`

	async function submit(event: FormEvent<HTMLFormElement>) {
		event.preventDefault()
		setTouched(true)
		if (errors.length > 0 || isPending) return
		setIsPending(true)
		setFailure(null)
		try {
			await actions.save(surcharge?.id ?? null, {
				name: name.trim(),
				rate,
				kind,
			})
			onClose()
		} catch (error) {
			setFailure(surchargeFailureOf(error))
		} finally {
			setIsPending(false)
		}
	}

	return (
		<form
			onSubmit={(event) => void submit(event)}
			className="grid gap-5"
			noValidate
		>
			<DialogHeader>
				<DialogTitle>
					{surcharge ? 'Modifier la majoration' : 'Nouvelle majoration'}
				</DialogTitle>
				<DialogDescription>
					Un barème propose un taux prêt à l’emploi lors de la déclaration des
					heures extra.
				</DialogDescription>
			</DialogHeader>
			<div className="grid gap-1.5">
				<Label htmlFor={nameId}>Nom</Label>
				<Input
					id={nameId}
					value={name}
					placeholder="Par exemple : Heures de nuit en chantier"
					aria-required
					aria-invalid={fault === 'name' ? true : undefined}
					aria-describedby={fault === 'name' ? errorId : undefined}
					onChange={(event) => {
						setFailure(null)
						setName(event.target.value)
					}}
				/>
			</div>
			<div className="grid gap-4 sm:grid-cols-[8rem_minmax(0,1fr)]">
				<div className="grid content-start gap-1.5">
					<Label htmlFor={rateId}>Taux</Label>
					<div className="relative">
						<Input
							id={rateId}
							type="number"
							inputMode="decimal"
							min={1}
							max={MAX_RATE}
							step="any"
							value={rateInput}
							placeholder="25"
							aria-required
							aria-invalid={fault === 'rate' ? true : undefined}
							aria-describedby={fault === 'rate' ? errorId : undefined}
							onChange={(event) => {
								setFailure(null)
								setRateInput(event.target.value)
							}}
							className="pr-8"
						/>
						<span
							aria-hidden
							className="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-muted-foreground text-sm"
						>
							%
						</span>
					</div>
				</div>
				<div className="grid content-start gap-1.5">
					<Label htmlFor={kindId}>Type</Label>
					<Select
						value={kind}
						onValueChange={(value) => {
							setFailure(null)
							setKind(value as SurchargeKind)
						}}
					>
						<SelectTrigger id={kindId} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{SURCHARGE_KINDS.map((entry) => (
								<SelectItem key={entry.value} value={entry.value}>
									{entry.label}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</div>
			</div>
			<p className="text-muted-foreground text-xs">
				{Number.isFinite(rate) && rate > 0
					? describeHourExample(rate)
					: 'Indique un pourcentage de majoration, de 1 à 200 %.'}{' '}
				Le type sert à proposer le bon barème selon le jour : dimanche, jour
				férié ou heures supplémentaires.
			</p>
			{shown && (
				<div id={errorId} role="alert">
					<p className="flex items-start gap-2 text-sm text-state-alert">
						<CircleAlert className="mt-0.5 size-4 shrink-0" />
						{shown}
					</p>
				</div>
			)}
			{surcharge && usage > 0 && (
				<p className="rounded-md bg-muted/50 px-3 py-2 text-[13px] text-muted-foreground">
					Cette majoration est utilisée par {usage} déclaration
					{usage > 1 ? 's' : ''} : elles gardent le taux d’origine, seules les
					nouvelles déclarations utiliseront le nouveau taux.
				</p>
			)}
			<DialogFooter>
				<Button type="button" variant="outline" onClick={onClose}>
					Annuler
				</Button>
				<Button type="submit" disabled={isPending}>
					{surcharge ? 'Enregistrer' : 'Créer la majoration'}
				</Button>
			</DialogFooter>
		</form>
	)
}

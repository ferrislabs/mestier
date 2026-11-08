import { Check } from 'lucide-react'
import { type CSSProperties, type FormEvent, useId, useState } from 'react'
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
import { Switch } from '#/components/ui/switch'
import { cn } from '#/lib/utils'
import { type TypeFailure, typeFailureOf } from '#/pages/hr/lib/rh/absence-wire'
import { typeUsage } from '#/pages/hr/lib/rh/absences'
import {
	COUNT_MODES,
	countModeOption,
	TYPE_HUES,
	validateTypeName,
} from '#/pages/hr/lib/rh/absences-view'
import type {
	AbsenceType,
	AbsenceTypeDraft,
	CountMode,
} from '#/pages/hr/lib/rh/types'
import {
	useRhAbsenceTypeActions,
	useRhState,
} from '#/pages/hr/ui/rh/shared/rh-context'
import { TYPE_SOLID, TypeChip } from '#/pages/hr/ui/rh/shared/type-chip'

const NO_ACCOUNT = 'none'

export function AbsenceTypeDialog({
	open,
	onOpenChange,
	type,
}: {
	open: boolean
	onOpenChange: (open: boolean) => void
	type: AbsenceType | null
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[calc(100dvh-2rem)] overflow-y-auto sm:max-w-lg">
				<AbsenceTypeForm type={type} onClose={() => onOpenChange(false)} />
			</DialogContent>
		</Dialog>
	)
}

function AbsenceTypeForm({
	type,
	onClose,
}: {
	type: AbsenceType | null
	onClose: () => void
}) {
	const actions = useRhAbsenceTypeActions()
	const state = useRhState()
	const baseId = useId()
	const [touched, setTouched] = useState(false)
	const [failure, setFailure] = useState<TypeFailure | null>(null)
	const [isPending, setIsPending] = useState(false)
	const [draft, setDraft] = useState<AbsenceTypeDraft>(() =>
		type
			? {
					name: type.name,
					hue: type.hue,
					kind: type.kind,
					countMode: type.countMode,
					accountId: type.accountId,
					paid: type.paid,
					needsApproval: type.needsApproval,
					needsProof: type.needsProof,
				}
			: {
					name: '',
					hue: TYPE_HUES[0].hue,
					kind: 'leave',
					countMode: 'working',
					accountId: null,
					paid: true,
					needsApproval: true,
					needsProof: false,
				},
	)
	const nameError = validateTypeName(state, draft.name, type?.id)
	const shownError = nameError && (touched || draft.name.trim() !== '')
	const usage = type ? typeUsage(state, type.id) : 0
	const hues = TYPE_HUES.some((entry) => entry.hue === draft.hue)
		? TYPE_HUES
		: [...TYPE_HUES, { hue: draft.hue, name: `Teinte ${draft.hue}` }]
	const nameId = `${baseId}-name`
	const modeId = `${baseId}-mode`
	const accountId = `${baseId}-account`
	const errorId = `${baseId}-error`

	function patch(partial: Partial<AbsenceTypeDraft>) {
		setFailure(null)
		setDraft((current) => ({ ...current, ...partial }))
	}

	async function submit(event: FormEvent<HTMLFormElement>) {
		event.preventDefault()
		setTouched(true)
		if (nameError || isPending) return
		setIsPending(true)
		setFailure(null)
		try {
			await actions.save(type?.id ?? null, {
				...draft,
				name: draft.name.trim(),
			})
			onClose()
		} catch (error) {
			setFailure(typeFailureOf(error))
		} finally {
			setIsPending(false)
		}
	}

	const nameMessage =
		(shownError ? nameError : null) ??
		(failure?.field === 'name' ? failure.message : null)
	const dialogFailure =
		failure && failure.field !== 'name' ? failure.message : null

	return (
		<form
			onSubmit={(event) => void submit(event)}
			className="grid gap-5"
			noValidate
		>
			<DialogHeader>
				<DialogTitle>
					{type ? 'Modifier le motif' : 'Nouveau motif'}
				</DialogTitle>
				<DialogDescription>
					Le motif fixe le décompte des jours, le compte débité et le circuit de
					validation.
				</DialogDescription>
			</DialogHeader>
			<div className="grid gap-1.5">
				<Label htmlFor={nameId}>Nom</Label>
				<Input
					id={nameId}
					value={draft.name}
					placeholder="Par exemple : Déménagement"
					aria-required
					aria-invalid={nameMessage ? true : undefined}
					aria-describedby={nameMessage ? errorId : undefined}
					onChange={(event) => patch({ name: event.target.value })}
					onBlur={() => setTouched(true)}
				/>
				{nameMessage && (
					<p id={errorId} className="text-sm text-state-alert">
						{nameMessage}
					</p>
				)}
			</div>
			<fieldset className="grid gap-2">
				<legend className="mb-2 flex items-center gap-3 font-medium text-sm">
					Couleur
					<TypeChip
						type={{ name: draft.name.trim() || 'Aperçu', hue: draft.hue }}
					/>
				</legend>
				<div className="flex flex-wrap gap-2.5">
					{hues.map((entry) => (
						<label key={entry.hue} className="relative cursor-pointer">
							<input
								type="radio"
								name={`${baseId}-hue`}
								value={entry.hue}
								checked={draft.hue === entry.hue}
								onChange={() => patch({ hue: entry.hue })}
								className="peer sr-only"
							/>
							<span
								style={{ '--hue': entry.hue } as CSSProperties}
								className={cn(
									'flex size-7 items-center justify-center rounded-full text-white outline-none ring-2 ring-transparent ring-offset-2 ring-offset-background transition-shadow hover:ring-border peer-checked:ring-foreground peer-focus-visible:ring-ring [&>svg]:opacity-0 peer-checked:[&>svg]:opacity-100',
									TYPE_SOLID,
								)}
							>
								<Check className="size-3.5" aria-hidden />
							</span>
							<span className="sr-only">{entry.name}</span>
						</label>
					))}
				</div>
			</fieldset>
			<div className="grid gap-4 sm:grid-cols-2">
				<div className="grid content-start gap-1.5">
					<Label htmlFor={modeId}>Mode de décompte</Label>
					<Select
						value={draft.countMode}
						onValueChange={(value) => patch({ countMode: value as CountMode })}
					>
						<SelectTrigger id={modeId} className="w-full">
							<SelectValue>
								{countModeOption(draft.countMode).label}
							</SelectValue>
						</SelectTrigger>
						<SelectContent>
							{COUNT_MODES.map((option) => (
								<SelectItem key={option.value} value={option.value}>
									<span className="grid gap-0.5">
										<span>{option.label}</span>
										<span className="max-w-64 whitespace-normal text-muted-foreground text-xs">
											{option.help}
										</span>
									</span>
								</SelectItem>
							))}
						</SelectContent>
					</Select>
					<p className="text-muted-foreground text-xs">
						{countModeOption(draft.countMode).help}
					</p>
				</div>
				<div className="grid content-start gap-1.5">
					<Label htmlFor={accountId}>Compte débité</Label>
					<Select
						value={draft.accountId ?? NO_ACCOUNT}
						onValueChange={(value) =>
							patch({ accountId: value === NO_ACCOUNT ? null : value })
						}
					>
						<SelectTrigger id={accountId} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value={NO_ACCOUNT}>Aucun</SelectItem>
							{state.accounts.map((account) => (
								<SelectItem key={account.id} value={account.id}>
									{account.name}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
					<p className="text-muted-foreground text-xs">
						{draft.accountId
							? 'Les jours décomptés réduisent le solde du membre.'
							: 'Les jours ne réduisent aucun solde.'}
					</p>
				</div>
			</div>
			<div className="grid gap-4 border-t pt-4">
				<SwitchRow
					label="Rémunéré"
					help="L’absence n’entraîne pas de retenue sur salaire."
					checked={draft.paid}
					onChange={(paid) => patch({ paid })}
				/>
				<SwitchRow
					label="Validation par un responsable"
					help="Sinon, l’absence est validée dès qu’elle est posée."
					checked={draft.needsApproval}
					onChange={(needsApproval) => patch({ needsApproval })}
				/>
				<SwitchRow
					label="Justificatif attendu"
					help="Un rappel s’affiche lors de la saisie, sans bloquer l’envoi."
					checked={draft.needsProof}
					onChange={(needsProof) => patch({ needsProof })}
				/>
			</div>
			{type && usage > 0 && (
				<p className="rounded-md bg-muted/50 px-3 py-2 text-[13px] text-muted-foreground">
					Ce motif est utilisé par {usage} demande{usage > 1 ? 's' : ''} : les
					changements s’appliquent aussi aux demandes existantes.
				</p>
			)}
			{dialogFailure && (
				<p role="alert" className="text-sm text-state-alert">
					{dialogFailure}
				</p>
			)}
			<DialogFooter>
				<Button type="button" variant="outline" onClick={onClose}>
					Annuler
				</Button>
				<Button type="submit" disabled={isPending}>
					{type ? 'Enregistrer' : 'Créer le motif'}
				</Button>
			</DialogFooter>
		</form>
	)
}

function SwitchRow({
	label,
	help,
	checked,
	onChange,
}: {
	label: string
	help: string
	checked: boolean
	onChange: (checked: boolean) => void
}) {
	const id = useId()
	return (
		<div className="flex items-center justify-between gap-4">
			<div className="grid gap-1">
				<Label htmlFor={id}>{label}</Label>
				<p className="text-muted-foreground text-xs">{help}</p>
			</div>
			<Switch id={id} checked={checked} onCheckedChange={onChange} />
		</div>
	)
}

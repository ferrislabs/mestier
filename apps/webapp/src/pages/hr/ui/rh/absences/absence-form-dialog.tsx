import {
	CircleAlert,
	Clock,
	Paperclip,
	TriangleAlert,
	X,
	Zap,
} from 'lucide-react'
import { type FormEvent, useId, useRef, useState } from 'react'
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
	type AbsenceFailure,
	absenceFailureOf,
} from '#/pages/hr/lib/rh/absence-wire'
import {
	absenceDays,
	findMember,
	findType,
	teamConflicts,
	validateAbsence,
} from '#/pages/hr/lib/rh/absences'
import {
	balanceImpact,
	countModeOption,
	formatDayCount,
	resolveStartDate,
} from '#/pages/hr/lib/rh/absences-view'
import { todayIso } from '#/pages/hr/lib/rh/dates'
import { fullName } from '#/pages/hr/lib/rh/members'
import type { AbsenceDraft, DayPart, IsoDate } from '#/pages/hr/lib/rh/types'
import {
	BalanceImpactLine,
	TeamConflictList,
} from '#/pages/hr/ui/rh/absences/decision-context'
import { MemberAvatar } from '#/pages/hr/ui/rh/shared/member-avatar'
import { useCanRequestForOthers } from '#/pages/hr/ui/rh/shared/request-access'
import {
	useRhAbsenceActions,
	useRhActorId,
	useRhState,
} from '#/pages/hr/ui/rh/shared/rh-context'
import { TypeDot } from '#/pages/hr/ui/rh/shared/type-chip'

export type AbsenceFormInitial = { memberId?: string; startDate?: IsoDate }

export type AbsenceFormDialogProps = {
	open: boolean
	onOpenChange: (open: boolean) => void
	initial?: AbsenceFormInitial
}

export function AbsenceFormDialog({
	open,
	onOpenChange,
	initial,
}: AbsenceFormDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[calc(100dvh-2rem)] overflow-y-auto sm:max-w-4xl">
				<AbsenceForm initial={initial} onClose={() => onOpenChange(false)} />
			</DialogContent>
		</Dialog>
	)
}

function AbsenceForm({
	initial,
	onClose,
}: {
	initial?: AbsenceFormInitial
	onClose: () => void
}) {
	const actions = useRhAbsenceActions()
	const [proof, setProof] = useState<File | null>(null)
	const [failure, setFailure] = useState<AbsenceFailure | null>(null)
	const [isPending, setIsPending] = useState(false)
	const actorId = useRhActorId()
	const state = useRhState()
	const baseId = useId()
	const proofRef = useRef<HTMLInputElement>(null)
	const today = todayIso()
	const forOthers = useCanRequestForOthers('absence')
	const activeMembers = state.members.filter(
		(member) => member.status === 'active',
	)
	const members = activeMembers.filter(
		(member) => forOthers || member.id === actorId,
	)
	const types = state.absenceTypes.filter((type) => !type.archived)
	const [stored, setDraft] = useState<AbsenceDraft>(() => {
		const startDate = resolveStartDate(initial?.startDate, today)
		return {
			memberId: activeMembers.some((member) => member.id === initial?.memberId)
				? (initial?.memberId ?? actorId)
				: actorId,
			typeId: types[0]?.id ?? '',
			startDate,
			startPart: 'morning',
			endDate: startDate,
			endPart: 'afternoon',
			comment: '',
			proofName: null,
		}
	})

	const draft = forOthers ? stored : { ...stored, memberId: actorId }
	const member = findMember(state, draft.memberId)
	const type = findType(state, draft.typeId)
	const hasDates = draft.startDate !== '' && draft.endDate !== ''
	const errors = validateAbsence(state, draft)
	const days = member && type && hasDates ? absenceDays(draft, member, type) : 0
	const conflicts = hasDates ? teamConflicts(state, draft) : []
	const impact =
		hasDates && days > 0 ? balanceImpact(state, draft, today, false) : null
	const mode = type ? countModeOption(type.countMode) : null

	function patch(partial: Partial<AbsenceDraft>) {
		setFailure(null)
		setDraft((current) => ({ ...current, ...partial }))
	}

	function changeStart(value: string) {
		setFailure(null)
		setDraft((current) => ({
			...current,
			startDate: value,
			endDate:
				value !== '' && (current.endDate === '' || value > current.endDate)
					? value
					: current.endDate,
		}))
	}

	async function submit(event: FormEvent<HTMLFormElement>) {
		event.preventDefault()
		if (errors.length > 0 || isPending) return
		setIsPending(true)
		setFailure(null)
		try {
			await actions.create({ ...draft, comment: draft.comment.trim() }, proof)
			onClose()
		} catch (error) {
			setFailure(absenceFailureOf(error))
		} finally {
			setIsPending(false)
		}
	}

	function pickProof(files: FileList | null) {
		const file = files?.[0] ?? null
		setProof(file)
		patch({ proofName: file?.name ?? null })
	}

	function clearProof() {
		if (proofRef.current) proofRef.current.value = ''
		setProof(null)
		patch({ proofName: null })
	}

	const fieldFailure = (field: AbsenceFailure['field']) =>
		failure?.field === field ? failure.message : null
	const periodFailure =
		fieldFailure('startDate') ?? fieldFailure('endDate') ?? fieldFailure('type')
	const dialogFailure =
		failure && (failure.field === null || failure.field === 'status')
			? failure.message
			: null

	const memberId = `${baseId}-member`
	const typeId = `${baseId}-type`
	const startId = `${baseId}-start`
	const endId = `${baseId}-end`
	const commentId = `${baseId}-comment`
	const proofId = `${baseId}-proof`
	const errorsId = `${baseId}-errors`

	return (
		<form
			onSubmit={(event) => void submit(event)}
			className="grid gap-5"
			noValidate
		>
			<DialogHeader>
				<DialogTitle>Poser une absence</DialogTitle>
				<DialogDescription>
					Pour toi ou pour un membre de l’équipe. Le récapitulatif se met à jour
					pendant la saisie.
				</DialogDescription>
			</DialogHeader>
			<div className="grid gap-6 md:grid-cols-[minmax(0,1fr)_17rem]">
				<div className="grid content-start gap-4">
					<div className="grid gap-4 sm:grid-cols-2">
						<div className="grid gap-1.5">
							<Label htmlFor={memberId}>Membre</Label>
							<Select
								value={draft.memberId}
								onValueChange={(value) => value && patch({ memberId: value })}
								disabled={!forOthers}
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
							<Label htmlFor={typeId}>Motif</Label>
							<Select
								value={draft.typeId}
								onValueChange={(value) => patch({ typeId: value })}
							>
								<SelectTrigger id={typeId} className="w-full">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									{types.map((entry) => (
										<SelectItem key={entry.id} value={entry.id}>
											<TypeDot hue={entry.hue} />
											{entry.name}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						</div>
					</div>
					<div className="grid gap-1.5">
						<Label htmlFor={startId}>Début</Label>
						<div className="flex gap-2">
							<Input
								id={startId}
								type="date"
								value={draft.startDate}
								aria-invalid={errors.length > 0 || undefined}
								aria-describedby={errors.length > 0 ? errorsId : undefined}
								onChange={(event) => changeStart(event.target.value)}
								className="w-40 shrink-0"
							/>
							<PartSelect
								label="Moment de début"
								value={draft.startPart}
								onChange={(startPart) => patch({ startPart })}
								morning="Début le matin"
								afternoon="Début l’après-midi"
							/>
						</div>
					</div>
					<div className="grid gap-1.5">
						<Label htmlFor={endId}>Fin</Label>
						<div className="flex gap-2">
							<Input
								id={endId}
								type="date"
								value={draft.endDate}
								min={draft.startDate || undefined}
								aria-invalid={errors.length > 0 || undefined}
								aria-describedby={errors.length > 0 ? errorsId : undefined}
								onChange={(event) => patch({ endDate: event.target.value })}
								className="w-40 shrink-0"
							/>
							<PartSelect
								label="Moment de fin"
								value={draft.endPart}
								onChange={(endPart) => patch({ endPart })}
								morning="Fin à midi"
								afternoon="Fin en fin de journée"
							/>
						</div>
					</div>
					{(errors.length > 0 || periodFailure) && (
						<div id={errorsId} role="alert">
							<ul className="grid gap-1">
								{[...errors, ...(periodFailure ? [periodFailure] : [])].map(
									(message) => (
										<li
											key={message}
											className="flex items-start gap-2 text-sm text-state-alert"
										>
											<CircleAlert className="mt-0.5 size-4 shrink-0" />
											{message}
										</li>
									),
								)}
							</ul>
						</div>
					)}
					<div className="grid gap-1.5">
						<Label htmlFor={commentId}>
							Commentaire
							<span className="font-normal text-muted-foreground">
								facultatif
							</span>
						</Label>
						<textarea
							id={commentId}
							rows={3}
							value={draft.comment}
							onChange={(event) => patch({ comment: event.target.value })}
							placeholder="Une précision utile pour la validation."
							aria-invalid={fieldFailure('comment') ? true : undefined}
							className="w-full resize-none rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-xs outline-none transition-[color,box-shadow] placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:bg-input/30"
						/>
					</div>
					<div className="grid gap-1.5">
						<Label htmlFor={proofId}>
							Justificatif
							{type?.needsProof ? (
								<span className="font-normal text-state-today">
									Justificatif attendu
								</span>
							) : (
								<span className="font-normal text-muted-foreground">
									facultatif
								</span>
							)}
						</Label>
						<input
							ref={proofRef}
							id={proofId}
							type="file"
							className="hidden"
							onChange={(event) => pickProof(event.target.files)}
						/>
						<div className="flex min-h-8 items-center gap-2">
							<Button
								type="button"
								variant="outline"
								size="sm"
								onClick={() => proofRef.current?.click()}
							>
								<Paperclip />
								{draft.proofName
									? 'Remplacer le fichier'
									: 'Joindre un fichier'}
							</Button>
							{draft.proofName ? (
								<>
									<span className="min-w-0 truncate text-sm">
										{draft.proofName}
									</span>
									<Button
										type="button"
										variant="ghost"
										size="icon-xs"
										aria-label="Retirer le justificatif"
										onClick={clearProof}
									>
										<X />
									</Button>
								</>
							) : (
								<span className="text-muted-foreground text-xs">
									Le fichier est envoyé avec la demande.
								</span>
							)}
						</div>
						{fieldFailure('proof') && (
							<p role="alert" className="text-sm text-state-alert">
								{fieldFailure('proof')}
							</p>
						)}
						{fieldFailure('comment') && (
							<p role="alert" className="text-sm text-state-alert">
								{fieldFailure('comment')}
							</p>
						)}
					</div>
				</div>
				<aside
					aria-label="Récapitulatif de la demande"
					className="grid content-start gap-4 self-start rounded-lg bg-muted/50 p-4 text-[13px]"
				>
					<p className="font-medium text-[11px] text-muted-foreground uppercase tracking-wide">
						Récapitulatif
					</p>
					<p className="font-medium text-sm">
						{days > 0
							? `${formatDayCount(days)} décompté${days >= 2 ? 's' : ''}`
							: 'Aucun jour décompté'}
						{mode && (
							<span className="font-normal text-muted-foreground">
								, {mode.short}
							</span>
						)}
					</p>
					{impact ? (
						<BalanceImpactLine impact={impact} />
					) : (
						type && (
							<p className="text-muted-foreground">
								{type.accountId
									? 'Choisis une période pour voir l’effet sur le solde.'
									: 'Ce motif ne débite aucun compte de congés.'}
							</p>
						)
					)}
					{type && (
						<p className="flex items-start gap-2">
							{type.needsApproval ? (
								<Clock className="mt-0.5 size-3.5 shrink-0 text-state-today" />
							) : (
								<Zap className="mt-0.5 size-3.5 shrink-0 text-state-progress" />
							)}
							{type.needsApproval
								? 'Cette demande sera envoyée à validation'
								: 'Validée automatiquement'}
						</p>
					)}
					{type?.needsProof && !draft.proofName && (
						<p className="flex items-start gap-2 text-muted-foreground">
							<TriangleAlert className="mt-0.5 size-3.5 shrink-0 text-state-today" />
							Justificatif attendu pour ce motif : sans lui, la demande ne
							pourra pas être validée.
						</p>
					)}
					<div className="border-t pt-3">
						<TeamConflictList state={state} conflicts={conflicts} />
					</div>
				</aside>
			</div>
			{dialogFailure && (
				<p role="alert" className="text-sm text-state-alert">
					{dialogFailure}
				</p>
			)}
			<DialogFooter>
				<Button type="button" variant="outline" onClick={onClose}>
					Annuler
				</Button>
				<Button type="submit" disabled={errors.length > 0 || isPending}>
					{type && !type.needsApproval
						? 'Enregistrer l’absence'
						: 'Envoyer la demande'}
				</Button>
			</DialogFooter>
		</form>
	)
}

function PartSelect({
	label,
	value,
	onChange,
	morning,
	afternoon,
}: {
	label: string
	value: DayPart
	onChange: (part: DayPart) => void
	morning: string
	afternoon: string
}) {
	return (
		<Select value={value} onValueChange={(next) => onChange(next as DayPart)}>
			<SelectTrigger aria-label={label} className="min-w-0 flex-1">
				<SelectValue />
			</SelectTrigger>
			<SelectContent>
				<SelectItem value="morning">{morning}</SelectItem>
				<SelectItem value="afternoon">{afternoon}</SelectItem>
			</SelectContent>
		</Select>
	)
}

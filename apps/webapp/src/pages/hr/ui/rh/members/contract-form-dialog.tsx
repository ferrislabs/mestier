import { type FormEvent, type ReactNode, useId, useState } from 'react'
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
import {
	contractDraftOf,
	finalizeContractDraft,
	isContractError,
} from '#/pages/hr/lib/rh/contract-draft'
import { validateSchedule } from '#/pages/hr/lib/rh/contract-schedule'
import {
	CONTRACT_LABELS,
	suggestNextContract,
} from '#/pages/hr/lib/rh/contracts'
import { todayIso } from '#/pages/hr/lib/rh/dates'
import { contractPeriod } from '#/pages/hr/lib/rh/format'
import {
	addContract,
	contractErrors,
	fullName,
	updateContract,
} from '#/pages/hr/lib/rh/members'
import type { Contract, ContractDraft, Member } from '#/pages/hr/lib/rh/types'
import { ContractFields } from '#/pages/hr/ui/rh/members/contract-fields'
import { useRhUpdate } from '#/pages/hr/ui/rh/shared/rh-context'

export type ContractFormDialogProps = {
	open: boolean
	onOpenChange: (open: boolean) => void
	member: Member
	contract?: Contract
	trigger?: ReactNode
}

export function ContractFormDialog({
	open,
	onOpenChange,
	member,
	contract,
	trigger,
}: ContractFormDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			{trigger && <DialogTrigger asChild>{trigger}</DialogTrigger>}
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-2xl">
				<ContractForm
					member={member}
					contract={contract}
					onClose={() => onOpenChange(false)}
				/>
			</DialogContent>
		</Dialog>
	)
}

function ContractForm({
	member,
	contract,
	onClose,
}: {
	member: Member
	contract?: Contract
	onClose: () => void
}) {
	const updateRh = useRhUpdate()
	const idPrefix = useId()
	const [draft, setDraft] = useState<ContractDraft>(() =>
		contract
			? contractDraftOf(contract)
			: suggestNextContract(member, todayIso()),
	)
	const final = finalizeContractDraft(draft)
	const errors = contractErrors(member, final, contract?.id)
	const scheduleMessages = validateSchedule(draft.schedule)
	const otherErrors = errors.filter(
		(message) =>
			!isContractError(message) && !scheduleMessages.includes(message),
	)
	const blocked = errors.length > 0

	function submit(event: FormEvent) {
		event.preventDefault()
		if (blocked) return
		if (contract)
			updateRh((current) =>
				updateContract(current, member.id, contract.id, final),
			)
		else
			updateRh((current, context) =>
				addContract(current, context, member.id, final),
			)
		onClose()
	}

	return (
		<form onSubmit={submit} noValidate className="flex min-h-0 flex-1 flex-col">
			<DialogHeader className="px-6 pt-6 pb-4">
				<DialogTitle>
					{contract ? 'Modifier le contrat' : 'Ajouter un contrat'}
				</DialogTitle>
				<DialogDescription>
					{contract
						? `${CONTRACT_LABELS[contract.type]}, ${contractPeriod(contract)}.`
						: `Renouvellement, passage en CDI, avenant d’horaires… Les dates ne doivent pas chevaucher les autres contrats de ${fullName(member)}.`}
				</DialogDescription>
			</DialogHeader>
			<div className="grid min-h-0 flex-1 gap-4 overflow-y-auto border-t px-6 py-5">
				<ContractFields
					idPrefix={idPrefix}
					value={draft}
					onChange={setDraft}
					errors={errors}
				/>
				{otherErrors.length > 0 && (
					<ul className="grid gap-1 text-[13px] text-state-alert">
						{otherErrors.map((message) => (
							<li key={message}>{message}</li>
						))}
					</ul>
				)}
			</div>
			<div className="flex items-center justify-between gap-3 border-t px-6 py-3">
				<p className="text-[12px] text-muted-foreground">
					{blocked
						? 'Complète les champs marqués d’une * ou corrige les erreurs pour continuer.'
						: ''}
				</p>
				<div className="flex gap-2">
					<DialogClose asChild>
						<Button type="button" variant="outline">
							Annuler
						</Button>
					</DialogClose>
					<Button type="submit" disabled={blocked}>
						{contract ? 'Enregistrer' : 'Ajouter le contrat'}
					</Button>
				</div>
			</div>
		</form>
	)
}

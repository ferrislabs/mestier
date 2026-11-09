import { useState } from 'react'
import { FormField } from '#/components/form-field'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import type { SelectOption } from '#/pages/cursus/shared/lib/options'
import { OptionSelect } from '#/pages/cursus/shared/ui/option-select'
import {
	addTutor,
	removeTutor,
	setMainTutor,
	type TutorEntry,
} from '#/pages/cursus/triads/lib/contract-tutors'

interface ContractTutorsDialogProps {
	open: boolean
	hostCompanyName: string
	initial: TutorEntry[]
	tutors: SelectOption[]
	isLoadingOptions: boolean
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onSubmit: (tutors: TutorEntry[]) => void
}

export function ContractTutorsDialog({
	open,
	onOpenChange,
	...body
}: ContractTutorsDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="sm:max-w-md">
				<ContractTutorsBody {...body} onCancel={() => onOpenChange(false)} />
			</DialogContent>
		</Dialog>
	)
}

interface ContractTutorsBodyProps
	extends Omit<ContractTutorsDialogProps, 'open' | 'onOpenChange'> {
	onCancel: () => void
}

function ContractTutorsBody({
	hostCompanyName,
	initial,
	tutors,
	isLoadingOptions,
	isPending,
	error,
	onCancel,
	onSubmit,
}: ContractTutorsBodyProps) {
	const [entries, setEntries] = useState(initial)
	const labelOf = (id: string) =>
		tutors.find((tutor) => tutor.value === id)?.label ?? id
	const available = tutors.filter(
		(tutor) => !entries.some((entry) => entry.companyTutorId === tutor.value),
	)

	return (
		<form
			noValidate
			className="grid gap-4"
			onSubmit={(event) => {
				event.preventDefault()
				if (isPending) return
				onSubmit(entries)
			}}
		>
			<DialogHeader>
				<DialogTitle>Tuteurs du contrat</DialogTitle>
				<DialogDescription>
					Tuteurs de {hostCompanyName} uniquement.
				</DialogDescription>
			</DialogHeader>
			<FormField
				id="tutor-add"
				label="Tuteurs entreprise"
				hint={
					isLoadingOptions
						? 'Chargement des tuteurs…'
						: tutors.length === 0
							? 'Aucun tuteur déclaré pour cette entreprise.'
							: undefined
				}
			>
				<OptionSelect
					id="tutor-add"
					value={null}
					options={available}
					placeholder="Ajouter un tuteur"
					onChange={(next) => {
						if (next) setEntries((previous) => addTutor(previous, next))
					}}
				/>
			</FormField>
			{entries.length === 0 ? (
				<p className="text-[13px] text-muted-foreground">Aucun tuteur.</p>
			) : (
				<ul className="grid gap-2">
					{entries.map((entry) => (
						<li
							key={entry.companyTutorId}
							className="flex items-center justify-between gap-2 rounded-md border px-3 py-2 text-[13px]"
						>
							<span className="flex items-center gap-2">
								{labelOf(entry.companyTutorId)}
								{entry.isMain ? <Badge>Principal</Badge> : null}
							</span>
							<span className="flex gap-2">
								{entry.isMain ? null : (
									<Button
										type="button"
										variant="outline"
										size="sm"
										aria-label={`Rendre ${labelOf(entry.companyTutorId)} principal`}
										onClick={() =>
											setEntries((previous) =>
												setMainTutor(previous, entry.companyTutorId),
											)
										}
									>
										Principal
									</Button>
								)}
								<Button
									type="button"
									variant="ghost"
									size="sm"
									aria-label={`Retirer ${labelOf(entry.companyTutorId)}`}
									onClick={() =>
										setEntries((previous) =>
											removeTutor(previous, entry.companyTutorId),
										)
									}
								>
									Retirer
								</Button>
							</span>
						</li>
					))}
				</ul>
			)}
			{error ? (
				<p role="alert" className="text-[13px] text-state-alert">
					{error}
				</p>
			) : null}
			<DialogFooter>
				<Button type="button" variant="outline" onClick={onCancel}>
					Annuler
				</Button>
				<Button type="submit" disabled={isPending}>
					Enregistrer
				</Button>
			</DialogFooter>
		</form>
	)
}

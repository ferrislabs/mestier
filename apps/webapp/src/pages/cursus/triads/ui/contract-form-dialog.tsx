import { type FormEvent, useState } from 'react'
import { FormField } from '#/components/form-field'
import { RichTextEditor } from '#/components/rich-text'
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
import {
	CONTRACT_KIND_LABELS,
	CONTRACT_KINDS,
} from '#/pages/cursus/shared/lib/format'
import type { SelectOption } from '#/pages/cursus/shared/lib/options'
import { OptionSelect } from '#/pages/cursus/shared/ui/option-select'
import {
	type ContractFormValues,
	contractFormErrors,
} from '#/pages/cursus/triads/lib/contract-form'

const KIND_OPTIONS: SelectOption[] = CONTRACT_KINDS.map((kind) => ({
	value: kind,
	label: CONTRACT_KIND_LABELS[kind],
}))

interface ContractFormDialogProps {
	open: boolean
	title: string
	submitLabel: string
	initial: ContractFormValues
	hostCompanies: SelectOption[]
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onSubmit: (values: ContractFormValues) => void
}

export function ContractFormDialog({
	open,
	onOpenChange,
	...body
}: ContractFormDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-xl">
				<ContractFormBody {...body} onCancel={() => onOpenChange(false)} />
			</DialogContent>
		</Dialog>
	)
}

interface ContractFormBodyProps
	extends Omit<ContractFormDialogProps, 'open' | 'onOpenChange'> {
	onCancel: () => void
}

function ContractFormBody({
	title,
	submitLabel,
	initial,
	hostCompanies,
	isPending,
	error,
	onCancel,
	onSubmit,
}: ContractFormBodyProps) {
	const [values, setValues] = useState(initial)
	const [revealed, setRevealed] = useState(false)
	const errors = contractFormErrors(values)
	const shown = revealed ? errors : {}

	function patch(next: Partial<ContractFormValues>) {
		setValues((previous) => ({ ...previous, ...next }))
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		setRevealed(true)
		if (Object.keys(errors).length > 0 || isPending) return
		onSubmit(values)
	}

	return (
		<form onSubmit={submit} noValidate className="grid gap-4">
			<DialogHeader>
				<DialogTitle>{title}</DialogTitle>
				<DialogDescription>
					Le contrat lie l’apprenti à une entreprise d’accueil.
				</DialogDescription>
			</DialogHeader>
			<FormField
				id="contract-company"
				label="Entreprise d’accueil"
				required
				error={shown.hostCompanyId}
			>
				<OptionSelect
					id="contract-company"
					value={values.hostCompanyId || null}
					options={hostCompanies}
					placeholder="Choisir une entreprise"
					onChange={(next) => patch({ hostCompanyId: next ?? '' })}
				/>
			</FormField>
			<FormField id="contract-kind" label="Type de contrat" required>
				<OptionSelect
					id="contract-kind"
					value={values.kind}
					options={KIND_OPTIONS}
					placeholder="Choisir un type"
					onChange={(next) => {
						const kind = CONTRACT_KINDS.find((entry) => entry === next)
						if (kind) patch({ kind })
					}}
				/>
			</FormField>
			<FormField id="contract-number" label="Numéro de contrat">
				<Input
					id="contract-number"
					value={values.number}
					onChange={(event) => patch({ number: event.target.value })}
				/>
			</FormField>
			<div className="grid grid-cols-2 gap-3">
				<FormField
					id="contract-start"
					label="Date de début"
					required
					error={shown.startDate}
				>
					<Input
						id="contract-start"
						type="date"
						value={values.startDate}
						onChange={(event) => patch({ startDate: event.target.value })}
					/>
				</FormField>
				<FormField
					id="contract-end"
					label="Date de fin"
					required
					error={shown.endDate}
				>
					<Input
						id="contract-end"
						type="date"
						value={values.endDate}
						onChange={(event) => patch({ endDate: event.target.value })}
					/>
				</FormField>
			</div>
			<FormField id="contract-duration" label="Durée (en mois)">
				<p id="contract-duration" className="text-[13px] text-muted-foreground">
					Calculée à partir des dates
				</p>
			</FormField>
			<FormField id="contract-comment" label="Commentaire">
				<RichTextEditor
					id="contract-comment"
					aria-label="Commentaire"
					value={values.comment}
					onChange={(comment) => patch({ comment })}
				/>
			</FormField>
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
					{submitLabel}
				</Button>
			</DialogFooter>
		</form>
	)
}

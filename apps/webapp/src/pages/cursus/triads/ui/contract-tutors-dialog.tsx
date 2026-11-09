import { useState } from 'react'
import { FormField } from '#/components/form-field'
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

export interface ContractTutorsValues {
	main: string | null
	secondary: string | null
}

interface ContractTutorsDialogProps {
	open: boolean
	hostCompanyName: string
	initial: ContractTutorsValues
	tutors: SelectOption[]
	isLoadingOptions: boolean
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onSubmit: (values: ContractTutorsValues) => void
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
	const [values, setValues] = useState(initial)
	const sameTutor = values.main !== null && values.main === values.secondary

	return (
		<form
			noValidate
			className="grid gap-4"
			onSubmit={(event) => {
				event.preventDefault()
				if (sameTutor || isPending) return
				onSubmit(values)
			}}
		>
			<DialogHeader>
				<DialogTitle>Tuteurs du contrat</DialogTitle>
				<DialogDescription>
					Tuteurs de {hostCompanyName} uniquement.
				</DialogDescription>
			</DialogHeader>
			<FormField
				id="tutor-main"
				label="Tuteur principal"
				hint={
					isLoadingOptions
						? 'Chargement des tuteurs…'
						: tutors.length === 0
							? 'Aucun tuteur déclaré pour cette entreprise.'
							: undefined
				}
			>
				<OptionSelect
					id="tutor-main"
					value={values.main}
					options={tutors}
					placeholder="Aucun"
					noneLabel="Aucun"
					onChange={(next) =>
						setValues((previous) => ({ ...previous, main: next }))
					}
				/>
			</FormField>
			<FormField
				id="tutor-secondary"
				label="Tuteur secondaire"
				error={sameTutor ? 'Choisissez deux tuteurs différents.' : undefined}
			>
				<OptionSelect
					id="tutor-secondary"
					value={values.secondary}
					options={tutors}
					placeholder="Aucun"
					noneLabel="Aucun"
					onChange={(next) =>
						setValues((previous) => ({ ...previous, secondary: next }))
					}
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
				<Button type="submit" disabled={isPending || sameTutor}>
					Enregistrer
				</Button>
			</DialogFooter>
		</form>
	)
}

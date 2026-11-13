import { type FormEvent, useState } from 'react'
import { FormField } from '#/components/form-field'
import { RichTextEditor } from '#/components/rich-text'
import {
	Dialog,
	DialogContent,
	DialogDescription,
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
import { StepIndicator } from '#/pages/cursus/shared/ui/step-indicator'
import { StepperFooter } from '#/pages/cursus/shared/ui/stepper-footer'
import { useSteps } from '#/pages/cursus/shared/ui/use-steps'
import {
	CONTRACT_STEP_FIELDS,
	type ContractFormValues,
	contractFormErrors,
	contractStepErrors,
} from '#/pages/cursus/triads/lib/contract-form'

const KIND_OPTIONS: SelectOption[] = CONTRACT_KINDS.map((kind) => ({
	value: kind,
	label: CONTRACT_KIND_LABELS[kind],
}))

const STEPS = ['Contrat', 'Période', 'Commentaire'] as const

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
	const steps = useSteps(CONTRACT_STEP_FIELDS.length)
	const shown = revealed ? contractStepErrors(values, steps.index) : {}

	function patch(next: Partial<ContractFormValues>) {
		setValues((previous) => ({ ...previous, ...next }))
	}

	function goBack(index: number) {
		setRevealed(false)
		steps.goTo(index)
	}

	function advance(event: FormEvent) {
		event.preventDefault()
		setRevealed(true)
		const errors = steps.isLast
			? contractFormErrors(values)
			: contractStepErrors(values, steps.index)
		if (Object.keys(errors).length > 0) return
		if (!steps.isLast) {
			setRevealed(false)
			steps.next()
			return
		}
		if (!isPending) onSubmit(values)
	}

	return (
		<form onSubmit={advance} noValidate className="grid gap-4">
			<DialogHeader>
				<DialogTitle>{title}</DialogTitle>
				<DialogDescription>
					Le contrat lie l’apprenti à une entreprise d’accueil.
				</DialogDescription>
			</DialogHeader>
			<StepIndicator steps={STEPS} current={steps.index} onSelect={goBack} />
			{steps.index === 0 ? (
				<>
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
				</>
			) : null}
			{steps.index === 1 ? (
				<>
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
						<p
							id="contract-duration"
							className="text-[13px] text-muted-foreground"
						>
							Calculée à partir des dates
						</p>
					</FormField>
				</>
			) : null}
			{steps.index === 2 ? (
				<FormField id="contract-comment" label="Commentaire">
					<RichTextEditor
						id="contract-comment"
						aria-label="Commentaire"
						value={values.comment}
						onChange={(comment) => patch({ comment })}
					/>
				</FormField>
			) : null}
			{steps.isLast && error ? (
				<p role="alert" className="text-[13px] text-state-alert">
					{error}
				</p>
			) : null}
			<StepperFooter
				isFirst={steps.isFirst}
				isLast={steps.isLast}
				submitLabel={submitLabel}
				isPending={isPending}
				onCancel={onCancel}
				onBack={() => goBack(steps.index - 1)}
			/>
		</form>
	)
}

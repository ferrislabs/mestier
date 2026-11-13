import { type FormEvent, useState } from 'react'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import {
	EMPTY_HOST_COMPANY,
	HOST_COMPANY_STEP_COUNT,
	type HostCompanyValues,
	validateHostCompany,
	validateHostCompanyStep,
} from '#/pages/cursus/host-companies/lib/host-companies'
import {
	HostCompanyActivityFields,
	HostCompanyContactFields,
	HostCompanyLegalFields,
} from '#/pages/cursus/host-companies/ui/host-company-sections'
import { FormError } from '#/pages/cursus/participants/ui/field-input'
import { StepIndicator } from '#/pages/cursus/shared/ui/step-indicator'
import { StepperFooter } from '#/pages/cursus/shared/ui/stepper-footer'
import { useSteps } from '#/pages/cursus/shared/ui/use-steps'

const STEPS = ['Identité légale', 'Coordonnées', 'Activité'] as const

export interface CreateHostCompanyDialogProps {
	open: boolean
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onSubmit: (values: HostCompanyValues) => void
}

export function CreateHostCompanyDialog({
	open,
	onOpenChange,
	...body
}: CreateHostCompanyDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-2xl">
				<DialogHeader>
					<DialogTitle>Nouvelle entreprise d'accueil</DialogTitle>
					<DialogDescription>
						Renseignez l'entreprise qui accueille des apprentis.
					</DialogDescription>
				</DialogHeader>
				<CreateHostCompanyBody {...body} onCancel={() => onOpenChange(false)} />
			</DialogContent>
		</Dialog>
	)
}

interface CreateHostCompanyBodyProps
	extends Omit<CreateHostCompanyDialogProps, 'open' | 'onOpenChange'> {
	onCancel: () => void
}

function CreateHostCompanyBody({
	isPending,
	error,
	onCancel,
	onSubmit,
}: CreateHostCompanyBodyProps) {
	const [values, setValues] = useState(EMPTY_HOST_COMPANY)
	const [formError, setFormError] = useState<string | null>(null)
	const steps = useSteps(HOST_COMPANY_STEP_COUNT)

	function patch(change: Partial<HostCompanyValues>) {
		setValues((previous) => ({ ...previous, ...change }))
	}

	function goBack(index: number) {
		setFormError(null)
		steps.goTo(index)
	}

	function advance(event: FormEvent) {
		event.preventDefault()
		const problem = steps.isLast
			? validateHostCompany(values)
			: validateHostCompanyStep(values, steps.index)
		setFormError(problem)
		if (problem !== null) return
		if (steps.isLast) onSubmit(values)
		else steps.next()
	}

	return (
		<form className="flex flex-col gap-4" onSubmit={advance}>
			<StepIndicator steps={STEPS} current={steps.index} onSelect={goBack} />
			{steps.index === 0 ? (
				<HostCompanyLegalFields values={values} onChange={patch} />
			) : null}
			{steps.index === 1 ? (
				<HostCompanyContactFields values={values} onChange={patch} />
			) : null}
			{steps.index === 2 ? (
				<HostCompanyActivityFields values={values} onChange={patch} />
			) : null}
			<FormError message={formError ?? (steps.isLast ? error : null)} />
			<StepperFooter
				isFirst={steps.isFirst}
				isLast={steps.isLast}
				submitLabel="Créer l'entreprise"
				isPending={isPending}
				onCancel={onCancel}
				onBack={() => goBack(steps.index - 1)}
			/>
		</form>
	)
}

import { useForm } from '@tanstack/react-form'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	EMPTY_HOST_COMPANY,
	type HostCompanyValues,
	validateHostCompany,
} from '#/pages/cursus/host-companies/lib/host-companies'
import {
	FieldInput,
	FormError,
} from '#/pages/cursus/participants/ui/field-input'

export interface HostCompanyFormProps {
	initialValues?: HostCompanyValues
	disabled?: boolean
	submitLabel: string
	isPending: boolean
	error: string | null
	onSubmit: (values: HostCompanyValues) => void
}

export function HostCompanyForm({
	initialValues = EMPTY_HOST_COMPANY,
	disabled = false,
	submitLabel,
	isPending,
	error,
	onSubmit,
}: HostCompanyFormProps) {
	const [formError, setFormError] = useState<string | null>(null)
	const form = useForm({
		defaultValues: initialValues,
		onSubmit: ({ value }) => {
			const problem = validateHostCompany(value)
			setFormError(problem)
			if (problem === null) onSubmit(value)
		},
	})

	return (
		<form
			className="flex flex-col gap-3"
			onSubmit={(event) => {
				event.preventDefault()
				void form.handleSubmit()
			}}
		>
			<div className="grid grid-cols-2 gap-3">
				<div className="col-span-2">
					<form.Field name="name">
						{(field) => (
							<FieldInput
								id="host-company-name"
								label="Raison sociale"
								value={field.state.value}
								disabled={disabled}
								onChange={field.handleChange}
							/>
						)}
					</form.Field>
				</div>
				<form.Field name="siret">
					{(field) => (
						<FieldInput
							id="host-company-siret"
							label="SIRET"
							value={field.state.value}
							disabled={disabled}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="nafCode">
					{(field) => (
						<FieldInput
							id="host-company-naf"
							label="Code NAF"
							value={field.state.value}
							disabled={disabled}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="legalForm">
					{(field) => (
						<FieldInput
							id="host-company-legal-form"
							label="Forme juridique"
							value={field.state.value}
							disabled={disabled}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="employeeCount">
					{(field) => (
						<FieldInput
							id="host-company-employees"
							label="Effectif"
							type="number"
							value={field.state.value}
							disabled={disabled}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="directorName">
					{(field) => (
						<FieldInput
							id="host-company-director"
							label="Dirigeant"
							value={field.state.value}
							disabled={disabled}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="address">
					{(field) => (
						<FieldInput
							id="host-company-address"
							label="Adresse"
							value={field.state.value}
							disabled={disabled}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
			</div>
			<FormError message={formError ?? error} />
			{disabled ? null : (
				<div>
					<Button type="submit" disabled={isPending}>
						{submitLabel}
					</Button>
				</div>
			)}
		</form>
	)
}

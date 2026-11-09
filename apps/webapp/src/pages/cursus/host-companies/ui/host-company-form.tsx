import { useForm } from '@tanstack/react-form'
import { useState } from 'react'
import { RichTextEditor } from '#/components/rich-text'
import { Button } from '#/components/ui/button'
import { Field } from '#/components/ui/field'
import { Textarea } from '#/components/ui/textarea'
import {
	EMPTY_HOST_COMPANY,
	type HostCompanyValues,
	validateHostCompany,
} from '#/pages/cursus/host-companies/lib/host-companies'
import { AddressListEditor } from '#/pages/cursus/host-companies/ui/address-list-editor'
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
								label="Nom de l'entreprise"
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
							label="Numéro de SIRET"
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
				<form.Field name="groupName">
					{(field) => (
						<FieldInput
							id="host-company-group"
							label="Groupe"
							value={field.state.value}
							disabled={disabled}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="capital">
					{(field) => (
						<FieldInput
							id="host-company-capital"
							label="Capital"
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
							label="Nom du directeur"
							value={field.state.value}
							disabled={disabled}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="phone">
					{(field) => (
						<FieldInput
							id="host-company-phone"
							label="Téléphone"
							type="tel"
							value={field.state.value}
							disabled={disabled}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="email">
					{(field) => (
						<FieldInput
							id="host-company-email"
							label="Email"
							type="email"
							value={field.state.value}
							disabled={disabled}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="website">
					{(field) => (
						<FieldInput
							id="host-company-website"
							label="URL site internet"
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
							label="Nombre d'employés dans l'entreprise"
							type="number"
							value={field.state.value}
							disabled={disabled}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="groupEmployeeCount">
					{(field) => (
						<FieldInput
							id="host-company-group-employees"
							label="Nombre d'employés dans le groupe"
							type="number"
							value={field.state.value}
							disabled={disabled}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<div className="col-span-2">
					<form.Field name="activityDomains">
						{(field) => (
							<Field
								label="Domaines d'activité"
								htmlFor="host-company-activities"
							>
								<Textarea
									id="host-company-activities"
									value={field.state.value}
									disabled={disabled}
									onChange={(event) => field.handleChange(event.target.value)}
								/>
							</Field>
						)}
					</form.Field>
				</div>
				<div className="col-span-2">
					<form.Field name="missions">
						{(field) => (
							<Field
								label="Missions et objectifs"
								htmlFor="host-company-missions"
							>
								<RichTextEditor
									id="host-company-missions"
									aria-label="Missions et objectifs"
									value={field.state.value}
									disabled={disabled}
									onChange={field.handleChange}
								/>
							</Field>
						)}
					</form.Field>
				</div>
			</div>
			<form.Field name="addresses">
				{(field) => (
					<AddressListEditor
						addresses={field.state.value}
						disabled={disabled}
						onChange={field.handleChange}
					/>
				)}
			</form.Field>
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

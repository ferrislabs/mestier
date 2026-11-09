import { useForm } from '@tanstack/react-form'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { Checkbox } from '#/components/ui/checkbox'
import { Field } from '#/components/ui/field'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	FieldInput,
	FormError,
} from '#/pages/cursus/participants/ui/field-input'
import {
	type ClosureReasonValues,
	type EstablishmentValues,
	type TrainingValues,
	validateClosureReason,
	validateEstablishment,
	validateTraining,
} from '#/pages/cursus/settings/lib/catalog'

interface FormShellProps {
	isPending: boolean
	error: string | null
	formError: string | null
	submitLabel: string
	onSubmit: () => void
	children: React.ReactNode
}

function FormShell({
	isPending,
	error,
	formError,
	submitLabel,
	onSubmit,
	children,
}: FormShellProps) {
	return (
		<form
			className="flex flex-col gap-3"
			onSubmit={(event) => {
				event.preventDefault()
				onSubmit()
			}}
		>
			{children}
			<FormError message={formError ?? error} />
			<div>
				<Button type="submit" disabled={isPending}>
					{submitLabel}
				</Button>
			</div>
		</form>
	)
}

export interface CatalogFormProps<Values> {
	initialValues: Values
	isPending: boolean
	error: string | null
	onSubmit: (values: Values) => void
}

export function EstablishmentForm({
	initialValues,
	isPending,
	error,
	onSubmit,
}: CatalogFormProps<EstablishmentValues>) {
	const [formError, setFormError] = useState<string | null>(null)
	const form = useForm({
		defaultValues: initialValues,
		onSubmit: ({ value }) => {
			const problem = validateEstablishment(value)
			setFormError(problem)
			if (problem === null) onSubmit(value)
		},
	})

	return (
		<FormShell
			isPending={isPending}
			error={error}
			formError={formError}
			submitLabel="Enregistrer"
			onSubmit={() => void form.handleSubmit()}
		>
			<form.Field name="name">
				{(field) => (
					<FieldInput
						id="establishment-name"
						label="Nom"
						value={field.state.value}
						onChange={field.handleChange}
					/>
				)}
			</form.Field>
			<div className="grid grid-cols-2 gap-3">
				<form.Field name="uai">
					{(field) => (
						<FieldInput
							id="establishment-uai"
							label="UAI"
							value={field.state.value}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="siret">
					{(field) => (
						<FieldInput
							id="establishment-siret"
							label="SIRET"
							value={field.state.value}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
			</div>
			<form.Field name="address">
				{(field) => (
					<FieldInput
						id="establishment-address"
						label="Adresse"
						value={field.state.value}
						onChange={field.handleChange}
					/>
				)}
			</form.Field>
			<form.Field name="isMainSite">
				{(field) => (
					<div className="flex items-center gap-2">
						<Checkbox
							id="establishment-main-site"
							checked={field.state.value}
							onCheckedChange={(checked) =>
								field.handleChange(checked === true)
							}
						/>
						<Label htmlFor="establishment-main-site">Site principal</Label>
					</div>
				)}
			</form.Field>
		</FormShell>
	)
}

export interface TrainingFormProps extends CatalogFormProps<TrainingValues> {
	establishments: { id: string; name: string }[]
}

export function TrainingForm({
	initialValues,
	establishments,
	isPending,
	error,
	onSubmit,
}: TrainingFormProps) {
	const [formError, setFormError] = useState<string | null>(null)
	const form = useForm({
		defaultValues: initialValues,
		onSubmit: ({ value }) => {
			const problem = validateTraining(value)
			setFormError(problem)
			if (problem === null) onSubmit(value)
		},
	})

	return (
		<FormShell
			isPending={isPending}
			error={error}
			formError={formError}
			submitLabel="Enregistrer"
			onSubmit={() => void form.handleSubmit()}
		>
			<form.Field name="establishmentId">
				{(field) => (
					<Field label="Établissement" htmlFor="training-establishment">
						<Select
							value={field.state.value}
							onValueChange={field.handleChange}
						>
							<SelectTrigger id="training-establishment">
								<SelectValue placeholder="Choisir…" />
							</SelectTrigger>
							<SelectContent>
								{establishments.map((establishment) => (
									<SelectItem key={establishment.id} value={establishment.id}>
										{establishment.name}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</Field>
				)}
			</form.Field>
			<form.Field name="name">
				{(field) => (
					<FieldInput
						id="training-name"
						label="Nom"
						value={field.state.value}
						onChange={field.handleChange}
					/>
				)}
			</form.Field>
			<div className="grid grid-cols-2 gap-3">
				<form.Field name="rncpCode">
					{(field) => (
						<FieldInput
							id="training-rncp-code"
							label="Code RNCP"
							value={field.state.value}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="rncpLabel">
					{(field) => (
						<FieldInput
							id="training-rncp-label"
							label="Libellé RNCP"
							value={field.state.value}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="level">
					{(field) => (
						<FieldInput
							id="training-level"
							label="Niveau"
							value={field.state.value}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
				<form.Field name="degreeType">
					{(field) => (
						<FieldInput
							id="training-degree-type"
							label="Type de diplôme"
							value={field.state.value}
							onChange={field.handleChange}
						/>
					)}
				</form.Field>
			</div>
		</FormShell>
	)
}

export function ClosureReasonForm({
	initialValues,
	isPending,
	error,
	onSubmit,
}: CatalogFormProps<ClosureReasonValues>) {
	const [formError, setFormError] = useState<string | null>(null)
	const form = useForm({
		defaultValues: initialValues,
		onSubmit: ({ value }) => {
			const problem = validateClosureReason(value)
			setFormError(problem)
			if (problem === null) onSubmit(value)
		},
	})

	return (
		<FormShell
			isPending={isPending}
			error={error}
			formError={formError}
			submitLabel="Enregistrer"
			onSubmit={() => void form.handleSubmit()}
		>
			<form.Field name="label">
				{(field) => (
					<FieldInput
						id="closure-reason-label"
						label="Libellé"
						value={field.state.value}
						onChange={field.handleChange}
					/>
				)}
			</form.Field>
		</FormShell>
	)
}

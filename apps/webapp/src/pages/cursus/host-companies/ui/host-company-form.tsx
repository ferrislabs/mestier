import { type FormEvent, useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	EMPTY_HOST_COMPANY,
	type HostCompanyValues,
	validateHostCompany,
} from '#/pages/cursus/host-companies/lib/host-companies'
import {
	HostCompanyActivityFields,
	HostCompanyContactFields,
	HostCompanyLegalFields,
} from '#/pages/cursus/host-companies/ui/host-company-sections'
import { FormError } from '#/pages/cursus/participants/ui/field-input'

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
	const [values, setValues] = useState(initialValues)
	const [formError, setFormError] = useState<string | null>(null)

	function patch(change: Partial<HostCompanyValues>) {
		setValues((previous) => ({ ...previous, ...change }))
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		const problem = validateHostCompany(values)
		setFormError(problem)
		if (problem === null) onSubmit(values)
	}

	return (
		<form className="flex flex-col gap-3" onSubmit={submit}>
			<HostCompanyLegalFields
				values={values}
				disabled={disabled}
				onChange={patch}
			/>
			<HostCompanyContactFields
				values={values}
				disabled={disabled}
				onChange={patch}
			/>
			<HostCompanyActivityFields
				values={values}
				disabled={disabled}
				onChange={patch}
			/>
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

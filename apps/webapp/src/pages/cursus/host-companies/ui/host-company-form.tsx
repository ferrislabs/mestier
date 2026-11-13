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
	useHostCompanyForm,
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
	const [formError, setFormError] = useState<string | null>(null)
	const form = useHostCompanyForm(initialValues, onSubmit)

	function submit(event: FormEvent) {
		event.preventDefault()
		const problem = validateHostCompany(form.state.values)
		setFormError(problem)
		if (problem === null) void form.handleSubmit()
	}

	return (
		<form className="flex flex-col gap-3" onSubmit={submit}>
			<HostCompanyLegalFields form={form} disabled={disabled} />
			<HostCompanyContactFields form={form} disabled={disabled} />
			<HostCompanyActivityFields form={form} disabled={disabled} />
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

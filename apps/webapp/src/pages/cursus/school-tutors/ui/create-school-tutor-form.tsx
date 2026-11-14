import { useForm } from '@tanstack/react-form'
import { type FormEvent, useState } from 'react'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import {
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import {
	FieldInput,
	FormError,
} from '#/pages/cursus/participants/ui/field-input'
import {
	EMPTY_SCHOOL_TUTOR,
	type SchoolTutorValues,
} from '#/pages/cursus/school-tutors/lib/school-tutors'
import type { SelectOption } from '#/pages/cursus/shared/lib/options'
import { OptionSelect } from '#/pages/cursus/shared/ui/option-select'

export interface CreateSchoolTutorFormProps {
	members: SelectOption[]
	isLoadingMembers: boolean
	isPending: boolean
	error: string | null
	onBack: () => void
	onSubmit: (values: SchoolTutorValues) => void
}

export function CreateSchoolTutorForm({
	members,
	isLoadingMembers,
	isPending,
	error,
	onBack,
	onSubmit,
}: CreateSchoolTutorFormProps) {
	const [memberError, setMemberError] = useState<string | undefined>()
	const form = useForm({
		defaultValues: EMPTY_SCHOOL_TUTOR,
		onSubmit: ({ value }) => onSubmit(value),
	})

	function submit(event: FormEvent) {
		event.preventDefault()
		if (form.state.values.memberId === '') {
			setMemberError('Choisissez un membre.')
			return
		}
		setMemberError(undefined)
		if (!isPending) void form.handleSubmit()
	}

	return (
		<form className="grid gap-4" onSubmit={submit} noValidate>
			<DialogHeader>
				<DialogTitle>Nouveau tuteur école</DialogTitle>
				<DialogDescription>
					Déclarez un membre de votre équipe comme tuteur école.
				</DialogDescription>
			</DialogHeader>
			<form.Field name="memberId">
				{(field) => (
					<FormField
						id="school-tutor-member"
						label="Membre"
						required
						error={memberError}
						hint={
							isLoadingMembers
								? 'Chargement des membres…'
								: members.length === 0
									? 'Tous les membres sont déjà tuteurs école.'
									: undefined
						}
					>
						<OptionSelect
							id="school-tutor-member"
							value={field.state.value || null}
							options={members}
							placeholder="Choisir un membre"
							onChange={(next) => field.handleChange(next ?? '')}
						/>
					</FormField>
				)}
			</form.Field>
			<form.Field name="jobTitle">
				{(field) => (
					<FieldInput
						id="school-tutor-job-title"
						label="Fonction"
						value={field.state.value}
						onChange={field.handleChange}
					/>
				)}
			</form.Field>
			<FormError message={error} />
			<DialogFooter>
				<Button type="button" variant="ghost" onClick={onBack}>
					Précédent
				</Button>
				<Button type="submit" disabled={isPending}>
					Créer le tuteur
				</Button>
			</DialogFooter>
		</form>
	)
}

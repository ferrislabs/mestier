import { FormField } from '#/components/form-field'
import { Input } from '#/components/ui/input'
import type { ContactForm } from '#/pages/customers/lib/account-form'
import type { BindField } from '#/pages/customers/ui/use-form-binding'

export function ContactFields({
	value,
	onChange,
	bind,
}: {
	value: ContactForm
	onChange: (next: ContactForm) => void
	bind: BindField
}) {
	const name = bind('contactName')
	const email = bind('contactEmail')
	const phone = bind('contactPhone')
	const nameInvalid = name.input['aria-invalid']
	const nameDescribedBy = name.input['aria-describedby']
	return (
		<>
			<FormField id={name.id} label="Prénom" error={name.error}>
				<Input
					id={name.id}
					value={value.firstName}
					onChange={(event) =>
						onChange({ ...value, firstName: event.target.value })
					}
					autoComplete="off"
					aria-invalid={nameInvalid}
					aria-describedby={nameDescribedBy}
					onBlur={name.input.onBlur}
				/>
			</FormField>
			<FormField id={`${name.id}-last`} label="Nom">
				<Input
					id={`${name.id}-last`}
					value={value.lastName}
					onChange={(event) =>
						onChange({ ...value, lastName: event.target.value })
					}
					autoComplete="off"
					aria-invalid={nameInvalid}
					aria-describedby={nameDescribedBy}
					onBlur={name.input.onBlur}
				/>
			</FormField>
			<FormField
				id={`${name.id}-role`}
				label="Rôle"
				className="sm:col-span-2"
				hint="Par exemple : gérant, conducteur de travaux, comptabilité."
			>
				<Input
					id={`${name.id}-role`}
					value={value.role}
					onChange={(event) => onChange({ ...value, role: event.target.value })}
					autoComplete="off"
				/>
			</FormField>
			<FormField
				id={email.id}
				label="E-mail"
				error={email.error}
				className="sm:col-span-2"
			>
				<Input
					{...email.input}
					type="email"
					value={value.email}
					onChange={(event) =>
						onChange({ ...value, email: event.target.value })
					}
					autoComplete="off"
					placeholder="prenom.nom@entreprise.fr"
				/>
			</FormField>
			<FormField
				id={phone.id}
				label="Téléphone"
				error={phone.error}
				className="sm:col-span-2"
			>
				<Input
					{...phone.input}
					type="tel"
					value={value.phone}
					onChange={(event) =>
						onChange({ ...value, phone: event.target.value })
					}
					autoComplete="off"
					placeholder="06 12 34 56 78"
				/>
			</FormField>
		</>
	)
}

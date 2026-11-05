import { FormField } from '#/components/form-field'
import { Input } from '#/components/ui/input'
import type { AddressForm } from '#/pages/customers/lib/account-form'
import type { BindField } from '#/pages/customers/ui/use-form-binding'

export function AddressFields({
	value,
	onChange,
	bind,
}: {
	value: AddressForm
	onChange: (next: AddressForm) => void
	bind: BindField
}) {
	const street = bind('street')
	const postalCode = bind('postalCode')
	const city = bind('city')
	return (
		<>
			<FormField
				id={street.id}
				label="Rue"
				error={street.error}
				className="sm:col-span-2"
			>
				<Input
					{...street.input}
					value={value.street}
					onChange={(event) =>
						onChange({ ...value, street: event.target.value })
					}
					autoComplete="off"
				/>
			</FormField>
			<FormField
				id={`${street.id}-complement`}
				label="Complément"
				className="sm:col-span-2"
			>
				<Input
					id={`${street.id}-complement`}
					value={value.complement}
					onChange={(event) =>
						onChange({ ...value, complement: event.target.value })
					}
					autoComplete="off"
					placeholder="Bâtiment, étage, lieu-dit…"
				/>
			</FormField>
			<FormField
				id={postalCode.id}
				label="Code postal"
				error={postalCode.error}
			>
				<Input
					{...postalCode.input}
					inputMode="numeric"
					value={value.postalCode}
					onChange={(event) =>
						onChange({ ...value, postalCode: event.target.value })
					}
					autoComplete="off"
				/>
			</FormField>
			<FormField id={city.id} label="Ville" error={city.error}>
				<Input
					{...city.input}
					value={value.city}
					onChange={(event) => onChange({ ...value, city: event.target.value })}
					autoComplete="off"
				/>
			</FormField>
		</>
	)
}

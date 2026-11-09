import { Button } from '#/components/ui/button'
import {
	type AddressValues,
	addAddress,
	removeAddress,
	setMainAddress,
	updateAddress,
} from '#/pages/cursus/host-companies/lib/host-companies'
import { FieldInput } from '#/pages/cursus/participants/ui/field-input'

export interface AddressListEditorProps {
	addresses: AddressValues[]
	disabled?: boolean
	onChange: (addresses: AddressValues[]) => void
}

export function AddressListEditor({
	addresses,
	disabled = false,
	onChange,
}: AddressListEditorProps) {
	return (
		<fieldset className="flex flex-col gap-3">
			<div className="flex items-center justify-between gap-3">
				<legend className="text-sm font-medium">Adresses</legend>
				{disabled ? null : (
					<Button
						type="button"
						variant="outline"
						size="sm"
						onClick={() => onChange(addAddress(addresses))}
					>
						Ajouter une adresse
					</Button>
				)}
			</div>
			{addresses.length === 0 ? (
				<p className="text-sm text-muted-foreground">Aucune adresse.</p>
			) : null}
			{addresses.map((address, index) => {
				const key = `host-company-address-${index}`
				const patch = (change: Partial<AddressValues>) =>
					onChange(updateAddress(addresses, index, change))
				return (
					<div
						// biome-ignore lint/suspicious/noArrayIndexKey: addresses have no stable id before being saved
						key={index}
						className="grid grid-cols-2 gap-3 rounded-lg border p-3"
					>
						<FieldInput
							id={`${key}-label`}
							label="Libellé"
							value={address.label}
							disabled={disabled}
							onChange={(label) => patch({ label })}
						/>
						<div className="col-span-2">
							<FieldInput
								id={`${key}-line1`}
								label="Adresse"
								value={address.line1}
								disabled={disabled}
								onChange={(line1) => patch({ line1 })}
							/>
						</div>
						<div className="col-span-2">
							<FieldInput
								id={`${key}-line2`}
								label="Adresse 2"
								value={address.line2}
								disabled={disabled}
								onChange={(line2) => patch({ line2 })}
							/>
						</div>
						<FieldInput
							id={`${key}-postal-code`}
							label="Code postal"
							value={address.postalCode}
							disabled={disabled}
							onChange={(postalCode) => patch({ postalCode })}
						/>
						<FieldInput
							id={`${key}-city`}
							label="Ville"
							value={address.city}
							disabled={disabled}
							onChange={(city) => patch({ city })}
						/>
						<div className="col-span-2 flex items-center justify-between gap-3">
							<label className="flex items-center gap-2 text-sm">
								<input
									type="radio"
									name="host-company-main-address"
									checked={address.isMain}
									disabled={disabled}
									aria-label={`Principale ${index + 1}`}
									onChange={() => onChange(setMainAddress(addresses, index))}
								/>
								Principale
							</label>
							{disabled ? null : (
								<Button
									type="button"
									variant="ghost"
									size="sm"
									aria-label={`Supprimer l'adresse ${index + 1}`}
									onClick={() => onChange(removeAddress(addresses, index))}
								>
									Supprimer
								</Button>
							)}
						</div>
					</div>
				)
			})}
		</fieldset>
	)
}

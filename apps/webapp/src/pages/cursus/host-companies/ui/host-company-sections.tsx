import { RichTextEditor } from '#/components/rich-text'
import { Field } from '#/components/ui/field'
import { Textarea } from '#/components/ui/textarea'
import type { HostCompanyValues } from '#/pages/cursus/host-companies/lib/host-companies'
import { AddressListEditor } from '#/pages/cursus/host-companies/ui/address-list-editor'
import { FieldInput } from '#/pages/cursus/participants/ui/field-input'

export interface HostCompanySectionProps {
	values: HostCompanyValues
	disabled?: boolean
	onChange: (patch: Partial<HostCompanyValues>) => void
}

export function HostCompanyLegalFields({
	values,
	disabled = false,
	onChange,
}: HostCompanySectionProps) {
	return (
		<div className="grid grid-cols-2 gap-3">
			<div className="col-span-2">
				<FieldInput
					id="host-company-name"
					label="Nom de l'entreprise"
					value={values.name}
					disabled={disabled}
					onChange={(name) => onChange({ name })}
				/>
			</div>
			<FieldInput
				id="host-company-siret"
				label="Numéro de SIRET"
				value={values.siret}
				disabled={disabled}
				onChange={(siret) => onChange({ siret })}
			/>
			<FieldInput
				id="host-company-naf"
				label="Code NAF"
				value={values.nafCode}
				disabled={disabled}
				onChange={(nafCode) => onChange({ nafCode })}
			/>
			<FieldInput
				id="host-company-legal-form"
				label="Forme juridique"
				value={values.legalForm}
				disabled={disabled}
				onChange={(legalForm) => onChange({ legalForm })}
			/>
			<FieldInput
				id="host-company-group"
				label="Groupe"
				value={values.groupName}
				disabled={disabled}
				onChange={(groupName) => onChange({ groupName })}
			/>
			<FieldInput
				id="host-company-capital"
				label="Capital"
				value={values.capital}
				disabled={disabled}
				onChange={(capital) => onChange({ capital })}
			/>
			<FieldInput
				id="host-company-director"
				label="Nom du directeur"
				value={values.directorName}
				disabled={disabled}
				onChange={(directorName) => onChange({ directorName })}
			/>
		</div>
	)
}

export function HostCompanyContactFields({
	values,
	disabled = false,
	onChange,
}: HostCompanySectionProps) {
	return (
		<div className="flex flex-col gap-3">
			<div className="grid grid-cols-2 gap-3">
				<FieldInput
					id="host-company-phone"
					label="Téléphone"
					type="tel"
					value={values.phone}
					disabled={disabled}
					onChange={(phone) => onChange({ phone })}
				/>
				<FieldInput
					id="host-company-email"
					label="Email"
					type="email"
					value={values.email}
					disabled={disabled}
					onChange={(email) => onChange({ email })}
				/>
				<div className="col-span-2">
					<FieldInput
						id="host-company-website"
						label="URL site internet"
						value={values.website}
						disabled={disabled}
						onChange={(website) => onChange({ website })}
					/>
				</div>
			</div>
			<AddressListEditor
				addresses={values.addresses}
				disabled={disabled}
				onChange={(addresses) => onChange({ addresses })}
			/>
		</div>
	)
}

export function HostCompanyActivityFields({
	values,
	disabled = false,
	onChange,
}: HostCompanySectionProps) {
	return (
		<div className="grid grid-cols-2 gap-3">
			<FieldInput
				id="host-company-employees"
				label="Nombre d'employés dans l'entreprise"
				type="number"
				value={values.employeeCount}
				disabled={disabled}
				onChange={(employeeCount) => onChange({ employeeCount })}
			/>
			<FieldInput
				id="host-company-group-employees"
				label="Nombre d'employés dans le groupe"
				type="number"
				value={values.groupEmployeeCount}
				disabled={disabled}
				onChange={(groupEmployeeCount) => onChange({ groupEmployeeCount })}
			/>
			<div className="col-span-2">
				<Field label="Domaines d'activité" htmlFor="host-company-activities">
					<Textarea
						id="host-company-activities"
						value={values.activityDomains}
						disabled={disabled}
						onChange={(event) =>
							onChange({ activityDomains: event.target.value })
						}
					/>
				</Field>
			</div>
			<div className="col-span-2">
				<Field label="Missions et objectifs" htmlFor="host-company-missions">
					<RichTextEditor
						id="host-company-missions"
						aria-label="Missions et objectifs"
						value={values.missions}
						disabled={disabled}
						onChange={(missions) => onChange({ missions })}
					/>
				</Field>
			</div>
		</div>
	)
}

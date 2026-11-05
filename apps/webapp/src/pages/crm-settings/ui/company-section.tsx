import { type FormEvent, useId, useState } from 'react'
import {
	type CompanyField,
	companyErrors,
} from '#/pages/crm-settings/lib/settings-model'
import type { Company, PostalAddress } from '#/pages/crm-settings/types'
import {
	SettingsCard,
	SettingsForm,
} from '#/pages/crm-settings/ui/settings-card'
import {
	TextAreaField,
	TextField,
} from '#/pages/crm-settings/ui/settings-fields'
import { useSyncedDraft } from '#/pages/crm-settings/ui/use-synced-draft'

const FIELD_ORDER: readonly CompanyField[] = [
	'name',
	'legalId',
	'vatNumber',
	'street',
	'city',
]

export function CompanySection({
	stored,
	onSave,
}: {
	stored: Company
	onSave: (company: Company) => Promise<void>
}) {
	const baseId = useId()
	const id = (field: string) => `${baseId}-${field}`
	const [draft, setDraft, dirty] = useSyncedDraft(stored)
	const [showErrors, setShowErrors] = useState(false)
	const [saved, setSaved] = useState(false)
	const errors = companyErrors(draft)
	const shown = showErrors ? errors : { fields: {}, other: [] }

	function change(patch: Partial<Company>) {
		setDraft((current) => ({ ...current, ...patch }))
		setSaved(false)
	}

	function changeAddress(patch: Partial<PostalAddress>) {
		setDraft((current) => ({
			...current,
			address: { ...current.address, ...patch },
		}))
		setSaved(false)
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		setShowErrors(true)
		const first = FIELD_ORDER.find((field) => errors.fields[field])
		if (first || errors.other.length > 0) {
			if (first) document.getElementById(id(first))?.focus()
			return
		}
		void onSave(draft).then(() => {
			setShowErrors(false)
			setSaved(true)
		})
	}

	return (
		<SettingsCard
			id="entreprise"
			title="Entreprise"
			description="Ces mentions sont reprises sur chaque devis et facture. Elles sont figées à l’émission : les documents déjà émis ne changent pas."
		>
			<SettingsForm onSubmit={submit} dirty={dirty} saved={saved}>
				<div className="grid gap-4 sm:grid-cols-2">
					<TextField
						id={id('name')}
						label="Raison sociale"
						required
						value={draft.name}
						onChange={(name) => change({ name })}
						error={shown.fields.name}
					/>
					<TextField
						id={id('legalForm')}
						label="Forme juridique"
						value={draft.legalForm}
						onChange={(legalForm) => change({ legalForm })}
						placeholder="SARL, SAS, EI…"
					/>
					<TextField
						id={id('capital')}
						label="Capital social"
						value={draft.capital}
						onChange={(capital) => change({ capital })}
						placeholder="50 000 €"
					/>
					<TextField
						id={id('legalId')}
						label="SIRET"
						required
						value={draft.legalId}
						onChange={(legalId) => change({ legalId })}
						error={shown.fields.legalId}
						inputMode="numeric"
						placeholder="14 chiffres"
					/>
					<TextField
						id={id('registry')}
						label="Immatriculation"
						value={draft.registry}
						onChange={(registry) => change({ registry })}
						placeholder="RCS Nantes B 823 456 789"
					/>
					<TextField
						id={id('vatNumber')}
						label="N° de TVA intracommunautaire"
						required
						value={draft.vatNumber}
						onChange={(vatNumber) => change({ vatNumber })}
						error={shown.fields.vatNumber}
						placeholder="FR12823456789"
					/>
				</div>

				<fieldset className="grid gap-4 sm:grid-cols-2">
					<legend className="mb-3 font-medium text-sm">Adresse du siège</legend>
					<TextField
						id={id('street')}
						label="Rue"
						required
						className="sm:col-span-2"
						value={draft.address.street}
						onChange={(street) => changeAddress({ street })}
						error={shown.fields.street}
					/>
					<TextField
						id={id('complement')}
						label="Complément"
						className="sm:col-span-2"
						value={draft.address.complement}
						onChange={(complement) => changeAddress({ complement })}
						placeholder="Bâtiment, zone d’activité…"
					/>
					<TextField
						id={id('postalCode')}
						label="Code postal"
						value={draft.address.postalCode}
						onChange={(postalCode) => changeAddress({ postalCode })}
						inputMode="numeric"
					/>
					<TextField
						id={id('city')}
						label="Ville"
						required
						value={draft.address.city}
						onChange={(city) => changeAddress({ city })}
						error={shown.fields.city}
					/>
					<TextField
						id={id('country')}
						label="Pays"
						value={draft.address.country}
						onChange={(country) => changeAddress({ country })}
					/>
				</fieldset>

				<fieldset className="grid gap-4 sm:grid-cols-2">
					<legend className="mb-3 font-medium text-sm">Coordonnées</legend>
					<TextField
						id={id('email')}
						label="E-mail"
						value={draft.email}
						onChange={(email) => change({ email })}
					/>
					<TextField
						id={id('phone')}
						label="Téléphone"
						value={draft.phone}
						onChange={(phone) => change({ phone })}
						inputMode="tel"
					/>
				</fieldset>

				<fieldset className="grid gap-4 sm:grid-cols-2">
					<legend className="mb-3 font-medium text-sm">Banque</legend>
					<TextField
						id={id('iban')}
						label="IBAN"
						value={draft.iban}
						onChange={(iban) => change({ iban })}
					/>
					<TextField
						id={id('bic')}
						label="BIC"
						value={draft.bic}
						onChange={(bic) => change({ bic })}
					/>
				</fieldset>

				<TextAreaField
					id={id('insurance')}
					label="Assurance professionnelle"
					value={draft.insurance}
					onChange={(insurance) => change({ insurance })}
					hint="Mention de l’assurance décennale ou de responsabilité civile."
				/>

				{shown.other.length > 0 && (
					<ul className="grid gap-1 text-[13px] text-state-alert">
						{shown.other.map((message) => (
							<li key={message}>{message}</li>
						))}
					</ul>
				)}
			</SettingsForm>
		</SettingsCard>
	)
}

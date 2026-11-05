import { type FormEvent, useId, useState } from 'react'
import { FormField } from '#/components/form-field'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	type BillingField,
	type BillingForm,
	type BillingPatch,
	billingFormOf,
	evaluateBilling,
	formatNumber,
} from '#/pages/crm-settings/lib/settings-model'
import {
	formatVatRate,
	type Settings,
	VAT_RATES,
	type VatRate,
} from '#/pages/crm-settings/types'
import {
	SettingsCard,
	SettingsForm,
} from '#/pages/crm-settings/ui/settings-card'
import {
	TextAreaField,
	TextField,
} from '#/pages/crm-settings/ui/settings-fields'
import { useSyncedDraft } from '#/pages/crm-settings/ui/use-synced-draft'

const FIELD_ORDER: readonly BillingField[] = [
	'paymentTermsDays',
	'quoteValidityDays',
	'recoveryFee',
	'latePenalty',
	'numberPadding',
]

export function BillingSection({
	settings,
	year,
	onSave,
}: {
	settings: Settings
	year: number
	onSave: (patch: BillingPatch) => Promise<void>
}) {
	const baseId = useId()
	const id = (field: string) => `${baseId}-${field}`
	const [draft, setDraft, dirty] = useSyncedDraft(billingFormOf(settings))
	const [showErrors, setShowErrors] = useState(false)
	const [saved, setSaved] = useState(false)
	const evaluation = evaluateBilling(draft)
	const errors = showErrors ? evaluation.errors : {}
	const padding = Number(draft.numberPadding)
	const example =
		Number.isInteger(padding) && padding >= 1 && padding <= 6
			? formatNumber('invoice', year, 7, padding)
			: null

	function change(patch: Partial<BillingForm>) {
		setDraft((current) => ({ ...current, ...patch }))
		setSaved(false)
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		setShowErrors(true)
		const { patch } = evaluation
		if (!patch) {
			const first = FIELD_ORDER.find((field) => evaluation.errors[field])
			if (first) document.getElementById(id(first))?.focus()
			return
		}
		void onSave(patch).then(() => {
			setShowErrors(false)
			setSaved(true)
		})
	}

	return (
		<SettingsCard
			id="facturation"
			title="Facturation"
			description="Valeurs proposées pour les nouveaux devis et factures, et mentions légales reprises sur chaque facture."
		>
			<SettingsForm onSubmit={submit} dirty={dirty} saved={saved}>
				<div className="grid gap-4 sm:grid-cols-2">
					<TextField
						id={id('paymentTermsDays')}
						label="Délai de paiement par défaut"
						required
						value={draft.paymentTermsDays}
						onChange={(paymentTermsDays) => change({ paymentTermsDays })}
						error={errors.paymentTermsDays}
						hint="Entre 0 et 60 jours. Un client peut avoir son propre délai."
						inputMode="numeric"
						suffix="jours"
					/>
					<TextField
						id={id('quoteValidityDays')}
						label="Validité des devis"
						required
						value={draft.quoteValidityDays}
						onChange={(quoteValidityDays) => change({ quoteValidityDays })}
						error={errors.quoteValidityDays}
						hint="Durée pendant laquelle un devis émis reste acceptable."
						inputMode="numeric"
						suffix="jours"
					/>
					<FormField
						id={id('defaultVatRate')}
						label="TVA par défaut"
						hint="Taux proposé sur les lignes libres."
					>
						<Select
							value={String(draft.defaultVatRate)}
							onValueChange={(value) =>
								change({ defaultVatRate: Number(value) as VatRate })
							}
						>
							<SelectTrigger id={id('defaultVatRate')} className="w-full">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{VAT_RATES.map((rate) => (
									<SelectItem key={rate} value={String(rate)}>
										{formatVatRate(rate)}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</FormField>
					<TextField
						id={id('recoveryFee')}
						label="Indemnité forfaitaire de recouvrement"
						required
						value={draft.recoveryFee}
						onChange={(recoveryFee) => change({ recoveryFee })}
						error={errors.recoveryFee}
						hint="Saisie en euros, 40 € pour un professionnel."
						inputMode="decimal"
						suffix="€"
					/>
				</div>
				<TextAreaField
					id={id('latePenalty')}
					label="Pénalités de retard"
					required
					rows={2}
					value={draft.latePenalty}
					onChange={(latePenalty) => change({ latePenalty })}
					error={errors.latePenalty}
					hint="Mention légale obligatoire sur chaque facture."
				/>
				<TextAreaField
					id={id('invoiceFooter')}
					label="Pied de page"
					rows={2}
					value={draft.invoiceFooter}
					onChange={(invoiceFooter) => change({ invoiceFooter })}
					hint="Affiché en bas des devis et des factures."
				/>
				<TextField
					id={id('numberPadding')}
					label="Nombre de chiffres du compteur"
					required
					className="sm:max-w-xs"
					value={draft.numberPadding}
					onChange={(numberPadding) => change({ numberPadding })}
					error={errors.numberPadding}
					hint={
						example
							? `Entre 1 et 6. Exemple : ${example}. Les numéros déjà attribués ne changent pas.`
							: 'Entre 1 et 6.'
					}
					inputMode="numeric"
				/>
			</SettingsForm>
		</SettingsCard>
	)
}

import { FormField } from '#/components/form-field'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Textarea } from '#/components/ui/textarea'
import {
	NO_OWNER,
	ownerFromSelect,
	ownerToSelect,
	parsePercent,
} from '#/pages/quotes/lib/quote-draft'
import type { QuoteFormValues } from '#/pages/quotes/types'

export interface QuoteOwner {
	id: string
	name: string
}

interface QuoteTermsFieldsProps {
	idPrefix: string
	values: Pick<
		QuoteFormValues,
		'ownerId' | 'validUntil' | 'depositPercent' | 'notes'
	>
	owners: readonly QuoteOwner[]
	onChange: (patch: Partial<QuoteFormValues>) => void
}

export function QuoteTermsFields({
	idPrefix,
	values,
	owners,
	onChange,
}: QuoteTermsFieldsProps) {
	const depositInvalid =
		values.depositPercent.trim() !== '' &&
		parsePercent(values.depositPercent) === null
	return (
		<>
			<div className="grid gap-4 sm:grid-cols-3">
				<FormField id={`${idPrefix}-owner`} label="Responsable">
					<Select
						value={ownerToSelect(values.ownerId)}
						onValueChange={(value) =>
							onChange({ ownerId: ownerFromSelect(value) })
						}
					>
						<SelectTrigger id={`${idPrefix}-owner`} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value={NO_OWNER}>Sans responsable</SelectItem>
							{owners.map((owner) => (
								<SelectItem key={owner.id} value={owner.id}>
									{owner.name}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</FormField>
				<FormField id={`${idPrefix}-valid-until`} label="Valable jusqu’au">
					<Input
						id={`${idPrefix}-valid-until`}
						type="date"
						value={values.validUntil}
						onChange={(event) => onChange({ validUntil: event.target.value })}
					/>
				</FormField>
				<FormField
					id={`${idPrefix}-deposit`}
					label="Acompte demandé (%)"
					error={
						depositInvalid
							? 'L’acompte doit être compris entre 0 et 100 %.'
							: undefined
					}
				>
					<Input
						id={`${idPrefix}-deposit`}
						inputMode="decimal"
						autoComplete="off"
						placeholder="0"
						value={values.depositPercent}
						aria-invalid={depositInvalid || undefined}
						onChange={(event) =>
							onChange({ depositPercent: event.target.value })
						}
					/>
				</FormField>
			</div>
			<FormField
				id={`${idPrefix}-notes`}
				label="Notes"
				hint="Imprimées en bas du devis."
			>
				<Textarea
					id={`${idPrefix}-notes`}
					rows={3}
					value={values.notes}
					onChange={(event) => onChange({ notes: event.target.value })}
					placeholder="Conditions particulières, délais, modalités de règlement"
				/>
			</FormField>
		</>
	)
}

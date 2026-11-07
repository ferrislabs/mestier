import type { ReactNode } from 'react'
import { FormField } from '#/components/form-field'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { Customer } from '#/hooks/use-customers'
import { PEOPLE } from '#/pages/customers/feature/mocks/people'
import { LEAD_SOURCE_LABELS, LEAD_SOURCES } from '#/pages/customers/types'
import { CustomerPicker } from '#/pages/customers/ui/customer-picker'
import { TextArea } from '#/pages/customers/ui/text-area'
import type {
	DealForm,
	DealFormErrors,
	DealFormField,
} from '#/pages/deals/lib/deal-form'

const NONE = '__none'

function invalidProps(id: string, error: string | undefined) {
	return {
		'aria-invalid': error ? true : undefined,
		'aria-describedby': error ? `${id}-error` : undefined,
	}
}

export function DealFields({
	idPrefix,
	form,
	errors,
	customers,
	clientLocked,
	clientNote,
	onChange,
}: {
	idPrefix: string
	form: DealForm
	errors: DealFormErrors
	customers: readonly Customer[]
	clientLocked?: boolean
	clientNote?: ReactNode
	onChange: (patch: Partial<DealForm>) => void
}) {
	const id = (field: string) => `${idPrefix}-${field}`
	const error = (field: DealFormField) => errors[field]
	return (
		<div className="grid gap-4 sm:grid-cols-2">
			<FormField
				id={id('customerId')}
				label="Client"
				required
				error={error('customerId')}
				className="sm:col-span-2"
			>
				<CustomerPicker
					id={id('customerId')}
					customers={customers}
					value={form.customerId === '' ? null : form.customerId}
					onChange={(customerId) => onChange({ customerId })}
					disabled={clientLocked}
					invalid={error('customerId') !== undefined}
				/>
				{clientNote && (
					<p className="text-[12px] text-muted-foreground">{clientNote}</p>
				)}
			</FormField>
			<FormField
				id={id('title')}
				label="Intitulé"
				required
				error={error('title')}
				className="sm:col-span-2"
			>
				<Input
					id={id('title')}
					value={form.title}
					onChange={(event) => onChange({ title: event.target.value })}
					autoComplete="off"
					placeholder="Remplacement de 4 fenêtres"
					{...invalidProps(id('title'), error('title'))}
				/>
			</FormField>
			<FormField
				id={id('amount')}
				label="Montant HT estimé (€)"
				error={error('amount')}
			>
				<Input
					id={id('amount')}
					value={form.amount}
					onChange={(event) => onChange({ amount: event.target.value })}
					inputMode="decimal"
					autoComplete="off"
					placeholder="0"
					{...invalidProps(id('amount'), error('amount'))}
				/>
			</FormField>
			<FormField
				id={id('expectedCloseDate')}
				label="Clôture prévue"
				error={error('expectedCloseDate')}
			>
				<Input
					id={id('expectedCloseDate')}
					type="date"
					value={form.expectedCloseDate}
					onChange={(event) =>
						onChange({ expectedCloseDate: event.target.value })
					}
					{...invalidProps(id('expectedCloseDate'), error('expectedCloseDate'))}
				/>
			</FormField>
			<FormField id={id('ownerId')} label="Responsable">
				<Select
					value={form.ownerId === '' ? NONE : form.ownerId}
					onValueChange={(value) =>
						onChange({ ownerId: value === NONE ? '' : value })
					}
				>
					<SelectTrigger id={id('ownerId')} className="w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={NONE}>Non assigné</SelectItem>
						{PEOPLE.map((person) => (
							<SelectItem key={person.id} value={person.id}>
								{person.name}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</FormField>
			<FormField id={id('source')} label="Source">
				<Select
					value={form.source === '' ? NONE : form.source}
					onValueChange={(value) =>
						onChange({ source: value === NONE ? '' : value })
					}
				>
					<SelectTrigger id={id('source')} className="w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={NONE}>Aucune</SelectItem>
						{LEAD_SOURCES.map((source) => (
							<SelectItem key={source} value={source}>
								{LEAD_SOURCE_LABELS[source]}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</FormField>
			<FormField id={id('notes')} label="Notes" className="sm:col-span-2">
				<TextArea
					id={id('notes')}
					rows={3}
					value={form.notes}
					onChange={(event) => onChange({ notes: event.target.value })}
					className="resize-y"
				/>
			</FormField>
		</div>
	)
}

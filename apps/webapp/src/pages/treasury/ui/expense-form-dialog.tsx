import { type FormEvent, useId, useState } from 'react'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { Customer } from '#/hooks/use-customers'
import {
	formatVatRate,
	VAT_RATES,
	type VatRate,
} from '#/pages/crm-settings/types'
import { PEOPLE } from '#/pages/customers/feature/mocks/people'
import { CustomerPicker } from '#/pages/customers/ui/customer-picker'
import { Money } from '#/pages/customers/ui/money'
import { TextArea } from '#/pages/customers/ui/text-area'
import {
	applyVatRate,
	changeAmountHt,
	changeVatAmount,
	type ExpenseFormValues,
	expenseFormErrors,
	expenseTotalOf,
} from '#/pages/treasury/lib/ledger-model'
import type { Expense, ExpenseCategory } from '#/pages/treasury/types'
import { CategoryChip } from '#/pages/treasury/ui/category-chip'

const NO_SUPPLIER = '__none'
const COMPANY = '__company'

export interface SupplierOption {
	id: string
	name: string
}

interface ExpenseFormDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	expense?: Expense
	initial: ExpenseFormValues
	categories: readonly ExpenseCategory[]
	suppliers: readonly SupplierOption[]
	customers: readonly Customer[]
	onSubmit: (values: ExpenseFormValues) => Promise<void>
}

export function ExpenseFormDialog({
	open,
	onOpenChange,
	...form
}: ExpenseFormDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-2xl">
				<ExpenseForm onClose={() => onOpenChange(false)} {...form} />
			</DialogContent>
		</Dialog>
	)
}

function ExpenseForm({
	expense,
	initial,
	categories,
	suppliers,
	customers,
	onClose,
	onSubmit,
}: Omit<ExpenseFormDialogProps, 'open' | 'onOpenChange'> & {
	onClose: () => void
}) {
	const baseId = useId()
	const [values, setValues] = useState<ExpenseFormValues>(initial)
	const [attempted, setAttempted] = useState(false)
	const [fileKey, setFileKey] = useState(0)
	const id = (field: string) => `${baseId}-${field}`
	const errors = expenseFormErrors(values)
	const total = expenseTotalOf(values)
	const advanced = values.advancedById !== null

	function change(patch: Partial<ExpenseFormValues>) {
		setValues((current) => ({ ...current, ...patch }))
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		if (errors.length > 0) {
			setAttempted(true)
			return
		}
		void onSubmit(values).then(onClose)
	}

	return (
		<form onSubmit={submit} noValidate className="flex min-h-0 flex-1 flex-col">
			<DialogHeader className="px-6 pt-6 pb-4">
				<DialogTitle>
					{expense ? 'Modifier la dépense' : 'Saisir une dépense'}
				</DialogTitle>
				<DialogDescription>
					{expense
						? 'Corrige les informations de la dépense. Son statut de paiement ne change pas.'
						: 'Renseigne l’achat ou le frais. Une dépense avancée par un collaborateur est considérée comme réglée par lui, à lui rembourser.'}
				</DialogDescription>
			</DialogHeader>
			<div className="grid min-h-0 flex-1 gap-4 overflow-y-auto border-t px-6 py-5 sm:grid-cols-2">
				<FormField id={id('date')} label="Date" required>
					<Input
						id={id('date')}
						type="date"
						value={values.date}
						onChange={(event) => change({ date: event.target.value })}
					/>
				</FormField>
				<FormField id={id('label')} label="Libellé" required>
					<Input
						id={id('label')}
						value={values.label}
						onChange={(event) => change({ label: event.target.value })}
						autoComplete="off"
						placeholder="Quincaillerie, carburant, assurance…"
					/>
				</FormField>
				<FormField id={id('supplier')} label="Fournisseur">
					<Select
						value={values.supplierId ?? NO_SUPPLIER}
						onValueChange={(value) =>
							change({ supplierId: value === NO_SUPPLIER ? null : value })
						}
					>
						<SelectTrigger id={id('supplier')} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value={NO_SUPPLIER}>
								Sans fournisseur (ticket, achat comptant)
							</SelectItem>
							{suppliers.map((supplier) => (
								<SelectItem key={supplier.id} value={supplier.id}>
									{supplier.name}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</FormField>
				<FormField id={id('category')} label="Catégorie" required>
					<Select
						value={values.categoryId}
						onValueChange={(value) =>
							change({ categoryId: value, categoryTouched: true })
						}
					>
						<SelectTrigger id={id('category')} className="w-full">
							<SelectValue placeholder="Choisir une catégorie" />
						</SelectTrigger>
						<SelectContent>
							{categories.map((category) => (
								<SelectItem key={category.id} value={category.id}>
									<CategoryChip category={category} />
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</FormField>
				<FormField
					id={id('reference')}
					label="N° de facture fournisseur"
					className="sm:col-span-2"
				>
					<Input
						id={id('reference')}
						value={values.reference}
						onChange={(event) => change({ reference: event.target.value })}
						autoComplete="off"
					/>
				</FormField>
				<FormField id={id('amountHt')} label="Montant HT (€)" required>
					<Input
						id={id('amountHt')}
						inputMode="decimal"
						value={values.amountHt}
						onChange={(event) =>
							setValues((current) =>
								changeAmountHt(current, event.target.value),
							)
						}
						autoComplete="off"
						placeholder="0,00"
					/>
				</FormField>
				<FormField id={id('vat')} label="TVA (€)">
					<Input
						id={id('vat')}
						inputMode="decimal"
						value={values.vatAmount}
						onChange={(event) =>
							setValues((current) =>
								changeVatAmount(current, event.target.value),
							)
						}
						autoComplete="off"
						placeholder="0,00"
					/>
					<fieldset className="flex min-w-0 flex-wrap items-center gap-1.5">
						<legend className="sr-only">
							Taux de TVA appliqué au montant HT
						</legend>
						{VAT_RATES.map((rate: VatRate) => (
							<Button
								key={rate}
								type="button"
								size="xs"
								variant={values.vatRate === rate ? 'default' : 'outline'}
								aria-pressed={values.vatRate === rate}
								onClick={() =>
									setValues((current) => applyVatRate(current, rate))
								}
							>
								{formatVatRate(rate)}
							</Button>
						))}
						<span className="ml-auto text-[12px] text-muted-foreground">
							Total TTC{' '}
							<span className="font-medium text-foreground">
								{total === null ? '—' : <Money cents={total} />}
							</span>
						</span>
					</fieldset>
				</FormField>
				<FormField
					id={id('advanced')}
					label="Avancée par"
					hint={
						expense
							? 'Non modifiable après la saisie.'
							: 'Choisis un collaborateur s’il a payé de sa poche.'
					}
				>
					<Select
						value={values.advancedById ?? COMPANY}
						onValueChange={(value) =>
							change({
								advancedById: value === COMPANY ? null : value,
								dueDate: value === COMPANY ? values.dueDate : '',
							})
						}
						disabled={expense !== undefined}
					>
						<SelectTrigger id={id('advanced')} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value={COMPANY}>L’entreprise</SelectItem>
							{PEOPLE.map((person) => (
								<SelectItem key={person.id} value={person.id}>
									{person.name}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</FormField>
				<FormField
					id={id('due')}
					label="Échéance"
					hint={
						advanced ? 'Sans objet : la dépense est déjà réglée.' : undefined
					}
				>
					<Input
						id={id('due')}
						type="date"
						value={advanced ? '' : values.dueDate}
						onChange={(event) => change({ dueDate: event.target.value })}
						disabled={advanced}
					/>
				</FormField>
				<FormField
					id={id('account')}
					label="Client concerné"
					className="sm:col-span-2"
					hint="Pour rattacher cette dépense à un chantier ou à un client."
				>
					<div className="flex items-center gap-2">
						<CustomerPicker
							id={id('account')}
							customers={customers}
							value={values.customerId}
							onChange={(customerId) => change({ customerId })}
							placeholder="Aucun client"
						/>
						{values.customerId !== null && (
							<Button
								type="button"
								variant="ghost"
								size="sm"
								onClick={() => change({ customerId: null })}
							>
								Retirer
							</Button>
						)}
					</div>
				</FormField>
				<FormField
					id={id('proof')}
					label="Justificatif"
					className="sm:col-span-2"
					hint="Seul le nom du fichier est conservé pour le moment."
				>
					<Input
						key={fileKey}
						id={id('proof')}
						type="file"
						accept=".pdf,image/*"
						onChange={(event) =>
							change({
								proofName:
									event.target.files?.item(0)?.name ?? values.proofName,
							})
						}
					/>
					{values.proofName !== null && (
						<p className="flex items-center gap-2 text-[13px]">
							<span className="text-muted-foreground">
								Justificatif retenu :
							</span>
							<span className="min-w-0 truncate font-medium">
								{values.proofName}
							</span>
							<Button
								type="button"
								variant="ghost"
								size="xs"
								onClick={() => {
									change({ proofName: null })
									setFileKey((current) => current + 1)
								}}
							>
								Retirer
							</Button>
						</p>
					)}
				</FormField>
				<FormField id={id('note')} label="Note" className="sm:col-span-2">
					<TextArea
						id={id('note')}
						rows={2}
						value={values.note}
						onChange={(event) => change({ note: event.target.value })}
					/>
				</FormField>
			</div>
			<div className="grid gap-3 border-t px-6 py-4">
				{attempted && errors.length > 0 && (
					<ul
						role="alert"
						className="list-disc rounded-md bg-state-alert/8 py-2 pr-3 pl-7 text-[13px] text-state-alert"
					>
						{errors.map((message) => (
							<li key={message}>{message}</li>
						))}
					</ul>
				)}
				<DialogFooter>
					<Button type="button" variant="outline" onClick={onClose}>
						Annuler
					</Button>
					<Button type="submit">
						{expense ? 'Enregistrer' : 'Saisir la dépense'}
					</Button>
				</DialogFooter>
			</div>
		</form>
	)
}

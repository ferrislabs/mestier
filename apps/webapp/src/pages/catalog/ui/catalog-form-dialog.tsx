import { type FormEvent, type ReactNode, useId, useState } from 'react'
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
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import {
	type BillingFrequency,
	CATALOG_KIND_LABELS,
	type CatalogField,
	type CatalogForm,
	type CatalogKind,
	type CatalogRow,
	evaluateForm,
	FREQUENCIES,
	FREQUENCY_LABELS,
	formatPercent,
	IMMUTABLE_NOTE,
	marginPreview,
} from '#/pages/catalog/catalog-model'
import {
	formatVatRate,
	VAT_RATES,
	type VatRate,
} from '#/pages/crm-settings/types'
import { Money } from '#/pages/customers/ui/money'
import { UnitSelect } from '#/pages/quotes/ui/unit-select'

export type CatalogEditor =
	| { mode: 'create' }
	| { mode: 'edit'; row: CatalogRow }
	| { mode: 'duplicate'; row: CatalogRow }

const FIELD_ORDER: readonly CatalogField[] = ['name', 'unit', 'price', 'cost']

const NO_VAT = '__none'

function titleOf(editor: CatalogEditor): string {
	if (editor.mode === 'edit') return `Modifier ${editor.row.name}`
	if (editor.mode === 'duplicate') return `Dupliquer ${editor.row.name}`
	return 'Nouvel article'
}

function descriptionOf(editor: CatalogEditor): string {
	if (editor.mode === 'edit')
		return 'Mets à jour la désignation, les prix ou la TVA proposés pour les prochaines lignes.'
	if (editor.mode === 'duplicate')
		return 'Une copie est créée avec les mêmes valeurs : renseigne une nouvelle référence et ajuste ce qui change.'
	return 'Un service ou un produit que tu pourras ajouter en un clic dans tes devis et factures.'
}

function keyOf(editor: CatalogEditor): string {
	return editor.mode === 'create' ? 'create' : `${editor.mode}:${editor.row.id}`
}

interface CatalogFormDialogProps {
	editor: CatalogEditor
	open: boolean
	initial: CatalogForm
	categories: readonly string[]
	customUnits: string[]
	isPending: boolean
	error: string | null
	sameReference: (reference: string) => boolean
	onClose: () => void
	onCreateCustomUnit: (code: string) => void
	isCreatingCustomUnit: boolean
	onSubmit: (form: CatalogForm) => Promise<void>
	productExtras: (
		form: CatalogForm,
		change: (patch: Partial<CatalogForm>) => void,
	) => ReactNode
}

export function CatalogFormDialog({
	open,
	onClose,
	...form
}: CatalogFormDialogProps) {
	return (
		<Dialog
			open={open}
			onOpenChange={(next) => {
				if (!next) onClose()
			}}
		>
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-xl">
				<CatalogItemForm key={keyOf(form.editor)} onClose={onClose} {...form} />
			</DialogContent>
		</Dialog>
	)
}

function AmountInput({
	id,
	value,
	error,
	onChange,
	onBlur,
}: {
	id: string
	value: string
	error?: string
	onChange: (value: string) => void
	onBlur: () => void
}) {
	return (
		<div className="relative">
			<Input
				id={id}
				inputMode="decimal"
				autoComplete="off"
				placeholder="0,00"
				value={value}
				onChange={(event) => onChange(event.target.value)}
				onBlur={onBlur}
				aria-invalid={error ? true : undefined}
				aria-describedby={error ? `${id}-error` : undefined}
				className="tabular pr-8 text-right"
			/>
			<span
				aria-hidden
				className="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-muted-foreground text-sm"
			>
				€
			</span>
		</div>
	)
}

function CatalogItemForm({
	editor,
	initial,
	categories,
	customUnits,
	isPending,
	error,
	sameReference,
	onClose,
	onCreateCustomUnit,
	isCreatingCustomUnit,
	onSubmit,
	productExtras,
}: Omit<CatalogFormDialogProps, 'open'>) {
	const baseId = useId()
	const id = (field: string) => `${baseId}-${field}`
	const [form, setForm] = useState<CatalogForm>(initial)
	const [touched, setTouched] = useState<ReadonlySet<CatalogField>>(new Set())
	const [submitted, setSubmitted] = useState(false)

	const evaluation = evaluateForm(form)
	const margin = marginPreview(form)
	const duplicateReference = sameReference(form.reference)

	function change(patch: Partial<CatalogForm>) {
		setForm((current) => ({ ...current, ...patch }))
	}

	function touch(field: CatalogField) {
		setTouched((current) => new Set([...current, field]))
	}

	function shown(field: CatalogField): string | undefined {
		return submitted || touched.has(field)
			? evaluation.errors[field]
			: undefined
	}

	function fieldProps(field: CatalogField) {
		const message = shown(field)
		return {
			'aria-invalid': message ? true : undefined,
			'aria-describedby': message ? `${id(field)}-error` : undefined,
			onBlur: () => touch(field),
		}
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		setSubmitted(true)
		if (!evaluation.valid) {
			const first = FIELD_ORDER.find((field) => evaluation.errors[field])
			if (first) document.getElementById(id(first))?.focus()
			return
		}
		void onSubmit(form).catch(() => undefined)
	}

	const isProduct = form.kind === 'product'

	return (
		<form onSubmit={submit} noValidate className="flex min-h-0 flex-1 flex-col">
			<DialogHeader className="px-6 pt-6 pb-4">
				<DialogTitle>{titleOf(editor)}</DialogTitle>
				<DialogDescription>{descriptionOf(editor)}</DialogDescription>
			</DialogHeader>

			<div className="grid min-h-0 flex-1 content-start gap-4 overflow-y-auto border-t px-6 py-5 sm:grid-cols-2">
				<div className="grid gap-1.5 sm:col-span-2">
					<span id={id('kind-label')} className="font-medium text-sm">
						Type
					</span>
					<ToggleGroup
						type="single"
						variant="outline"
						size="sm"
						value={form.kind}
						onValueChange={(value) => {
							if (value && editor.mode !== 'edit')
								change({ kind: value as CatalogKind })
						}}
						aria-labelledby={id('kind-label')}
					>
						<ToggleGroupItem
							value="service"
							disabled={editor.mode === 'edit' && form.kind !== 'service'}
						>
							{CATALOG_KIND_LABELS.service}
						</ToggleGroupItem>
						<ToggleGroupItem
							value="product"
							disabled={editor.mode === 'edit' && form.kind !== 'product'}
						>
							{CATALOG_KIND_LABELS.product}
						</ToggleGroupItem>
					</ToggleGroup>
				</div>

				{isProduct && (
					<FormField
						id={id('reference')}
						label="Référence"
						hint={
							duplicateReference
								? 'Cette référence est déjà utilisée par un autre article.'
								: undefined
						}
					>
						<Input
							id={id('reference')}
							value={form.reference}
							onChange={(event) => change({ reference: event.target.value })}
							autoComplete="off"
							placeholder="FEN-PVC-120"
						/>
					</FormField>
				)}
				<FormField
					id={id('name')}
					label="Désignation"
					required
					error={shown('name')}
					className={isProduct ? undefined : 'sm:col-span-2'}
				>
					<Input
						id={id('name')}
						value={form.name}
						onChange={(event) => change({ name: event.target.value })}
						autoComplete="off"
						{...fieldProps('name')}
					/>
				</FormField>
				<FormField
					id={id('description')}
					label="Description"
					className="sm:col-span-2"
				>
					<Input
						id={id('description')}
						value={form.description}
						onChange={(event) => change({ description: event.target.value })}
						autoComplete="off"
						placeholder="Une ligne, reprise sous la désignation dans les documents"
					/>
				</FormField>

				<FormField id={id('category')} label="Catégorie">
					<Input
						id={id('category')}
						list={id('categories')}
						value={form.category}
						onChange={(event) => change({ category: event.target.value })}
						autoComplete="off"
						placeholder="Fenêtres, Chantier…"
					/>
					<datalist id={id('categories')}>
						{categories.map((category) => (
							<option key={category} value={category} />
						))}
					</datalist>
				</FormField>
				<FormField id={id('unit')} label="Unité" required error={shown('unit')}>
					<UnitSelect
						value={form.unit}
						onChange={(unit) => {
							change({ unit })
							touch('unit')
						}}
						customUnits={customUnits}
						onCreateCustomUnit={onCreateCustomUnit}
						isCreatingCustomUnit={isCreatingCustomUnit}
					/>
				</FormField>

				<FormField
					id={id('frequency')}
					label="Fréquence de facturation"
					hint="Ponctuel pour une vente unique, sinon le tarif se répète à chaque période."
				>
					<Select
						value={form.frequency}
						onValueChange={(value) =>
							change({ frequency: value as BillingFrequency })
						}
					>
						<SelectTrigger id={id('frequency')} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{FREQUENCIES.map((frequency) => (
								<SelectItem key={frequency} value={frequency}>
									{FREQUENCY_LABELS[frequency]}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</FormField>

				<FormField
					id={id('price')}
					label="Prix HT"
					required
					error={shown('price')}
				>
					<AmountInput
						id={id('price')}
						value={form.price}
						error={shown('price')}
						onChange={(price) => change({ price })}
						onBlur={() => touch('price')}
					/>
				</FormField>
				<FormField id={id('vat')} label="TVA">
					<Select
						value={form.vatRate === null ? NO_VAT : String(form.vatRate)}
						onValueChange={(value) =>
							change({
								vatRate: value === NO_VAT ? null : (Number(value) as VatRate),
							})
						}
					>
						<SelectTrigger id={id('vat')} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value={NO_VAT}>Non définie</SelectItem>
							{VAT_RATES.map((rate) => (
								<SelectItem key={rate} value={String(rate)}>
									{formatVatRate(rate)}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</FormField>

				<FormField
					id={id('cost')}
					label="Coût"
					error={shown('cost')}
					hint="Facultatif : sert à calculer la marge."
				>
					<AmountInput
						id={id('cost')}
						value={form.cost}
						error={shown('cost')}
						onChange={(cost) => change({ cost })}
						onBlur={() => touch('cost')}
					/>
				</FormField>
				<div className="grid content-start gap-1.5">
					<span className="font-medium text-sm">Marge</span>
					<output className="flex h-9 items-center text-sm">
						{margin ? (
							<span className="tabular">
								<Money cents={margin.amount} /> ({formatPercent(margin.percent)}
								)
							</span>
						) : (
							<span className="text-muted-foreground">
								Renseigne le prix et le coût
							</span>
						)}
					</output>
				</div>

				{isProduct && (
					<div className="grid gap-4 sm:col-span-2">
						{productExtras(form, change)}
					</div>
				)}

				{error && (
					<ul className="grid gap-1 text-[13px] text-state-alert sm:col-span-2">
						<li>{error}</li>
					</ul>
				)}
				<p className="rounded-md bg-muted/60 px-3 py-2 text-[13px] text-muted-foreground sm:col-span-2">
					{IMMUTABLE_NOTE}
				</p>
			</div>

			<DialogFooter className="border-t px-6 py-3">
				<Button type="button" variant="outline" onClick={onClose}>
					Annuler
				</Button>
				<Button type="submit" disabled={isPending}>
					{editor.mode === 'edit' ? 'Enregistrer' : 'Créer l’article'}
				</Button>
			</DialogFooter>
		</form>
	)
}

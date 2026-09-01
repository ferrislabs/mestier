import { ChevronRight, Trash2 } from 'lucide-react'
import { useState } from 'react'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from '#/components/ui/alert-dialog'
import { Button } from '#/components/ui/button'
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from '#/components/ui/collapsible'
import { Field } from '#/components/ui/field'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectGroup,
	SelectItem,
	SelectLabel,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Textarea } from '#/components/ui/textarea'
import type { CatalogItem } from '#/hooks/use-catalog-items'
import type { FilePreview } from '#/hooks/use-file-url'
import { cn } from '#/lib/utils'
import {
	COMMON_VAT_RATES_BP,
	eurosToCents,
	formatCents,
	formatUnit,
	formatVatRateBp,
	type QuoteLineFormValues,
	quoteLineSourceLabel,
	quoteLineSummary,
	quoteLineTotalCents,
} from '../types'
import { PhotoStrip } from './photo-strip'
import { UnitSelect } from './unit-select'

interface QuoteLineEditorProps {
	index: number
	line: QuoteLineFormValues
	catalogItems: CatalogItem[]
	photos: FilePreview[]
	isOpen: boolean
	canRemove: boolean
	isUploading?: boolean
	/** The organization's VAT status, read once at the page level
	 * (`activeOrganization.vat_status`). An organization not subject to VAT
	 * gets no rate column at all — not a column of zeros. */
	vatEnabled: boolean
	/** Shared with the header row in `quote-lines-table.tsx` so the columns
	 * actually line up — a `<table>` would do this for free, but `Collapsible`
	 * needs a block-level wrapper it cannot be given inside a `<tbody>`, so
	 * the "table" is a CSS grid instead, and its column template has to be
	 * threaded through by hand. */
	gridTemplateColumns: string
	onOpenChange: (open: boolean) => void
	onChange: (patch: Partial<QuoteLineFormValues>) => void
	onSelectCatalogItem: (catalogItemId: string) => void
	onRemove: () => void
	onUploadPhoto: (file: File) => void
	onRemovePhoto: (key: string) => void
}

/**
 * One quote line, folded to a summary until it is being edited.
 *
 * A quote of six lines used to render six copies of a nine-field grid, and
 * finding the line you meant to change cost more than changing it. Folded, the
 * list reads like the quote itself; one line is open at a time and its detail
 * is the only detail on screen.
 */
export function QuoteLineEditor({
	index,
	line,
	catalogItems,
	photos,
	isOpen,
	canRemove,
	isUploading,
	vatEnabled,
	gridTemplateColumns,
	onOpenChange,
	onChange,
	onSelectCatalogItem,
	onRemove,
	onUploadPhoto,
	onRemovePhoto,
}: QuoteLineEditorProps) {
	const lineTotalCents = quoteLineTotalCents(line)
	const serviceItems = catalogItems.filter((item) => item.type === 'SERVICE')
	const productItems = catalogItems.filter((item) => item.type === 'PRODUCT')
	const selectedCatalogItem = catalogItems.find(
		(item) => item.id === line.catalogItemId,
	)
	const vatLabel =
		line.vatRateBp !== '' ? formatVatRateBp(Number(line.vatRateBp)) : '—'

	// A catalogue pick overwrites label, unit and price outright — correct
	// when there is nothing to lose, destructive when it silently replaces a
	// free line someone already wrote out by hand. Only that second case
	// pauses to confirm.
	const [pendingCatalogItemId, setPendingCatalogItemId] = useState<
		string | null
	>(null)
	const hasHandwrittenContent =
		line.catalogItemType === 'CUSTOM' && line.label.trim() !== ''

	const handleCatalogValueChange = (value: string) => {
		if (value === 'custom') {
			onSelectCatalogItem('')
			return
		}
		if (hasHandwrittenContent) {
			setPendingCatalogItemId(value)
			return
		}
		onSelectCatalogItem(value)
	}

	return (
		<>
			<Collapsible
				open={isOpen}
				onOpenChange={onOpenChange}
				className="border-b bg-card transition-colors last:border-b-0 hover:bg-muted/40 data-[state=open]:bg-brand-soft data-[state=open]:hover:bg-brand-soft"
			>
				<div
					className="grid items-center gap-3 py-2.5 pr-2 pl-3"
					style={{ gridTemplateColumns }}
				>
					<CollapsibleTrigger className="flex min-w-0 items-center gap-2.5 text-left">
						<ChevronRight
							className={cn(
								'size-3.5 shrink-0 text-muted-foreground transition-transform',
								isOpen && 'rotate-90',
							)}
						/>
						<span className="min-w-0">
							<span className="block truncate text-sm font-medium">
								{line.label.trim() || `Ligne ${index + 1}`}
							</span>
							<span className="block truncate text-xs text-muted-foreground">
								{isOpen
									? quoteLineSourceLabel(line.catalogItemType)
									: quoteLineSummary(line)}
							</span>
						</span>
					</CollapsibleTrigger>

					<span className="justify-self-end text-sm tabular-nums">
						{line.quantity.trim() || '0'} {formatUnit(line.unit)}
					</span>
					<span className="justify-self-end text-sm tabular-nums">
						{formatCents(eurosToCents(line.unitPrice))}
					</span>
					{vatEnabled ? (
						<span className="justify-self-end text-sm tabular-nums text-muted-foreground">
							{vatLabel}
						</span>
					) : null}
					<span className="justify-self-end text-sm font-semibold tabular-nums">
						{formatCents(lineTotalCents)}
					</span>
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						disabled={!canRemove}
						onClick={onRemove}
					>
						<Trash2 />
						<span className="sr-only">Supprimer la ligne</span>
					</Button>
				</div>

				<CollapsibleContent>
					<div className="space-y-5 border-t px-4 py-4">
						{/* What is being sold. */}
						<div className="grid gap-4 md:grid-cols-2">
							<Field
								label="Catalogue"
								htmlFor={`catalog-${line.clientId}`}
								size="compact"
							>
								<Select
									value={line.catalogItemId || 'custom'}
									onValueChange={handleCatalogValueChange}
								>
									<SelectTrigger
										id={`catalog-${line.clientId}`}
										className="w-full"
									>
										<SelectValue placeholder="Ligne libre" />
									</SelectTrigger>
									<SelectContent>
										<SelectItem value="custom">Ligne libre</SelectItem>
										{serviceItems.length > 0 ? (
											<SelectGroup>
												<SelectLabel>Services</SelectLabel>
												{serviceItems.map((item) => (
													<SelectItem key={item.id} value={item.id}>
														{item.label}
													</SelectItem>
												))}
											</SelectGroup>
										) : null}
										{productItems.length > 0 ? (
											<SelectGroup>
												<SelectLabel>Produits</SelectLabel>
												{productItems.map((item) => (
													<SelectItem key={item.id} value={item.id}>
														{item.label}
													</SelectItem>
												))}
											</SelectGroup>
										) : null}
									</SelectContent>
								</Select>
								{selectedCatalogItem ? (
									<p className="truncate text-xs text-muted-foreground">
										{formatCents(selectedCatalogItem.unitPriceCents)} /{' '}
										{formatUnit(selectedCatalogItem.unit)}
									</p>
								) : null}
							</Field>

							<Field
								label="Prestation"
								htmlFor={`label-${line.clientId}`}
								size="compact"
							>
								<Input
									id={`label-${line.clientId}`}
									value={line.label}
									onChange={(event) => onChange({ label: event.target.value })}
									placeholder="Ce qui sera facturé"
								/>
							</Field>
						</div>

						{/* What it costs. Kept on one row so the arithmetic reads left to
					    right and the total sits where the eye lands last. */}
						<div
							className={cn(
								'grid items-end gap-4',
								vatEnabled
									? 'sm:grid-cols-[1fr_1fr_1fr_1fr_auto]'
									: 'sm:grid-cols-[1fr_1fr_1fr_auto]',
							)}
						>
							<Field
								label="Quantité"
								htmlFor={`quantity-${line.clientId}`}
								size="compact"
							>
								<Input
									id={`quantity-${line.clientId}`}
									inputMode="decimal"
									value={line.quantity}
									onChange={(event) =>
										onChange({ quantity: event.target.value })
									}
								/>
							</Field>
							<Field label="Unité" htmlFor={null} size="compact">
								<UnitSelect
									value={line.unit}
									onChange={(unit) => onChange({ unit })}
								/>
							</Field>
							<Field
								label="Prix unitaire"
								htmlFor={`price-${line.clientId}`}
								size="compact"
							>
								<Input
									id={`price-${line.clientId}`}
									inputMode="decimal"
									value={line.unitPrice}
									onChange={(event) =>
										onChange({ unitPrice: event.target.value })
									}
									placeholder="0,00"
								/>
							</Field>
							{vatEnabled ? (
								<Field
									label="TVA"
									htmlFor={`vat-${line.clientId}`}
									size="compact"
								>
									<VatRateField
										id={`vat-${line.clientId}`}
										valueBp={line.vatRateBp}
										onChange={(vatRateBp) => onChange({ vatRateBp })}
									/>
								</Field>
							) : null}
							<div className="rounded-lg bg-muted px-4 py-2 text-right sm:min-w-32">
								<p className="text-xs text-muted-foreground">Total ligne</p>
								<p className="font-semibold tabular-nums">
									{formatCents(lineTotalCents)}
								</p>
							</div>
						</div>

						{/* Everything the price does not say. */}
						<div className="grid gap-4 lg:grid-cols-2">
							<Field
								label="Note"
								htmlFor={`notes-${line.clientId}`}
								size="compact"
							>
								<Textarea
									id={`notes-${line.clientId}`}
									rows={5}
									className="min-h-28 resize-y"
									value={line.notes}
									onChange={(event) => onChange({ notes: event.target.value })}
									placeholder="Accès, contraintes de projet, matériaux imposés, tout ce qui explique le prix"
								/>
							</Field>
							<Field label="Photos" htmlFor={null} size="compact">
								<PhotoStrip
									photos={photos}
									isUploading={isUploading}
									onAdd={onUploadPhoto}
									onRemove={onRemovePhoto}
								/>
							</Field>
						</div>
					</div>
				</CollapsibleContent>
			</Collapsible>

			<AlertDialog
				open={pendingCatalogItemId !== null}
				onOpenChange={(open) => {
					if (!open) setPendingCatalogItemId(null)
				}}
			>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>Remplacer cette ligne libre ?</AlertDialogTitle>
						<AlertDialogDescription>
							« {line.label} » sera remplacé par l'article du catalogue et son
							prix. Le texte saisi sur cette ligne sera perdu.
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel>Annuler</AlertDialogCancel>
						<AlertDialogAction
							onClick={() => {
								if (pendingCatalogItemId) {
									onSelectCatalogItem(pendingCatalogItemId)
								}
							}}
						>
							Remplacer
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>
		</>
	)
}

const CUSTOM_VAT_RATE = 'custom'

/**
 * The organization's usual rates as one-click choices, with a free entry for
 * the rest — never a blank that could mean either "no VAT" and "not decided
 * yet" (see #310's `VatStatus`).
 */
function VatRateField({
	id,
	valueBp,
	onChange,
}: {
	id: string
	valueBp: string
	onChange: (valueBp: string) => void
}) {
	const isCommonRate =
		valueBp !== '' &&
		(COMMON_VAT_RATES_BP as readonly number[]).includes(Number(valueBp))
	const selectValue =
		valueBp === '' ? '' : isCommonRate ? valueBp : CUSTOM_VAT_RATE

	return (
		<div className="space-y-1.5">
			<Select
				value={selectValue}
				onValueChange={(next) => {
					if (next === CUSTOM_VAT_RATE) {
						onChange(valueBp || '0')
						return
					}
					onChange(next)
				}}
			>
				<SelectTrigger id={id} className="w-full">
					<SelectValue placeholder="Taux" />
				</SelectTrigger>
				<SelectContent>
					{COMMON_VAT_RATES_BP.map((rateBp) => (
						<SelectItem key={rateBp} value={String(rateBp)}>
							{formatVatRateBp(rateBp)}
						</SelectItem>
					))}
					<SelectItem value={CUSTOM_VAT_RATE}>Autre…</SelectItem>
				</SelectContent>
			</Select>
			{selectValue === CUSTOM_VAT_RATE ? (
				<Input
					inputMode="decimal"
					value={valueBp === '' ? '' : String(Number(valueBp) / 100)}
					onChange={(event) => {
						const percent = Number(event.target.value.replace(',', '.'))
						onChange(
							Number.isFinite(percent)
								? String(Math.round(percent * 100))
								: '0',
						)
					}}
					placeholder="Taux en %"
				/>
			) : null}
		</div>
	)
}

import { ChevronRight, Trash2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from '#/components/ui/collapsible'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
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
	formatCents,
	formatUnit,
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

	return (
		<Collapsible
			open={isOpen}
			onOpenChange={onOpenChange}
			className="bg-card data-[state=open]:bg-muted/20"
		>
			<div className="flex items-center gap-2 px-4 py-3">
				<CollapsibleTrigger className="flex min-w-0 flex-1 items-center gap-3 text-left">
					<ChevronRight
						className={cn(
							'size-4 shrink-0 text-muted-foreground transition-transform',
							isOpen && 'rotate-90',
						)}
					/>
					<span className="flex size-7 shrink-0 items-center justify-center rounded-md bg-muted text-xs font-semibold text-muted-foreground">
						{index + 1}
					</span>
					<span className="min-w-0 flex-1">
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

				<span className="shrink-0 text-sm font-semibold tabular-nums">
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
						<Field label="Catalogue" htmlFor={`catalog-${line.clientId}`}>
							<Select
								value={line.catalogItemId || 'custom'}
								onValueChange={(value) =>
									onSelectCatalogItem(value === 'custom' ? '' : value)
								}
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

						<Field label="Prestation" htmlFor={`label-${line.clientId}`}>
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
					<div className="grid items-end gap-4 sm:grid-cols-[1fr_1fr_1fr_auto]">
						<Field label="Quantité" htmlFor={`quantity-${line.clientId}`}>
							<Input
								id={`quantity-${line.clientId}`}
								inputMode="decimal"
								value={line.quantity}
								onChange={(event) => onChange({ quantity: event.target.value })}
							/>
						</Field>
						<Field label="Unité">
							<UnitSelect
								value={line.unit}
								onChange={(unit) => onChange({ unit })}
							/>
						</Field>
						<Field label="Prix unitaire" htmlFor={`price-${line.clientId}`}>
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
						<div className="rounded-lg bg-muted px-4 py-2 text-right sm:min-w-32">
							<p className="text-xs text-muted-foreground">Total ligne</p>
							<p className="font-semibold tabular-nums">
								{formatCents(lineTotalCents)}
							</p>
						</div>
					</div>

					{/* Everything the price does not say. */}
					<div className="grid gap-4 lg:grid-cols-2">
						<Field label="Note" htmlFor={`notes-${line.clientId}`}>
							<Textarea
								id={`notes-${line.clientId}`}
								rows={5}
								className="min-h-28 resize-y"
								value={line.notes}
								onChange={(event) => onChange({ notes: event.target.value })}
								placeholder="Accès, contraintes de chantier, matériaux imposés, tout ce qui explique le prix"
							/>
						</Field>
						<Field label="Photos">
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
	)
}

interface FieldProps {
	label: string
	htmlFor?: string
	children: React.ReactNode
}

function Field({ label, htmlFor, children }: FieldProps) {
	return (
		<div className="space-y-1.5">
			<Label htmlFor={htmlFor} className="text-xs text-muted-foreground">
				{label}
			</Label>
			{children}
		</div>
	)
}

import {
	ArrowDown,
	ArrowUp,
	ChevronRight,
	Copy,
	Ellipsis,
	Library,
	Plus,
	Trash2,
} from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Command,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
} from '#/components/ui/command'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { Input } from '#/components/ui/input'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { Textarea } from '#/components/ui/textarea'
import type { CatalogItem } from '#/hooks/use-catalog-items'
import { cn } from '#/lib/utils'
import {
	BILLING_FREQUENCIES,
	type BillingFrequency,
	FREQUENCY_LABELS,
} from '#/pages/quotes/lib/billing-frequency'
import {
	COMMON_VAT_RATES_BP,
	formatCents,
	formatUnit,
	formatVatRateBp,
} from '#/pages/quotes/types'
import { AutoTextarea, Money } from '#/pages/quotes/ui/doc/primitives'
import { PhotoStrip } from '#/pages/quotes/ui/photo-strip'
import { UnitSelect } from '#/pages/quotes/ui/unit-select'

export interface EditableDocLine {
	clientId: string
	label: string
	quantity: string
	unitPrice: string
	vatRateBp: string
	unit?: string
	notes?: string
	photoKeys?: string[]
	discountPercent?: string
	frequency?: BillingFrequency
}

export interface LineProblems {
	label: boolean
	quantity: boolean
	unitPrice: boolean
}

export function lineProblems(line: EditableDocLine): LineProblems {
	const quantity = Number(line.quantity.replace(',', '.'))
	const priced = line.unitPrice.trim() !== ''
	const named = line.label.trim() !== ''
	return {
		label: !named && priced,
		quantity: !(Number.isFinite(quantity) && quantity > 0),
		unitPrice: named && !priced,
	}
}

const HEAD = 'h-9 text-muted-foreground text-xs'

const CUSTOM_VAT_RATE = 'custom'

function VatRateField({
	id,
	label,
	valueBp,
	onChange,
}: {
	id: string
	label: string
	valueBp: string
	onChange: (valueBp: string) => void
}) {
	const isCommonRate =
		valueBp !== '' &&
		(COMMON_VAT_RATES_BP as readonly number[]).includes(Number(valueBp))
	const selectValue =
		valueBp === '' ? '' : isCommonRate ? valueBp : CUSTOM_VAT_RATE

	return (
		<div className="grid gap-1.5">
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
				<SelectTrigger id={id} aria-label={label} className="w-full">
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
					aria-label={`${label}, taux en pourcentage`}
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

function CatalogPicker({
	items,
	onPick,
}: {
	items: readonly CatalogItem[]
	onPick: (item: CatalogItem) => void
}) {
	const [open, setOpen] = useState(false)
	const groups: [string, CatalogItem[]][] = [
		['Services', items.filter((item) => item.type === 'SERVICE')],
		['Produits', items.filter((item) => item.type === 'PRODUCT')],
	]
	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<Button type="button" variant="outline" size="sm" aria-expanded={open}>
					<Library aria-hidden />
					Ajouter depuis le catalogue
				</Button>
			</PopoverTrigger>
			<PopoverContent
				align="start"
				className="w-[min(30rem,calc(100vw-2rem))] p-0"
			>
				<Command>
					<CommandInput placeholder="Rechercher par nom ou référence…" />
					<CommandList>
						<CommandEmpty>
							{items.length === 0
								? 'Le catalogue est vide.'
								: 'Aucun article trouvé.'}
						</CommandEmpty>
						{groups
							.filter(([, rows]) => rows.length > 0)
							.map(([heading, rows]) => (
								<CommandGroup key={heading} heading={heading}>
									{rows.map((item) => (
										<CommandItem
											key={item.id}
											value={`${item.label} ${item.sku ?? ''} ${item.id}`}
											onSelect={() => {
												onPick(item)
												setOpen(false)
											}}
										>
											<span className="min-w-0 flex-1 truncate">
												{item.label}
											</span>
											{item.sku && (
												<span className="shrink-0 text-muted-foreground text-xs">
													{item.sku}
												</span>
											)}
											<span className="tabular shrink-0 text-muted-foreground text-xs">
												{formatCents(item.unitPriceCents)} /{' '}
												{formatUnit(item.unit)}
											</span>
										</CommandItem>
									))}
								</CommandGroup>
							))}
					</CommandList>
				</Command>
			</PopoverContent>
		</Popover>
	)
}

export interface LinesEditorDetails {
	photoUrls: Record<string, string | undefined>
	isUploading?: boolean
	onUpload: (index: number, file: File) => void
}

interface LinesEditorProps<L extends EditableDocLine> {
	lines: L[]
	idPrefix: string
	vatEnabled: boolean
	totalOf: (line: L) => number
	onPatch: (index: number, patch: Partial<EditableDocLine>) => void
	onMove: (index: number, delta: -1 | 1) => void
	onDuplicate: (index: number) => string
	onRemove: (index: number) => void
	onAdd: () => string
	catalog?: {
		items: readonly CatalogItem[]
		onPick: (item: CatalogItem) => string
	}
	customUnits?: string[]
	details?: LinesEditorDetails
}

export function LinesEditor<L extends EditableDocLine>({
	lines,
	idPrefix,
	vatEnabled,
	totalOf,
	onPatch,
	onMove,
	onDuplicate,
	onRemove,
	onAdd,
	catalog,
	customUnits,
	details,
}: LinesEditorProps<L>) {
	const [focusId, setFocusId] = useState<string | null>(null)
	const [openDetails, setOpenDetails] = useState<ReadonlySet<string>>(new Set())
	const withUnit = lines.some((line) => line.unit !== undefined)
	const withDiscount = lines.some((line) => line.discountPercent !== undefined)
	const withFrequency = lines.some((line) => line.frequency !== undefined)
	const columns =
		5 +
		(withUnit ? 1 : 0) +
		(withDiscount ? 1 : 0) +
		(withFrequency ? 1 : 0) +
		(vatEnabled ? 1 : 0)

	const toggleDetails = (clientId: string) =>
		setOpenDetails((current) => {
			const next = new Set(current)
			if (next.has(clientId)) next.delete(clientId)
			else next.add(clientId)
			return next
		})

	return (
		<div className="grid gap-3">
			<div className="overflow-hidden rounded-lg border">
				<Table
					className={cn(
						'min-w-[44rem]',
						(withDiscount || withFrequency) && 'min-w-[62rem]',
					)}
				>
					<caption className="sr-only">Lignes du document</caption>
					<TableHeader className="bg-muted/40">
						<TableRow className="hover:bg-transparent">
							<TableHead scope="col" className={cn(HEAD, 'w-8')}>
								N°
							</TableHead>
							<TableHead scope="col" className={cn(HEAD, 'min-w-56')}>
								Désignation
							</TableHead>
							<TableHead scope="col" className={cn(HEAD, 'w-20 text-right')}>
								Quantité
							</TableHead>
							{withUnit && (
								<TableHead scope="col" className={cn(HEAD, 'w-32')}>
									Unité
								</TableHead>
							)}
							<TableHead scope="col" className={cn(HEAD, 'w-32 text-right')}>
								PU HT
							</TableHead>
							{withDiscount && (
								<TableHead scope="col" className={cn(HEAD, 'w-24 text-right')}>
									Remise %
								</TableHead>
							)}
							{withFrequency && (
								<TableHead scope="col" className={cn(HEAD, 'w-36')}>
									Facturation
								</TableHead>
							)}
							{vatEnabled && (
								<TableHead scope="col" className={cn(HEAD, 'w-28')}>
									TVA
								</TableHead>
							)}
							<TableHead scope="col" className={cn(HEAD, 'w-28 text-right')}>
								Total HT
							</TableHead>
							<TableHead scope="col" className={cn(HEAD, 'w-12')}>
								<span className="sr-only">Actions</span>
							</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{lines.length === 0 && (
							<TableRow className="hover:bg-transparent">
								<TableCell
									colSpan={columns + 1}
									className="py-6 text-center text-muted-foreground"
								>
									Aucune ligne pour le moment.
								</TableCell>
							</TableRow>
						)}
						{lines.map((line, index) => {
							const problems = lineProblems(line)
							const flagged = Object.values(problems).some(Boolean)
							const position = index + 1
							const base = `${idPrefix}-${line.clientId}`
							const isOpen = openDetails.has(line.clientId)
							const photoCount = line.photoKeys?.length ?? 0
							return (
								<TableRow
									key={line.clientId}
									data-invalid={flagged || undefined}
									className="data-invalid:bg-state-alert/5"
								>
									<TableCell className="pt-4 align-top text-muted-foreground text-xs">
										{position}
									</TableCell>
									<TableCell className="whitespace-normal align-top">
										<AutoTextarea
											id={`${base}-label`}
											aria-label={`Désignation, ligne ${position}`}
											placeholder="Désignation de la prestation ou du produit"
											value={line.label}
											aria-invalid={problems.label || undefined}
											focusOnMount={focusId === line.clientId}
											onFocused={() => setFocusId(null)}
											onChange={(event) =>
												onPatch(index, { label: event.target.value })
											}
										/>
										{details && (
											<div className="mt-1.5 grid gap-2">
												<Button
													type="button"
													variant="ghost"
													size="xs"
													aria-expanded={isOpen}
													className="w-fit text-muted-foreground"
													onClick={() => toggleDetails(line.clientId)}
												>
													<ChevronRight
														aria-hidden
														className={cn(
															'transition-transform',
															isOpen && 'rotate-90',
														)}
													/>
													Note et photos
													{!isOpen && (line.notes?.trim() || photoCount > 0) ? (
														<span className="tabular-nums">
															{photoCount > 0 ? ` (${photoCount})` : ' •'}
														</span>
													) : null}
												</Button>
												{isOpen && (
													<div className="grid gap-3 rounded-md border bg-muted/30 p-3 lg:grid-cols-2">
														<Textarea
															aria-label="Note"
															rows={3}
															className="min-h-20 resize-y bg-background"
															value={line.notes ?? ''}
															onChange={(event) =>
																onPatch(index, {
																	notes: event.target.value,
																})
															}
															placeholder="Accès, contraintes de projet, matériaux imposés, tout ce qui explique le prix"
														/>
														<PhotoStrip
															photos={(line.photoKeys ?? []).map((key) => ({
																key,
																url: details.photoUrls[key],
															}))}
															isUploading={details.isUploading}
															onAdd={(file) => details.onUpload(index, file)}
															onRemove={(key) =>
																onPatch(index, {
																	photoKeys: (line.photoKeys ?? []).filter(
																		(photoKey) => photoKey !== key,
																	),
																})
															}
														/>
													</div>
												)}
											</div>
										)}
									</TableCell>
									<TableCell className="align-top">
										<Input
											id={`${base}-quantity`}
											aria-label={`Quantité, ligne ${position}`}
											inputMode="decimal"
											autoComplete="off"
											value={line.quantity}
											aria-invalid={problems.quantity || undefined}
											onFocus={(event) => event.currentTarget.select()}
											onChange={(event) =>
												onPatch(index, {
													quantity: event.target.value,
												})
											}
											className="tabular text-right"
										/>
									</TableCell>
									{withUnit && (
										<TableCell className="align-top">
											<UnitSelect
												value={line.unit ?? ''}
												customUnits={customUnits}
												onChange={(unit) => onPatch(index, { unit })}
											/>
										</TableCell>
									)}
									<TableCell className="align-top">
										<Input
											id={`${base}-price`}
											aria-label={`Prix unitaire HT, ligne ${position}`}
											inputMode="decimal"
											autoComplete="off"
											placeholder="0,00"
											value={line.unitPrice}
											aria-invalid={problems.unitPrice || undefined}
											onFocus={(event) => event.currentTarget.select()}
											onChange={(event) =>
												onPatch(index, {
													unitPrice: event.target.value,
												})
											}
											className="tabular text-right"
										/>
									</TableCell>
									{withDiscount && (
										<TableCell className="align-top">
											<Input
												id={`${base}-discount`}
												aria-label={`Remise en pourcentage, ligne ${position}`}
												inputMode="decimal"
												autoComplete="off"
												placeholder="0"
												value={line.discountPercent ?? ''}
												onFocus={(event) => event.currentTarget.select()}
												onChange={(event) =>
													onPatch(index, {
														discountPercent: event.target.value,
													})
												}
												className="tabular text-right"
											/>
										</TableCell>
									)}
									{withFrequency && (
										<TableCell className="align-top">
											<Select
												value={line.frequency ?? 'one_shot'}
												onValueChange={(frequency) =>
													onPatch(index, {
														frequency: frequency as BillingFrequency,
													})
												}
											>
												<SelectTrigger
													id={`${base}-frequency`}
													aria-label={`Fréquence de facturation, ligne ${position}`}
													className="w-full"
												>
													<SelectValue />
												</SelectTrigger>
												<SelectContent>
													{BILLING_FREQUENCIES.map((frequency) => (
														<SelectItem key={frequency} value={frequency}>
															{FREQUENCY_LABELS[frequency]}
														</SelectItem>
													))}
												</SelectContent>
											</Select>
										</TableCell>
									)}
									{vatEnabled && (
										<TableCell className="align-top">
											<VatRateField
												id={`${base}-vat`}
												label={`TVA, ligne ${position}`}
												valueBp={line.vatRateBp}
												onChange={(vatRateBp) => onPatch(index, { vatRateBp })}
											/>
										</TableCell>
									)}
									<TableCell className="align-top">
										<div className="flex h-9 items-center justify-end font-medium text-sm">
											<Money cents={totalOf(line)} />
										</div>
									</TableCell>
									<TableCell className="align-top">
										<div className="flex justify-end">
											<DropdownMenu>
												<DropdownMenuTrigger asChild>
													<Button
														type="button"
														variant="ghost"
														size="icon-sm"
														aria-label={`Actions de la ligne ${position}`}
													>
														<Ellipsis aria-hidden />
													</Button>
												</DropdownMenuTrigger>
												<DropdownMenuContent align="end">
													<DropdownMenuItem
														disabled={index === 0}
														onSelect={() => onMove(index, -1)}
													>
														<ArrowUp aria-hidden /> Monter la ligne
													</DropdownMenuItem>
													<DropdownMenuItem
														disabled={index === lines.length - 1}
														onSelect={() => onMove(index, 1)}
													>
														<ArrowDown aria-hidden /> Descendre la ligne
													</DropdownMenuItem>
													<DropdownMenuItem
														onSelect={() => setFocusId(onDuplicate(index))}
													>
														<Copy aria-hidden /> Dupliquer la ligne
													</DropdownMenuItem>
													<DropdownMenuSeparator />
													<DropdownMenuItem
														variant="destructive"
														onSelect={() => onRemove(index)}
													>
														<Trash2 aria-hidden /> Supprimer la ligne
													</DropdownMenuItem>
												</DropdownMenuContent>
											</DropdownMenu>
										</div>
									</TableCell>
								</TableRow>
							)
						})}
					</TableBody>
				</Table>
			</div>
			<div className="flex flex-wrap items-center gap-2">
				<Button
					type="button"
					variant="outline"
					size="sm"
					onClick={() => setFocusId(onAdd())}
				>
					<Plus aria-hidden />
					Ajouter une ligne
				</Button>
				{catalog && (
					<CatalogPicker
						items={catalog.items}
						onPick={(item) => setFocusId(catalog.onPick(item))}
					/>
				)}
			</div>
		</div>
	)
}

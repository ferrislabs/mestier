import { Link, useNavigate } from '@tanstack/react-router'
import {
	AlertCircle,
	ArrowLeft,
	ArrowRightLeft,
	FileText,
	Loader2,
	Plus,
	Trash2,
} from 'lucide-react'
import type * as React from 'react'
import { useEffect, useMemo, useState } from 'react'
import { LegalMentionSelector } from '#/components/legal-mention-selector'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
	AlertDialogTrigger,
} from '#/components/ui/alert-dialog'
import { Button } from '#/components/ui/button'
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
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
} from '#/components/ui/surface'
import { Textarea } from '#/components/ui/textarea'
import { type CatalogItem, useCatalogItems } from '#/hooks/use-catalog-items'
import {
	type Customer,
	type CustomerContext,
	useCustomerContexts,
	useCustomers,
	useUploadFile,
} from '#/hooks/use-customers'
import { useConvertQuoteToInvoice } from '#/hooks/use-invoices'
import {
	type LegalMentionTemplate,
	useLegalMentionTemplates,
} from '#/hooks/use-legal-mentions'
import {
	type Quote,
	type QuoteStatus,
	useDeleteQuote,
	useQuote,
	useUpdateQuote,
} from '#/hooks/use-quotes'
import {
	type ServiceRateUnit,
	useReferenceCatalog,
} from '#/hooks/use-reference-catalog'
import {
	centsToEuros,
	customerContextDisplayName,
	customerDisplayName,
	emptyQuoteLine,
	eurosToCents,
	formatCents,
	formatUnit,
	type QuoteFormValues,
	type QuoteLineFormValues,
	quoteStatusLabel,
} from '#/pages/quotes/types'

interface QuoteEditFeatureProps {
	quoteId: string
}

export function QuoteEditFeature({ quoteId }: QuoteEditFeatureProps) {
	const quote = useQuote(quoteId)

	if (quote.isLoading) return <QuoteEditLoading />

	if (quote.isError) {
		return (
			<QuoteEditError
				title="Impossible de charger le devis"
				message={quote.error.message}
				onRetry={() => void quote.refetch()}
			/>
		)
	}

	if (!quote.data?.data) {
		return (
			<QuoteEditError
				title="Devis introuvable"
				message="Aucun devis ne correspond à cet identifiant."
			/>
		)
	}

	return <QuoteEditWorkspace quote={quote.data.data} />
}

function QuoteEditWorkspace({ quote }: { quote: Quote }) {
	const navigate = useNavigate()
	const customers = useCustomers(quote.org_id)
	const legalMentionTemplates = useLegalMentionTemplates(quote.org_id)
	const catalog = useReferenceCatalog(quote.org_id, {
		employees: false,
		equipment: false,
	})
	const serviceRates = useMemo(
		() => catalog.serviceRates.data?.data ?? [],
		[catalog.serviceRates.data],
	)
	const products = useMemo(
		() => catalog.products.data?.data ?? [],
		[catalog.products.data],
	)
	const catalogItems = useCatalogItems(serviceRates, products)
	const updateQuote = useUpdateQuote()
	const deleteQuote = useDeleteQuote(quote.org_id)
	const convertQuote = useConvertQuoteToInvoice()
	const uploadFile = useUploadFile()
	const [status, setStatus] = useState<QuoteStatus>(quote.status)
	const [values, setValues] = useState<QuoteFormValues>(() =>
		quoteToForm(quote, catalogItems),
	)
	const customerContexts = useCustomerContexts(
		values.customerId,
		Boolean(values.customerId),
	)

	useEffect(() => {
		setStatus(quote.status)
		setValues(quoteToForm(quote, catalogItems))
	}, [catalogItems, quote])

	const updateValues = (patch: Partial<QuoteFormValues>) => {
		setValues((current) => ({ ...current, ...patch }))
	}

	const updateLine = (index: number, patch: Partial<QuoteLineFormValues>) => {
		setValues((current) => {
			const lines = [...current.lines]
			const line = lines[index]
			if (!line) return current
			lines[index] = { ...line, ...patch }
			return { ...current, lines }
		})
	}

	const selectCatalogItem = (index: number, catalogItemId: string) => {
		const catalogItem = catalogItems.find((item) => item.id === catalogItemId)
		if (!catalogItem) {
			updateLine(index, {
				catalogItemId: '',
				catalogItemType: 'CUSTOM',
				serviceRateId: '',
			})
			return
		}

		updateLine(index, {
			catalogItemId: catalogItem.id,
			catalogItemType: catalogItem.type,
			serviceRateId: catalogItem.type === 'SERVICE' ? catalogItem.sourceId : '',
			label: catalogItem.label,
			unit: catalogItem.unit,
			unitPrice: centsToEuros(catalogItem.unitPriceCents),
			notes: catalogItem.description || values.lines[index]?.notes || '',
		})
	}

	const addLine = () => {
		setValues((current) => ({
			...current,
			lines: [
				...current.lines,
				emptyQuoteLine(`line-${Date.now()}-${current.lines.length}`),
			],
		}))
	}

	const removeLine = (index: number) => {
		setValues((current) => {
			const lines = current.lines.filter(
				(_line, lineIndex) => lineIndex !== index,
			)
			return { ...current, lines: lines.length ? lines : [emptyQuoteLine()] }
		})
	}

	const uploadLinePhoto = async (index: number, file: File) => {
		const uploaded = await uploadFile.mutateAsync(file)
		setValues((current) => {
			const lines = [...current.lines]
			const line = lines[index]
			if (!line) return current
			lines[index] = {
				...line,
				photoKeys: [...line.photoKeys, uploaded.data.key],
			}
			return { ...current, lines }
		})
	}

	const canSave =
		Boolean(values.title.trim()) &&
		Boolean(values.customerId) &&
		Boolean(values.customerContextId) &&
		values.lines.every((line) => {
			const quantity = Number(line.quantity.replace(',', '.'))
			return (
				line.label.trim() &&
				Number.isFinite(quantity) &&
				quantity > 0 &&
				line.unitPrice.trim() &&
				eurosToCents(line.unitPrice) >= 0
			)
		})

	const error =
		customers.error?.message ??
		customerContexts.error?.message ??
		catalog.serviceRates.error?.message ??
		catalog.products.error?.message ??
		updateQuote.error?.message ??
		deleteQuote.error?.message ??
		convertQuote.error?.message ??
		uploadFile.error?.message ??
		null

	const saveQuote = async () => {
		if (!canSave) return
		const depositBasis =
			values.depositBasis === 'PERCENT' || values.depositBasis === 'FIXED'
				? values.depositBasis
				: null
		const depositValue =
			depositBasis && values.depositValue.trim()
				? values.depositValue.trim()
				: null
		await updateQuote.mutateAsync({
			path: { quote_id: quote.id },
			body: {
				title: values.title.trim(),
				customer_id: values.customerId,
				customer_context_id: values.customerContextId,
				legal_mention_template_ids: values.legalMentionTemplateIds,
				deposit_basis: depositBasis,
				deposit_value: depositValue,
				status,
				lines: values.lines.map((line) => ({
					service_rate_id: line.serviceRateId || null,
					label: line.label.trim(),
					quantity: line.quantity.replace(',', '.').trim(),
					unit: line.unit,
					unit_price_cents: eurosToCents(line.unitPrice),
					vat_rate: line.vatRate.replace(',', '.').trim(),
					notes: line.notes.trim() || null,
					photo_keys: line.photoKeys,
				})),
			},
		})
	}

	const deleteCurrentQuote = async () => {
		await deleteQuote.mutateAsync({
			path: { quote_id: quote.id },
		})
		await navigate({ to: '/quotes' })
	}

	const convertCurrentQuote = async () => {
		const result = await convertQuote.mutateAsync({
			path: { quote_id: quote.id },
		})
		await navigate({
			to: '/invoices/$invoiceId',
			params: { invoiceId: result.data.id },
		})
	}

	return (
		<QuoteEditUI
			quote={quote}
			values={values}
			status={status}
			customers={customers.data?.data ?? []}
			legalMentionTemplates={legalMentionTemplates.data?.data ?? []}
			isLegalMentionTemplatesLoading={legalMentionTemplates.isLoading}
			customerContexts={customerContexts.data?.data ?? []}
			catalogItems={catalogItems}
			error={error}
			isLoading={
				customers.isLoading ||
				customerContexts.isLoading ||
				catalog.serviceRates.isLoading ||
				catalog.products.isLoading
			}
			isSaving={updateQuote.isPending}
			isDeleting={deleteQuote.isPending}
			isConverting={convertQuote.isPending}
			isUploading={uploadFile.isPending}
			canSave={canSave}
			canConvert={quote.status === 'ACCEPTED' || quote.status === 'SENT'}
			onChange={(patch) => {
				if (patch.customerId !== undefined) {
					updateValues({ customerId: patch.customerId, customerContextId: '' })
					return
				}
				updateValues(patch)
			}}
			onStatusChange={setStatus}
			onLineChange={updateLine}
			onSelectCatalogItem={selectCatalogItem}
			onAddLine={addLine}
			onRemoveLine={removeLine}
			onUploadLinePhoto={uploadLinePhoto}
			onSave={saveQuote}
			onDelete={deleteCurrentQuote}
			onConvert={convertCurrentQuote}
		/>
	)
}

function QuoteEditUI({
	quote,
	values,
	status,
	customers,
	legalMentionTemplates,
	isLegalMentionTemplatesLoading,
	customerContexts,
	catalogItems,
	error,
	isLoading,
	isSaving,
	isDeleting,
	isConverting,
	isUploading,
	canSave,
	canConvert,
	onChange,
	onStatusChange,
	onLineChange,
	onSelectCatalogItem,
	onAddLine,
	onRemoveLine,
	onUploadLinePhoto,
	onSave,
	onDelete,
	onConvert,
}: {
	quote: Quote
	values: QuoteFormValues
	status: QuoteStatus
	customers: Customer[]
	legalMentionTemplates: LegalMentionTemplate[]
	isLegalMentionTemplatesLoading: boolean
	customerContexts: CustomerContext[]
	catalogItems: CatalogItem[]
	error: string | null
	isLoading: boolean
	isSaving: boolean
	isDeleting: boolean
	isConverting: boolean
	isUploading: boolean
	canSave: boolean
	canConvert: boolean
	onChange: (patch: Partial<QuoteFormValues>) => void
	onStatusChange: (status: QuoteStatus) => void
	onLineChange: (index: number, patch: Partial<QuoteLineFormValues>) => void
	onSelectCatalogItem: (index: number, catalogItemId: string) => void
	onAddLine: () => void
	onRemoveLine: (index: number) => void
	onUploadLinePhoto: (index: number, file: File) => Promise<void>
	onSave: () => void
	onDelete: () => void
	onConvert: () => void
}) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={quote.reference}
				title={quote.title}
				description="Visualisez et modifiez le contenu du devis."
				actions={
					<div className="flex flex-col gap-2 sm:flex-row">
						<Button asChild variant="outline">
							<Link to="/quotes">
								<ArrowLeft />
								Retour
							</Link>
						</Button>
						<AlertDialog>
							<AlertDialogTrigger asChild>
								<Button
									type="button"
									variant="destructive"
									disabled={isDeleting || isSaving || isConverting}
								>
									{isDeleting ? (
										<Loader2 className="animate-spin" />
									) : (
										<Trash2 />
									)}
									Supprimer
								</Button>
							</AlertDialogTrigger>
							<AlertDialogContent>
								<AlertDialogHeader>
									<AlertDialogTitle>Supprimer ce devis ?</AlertDialogTitle>
									<AlertDialogDescription>
										Le devis {quote.reference} sera supprimé de la liste. Cette
										action est irréversible.
									</AlertDialogDescription>
								</AlertDialogHeader>
								<AlertDialogFooter>
									<AlertDialogCancel disabled={isDeleting}>
										Annuler
									</AlertDialogCancel>
									<AlertDialogAction
										disabled={isDeleting}
										className="bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20"
										onClick={(event) => {
											event.preventDefault()
											void onDelete()
										}}
									>
										{isDeleting ? (
											<Loader2 className="animate-spin" />
										) : (
											<Trash2 />
										)}
										Supprimer
									</AlertDialogAction>
								</AlertDialogFooter>
							</AlertDialogContent>
						</AlertDialog>
						<Button
							type="button"
							variant="outline"
							disabled={!canConvert || isConverting || isSaving || isDeleting}
							onClick={() => void onConvert()}
						>
							{isConverting ? (
								<Loader2 className="animate-spin" />
							) : (
								<ArrowRightLeft />
							)}
							Convertir en facture
						</Button>
						<Button
							type="button"
							disabled={!canSave || isSaving || isDeleting || isConverting}
							onClick={onSave}
						>
							{isSaving ? <Loader2 className="animate-spin" /> : <FileText />}
							Enregistrer
						</Button>
					</div>
				}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			<div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_320px]">
				<div className="space-y-5">
					<SectionCard>
						<SectionHeader
							title="Informations"
							description="Objet, client et contexte associés au devis."
						/>
						<div className="grid gap-4 p-5 md:grid-cols-2">
							<FieldBlock label="Objet du devis">
								<Input
									value={values.title}
									onChange={(event) => onChange({ title: event.target.value })}
									placeholder="Ex. Rénovation salle de bain"
								/>
							</FieldBlock>
							<FieldBlock label="Statut">
								<Select
									value={status}
									onValueChange={(value) =>
										onStatusChange(value as QuoteStatus)
									}
								>
									<SelectTrigger className="w-full">
										<SelectValue />
									</SelectTrigger>
									<SelectContent>
										<SelectItem value="DRAFT">Brouillon</SelectItem>
										<SelectItem value="SENT">Envoyé</SelectItem>
										<SelectItem value="ACCEPTED">Accepté</SelectItem>
										<SelectItem value="DECLINED">Refusé</SelectItem>
										<SelectItem value="CANCELLED">Annulé</SelectItem>
									</SelectContent>
								</Select>
							</FieldBlock>
							<FieldBlock label="Client">
								<Select
									value={values.customerId}
									onValueChange={(customerId) => onChange({ customerId })}
								>
									<SelectTrigger className="w-full">
										<SelectValue placeholder="Sélectionner un client" />
									</SelectTrigger>
									<SelectContent>
										{customers.map((customer) => (
											<SelectItem key={customer.id} value={customer.id}>
												{customerDisplayName(customer)}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</FieldBlock>
							<FieldBlock label="Contexte client">
								<Select
									value={values.customerContextId}
									onValueChange={(customerContextId) =>
										onChange({ customerContextId })
									}
									disabled={!values.customerId || isLoading}
								>
									<SelectTrigger className="w-full">
										<SelectValue placeholder="Sélectionner un contexte" />
									</SelectTrigger>
									<SelectContent>
										{customerContexts.map((customerContext) => (
											<SelectItem
												key={customerContext.id}
												value={customerContext.id}
											>
												{customerContextDisplayName(customerContext)}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</FieldBlock>
						</div>
					</SectionCard>

					<SectionCard>
						<SectionHeader
							title={`Lignes (${values.lines.length})`}
							description="Chaque ligne peut venir du catalogue ou rester libre."
							actions={
								<Button
									type="button"
									variant="outline"
									size="sm"
									onClick={onAddLine}
								>
									<Plus />
									Ajouter
								</Button>
							}
						/>
						<div className="divide-y">
							{values.lines.map((line, index) => (
								<QuoteLineEditor
									key={line.clientId}
									index={index}
									line={line}
									catalogItems={catalogItems}
									canRemove={values.lines.length > 1}
									isUploading={isUploading}
									onChange={(patch) => onLineChange(index, patch)}
									onSelectCatalogItem={(catalogItemId) =>
										onSelectCatalogItem(index, catalogItemId)
									}
									onRemove={() => onRemoveLine(index)}
									onUploadPhoto={(file) => onUploadLinePhoto(index, file)}
								/>
							))}
						</div>
					</SectionCard>

					<SectionCard>
						<SectionHeader
							title="Acompte"
							description="Montant d'acompte mentionné sur le devis (informatif)."
						/>
						<div className="grid gap-4 p-5 md:grid-cols-2">
							<FieldBlock label="Type d'acompte">
								<Select
									value={values.depositBasis || 'NONE'}
									onValueChange={(v) =>
										onChange({
											depositBasis: v === 'NONE' ? '' : v,
											depositValue: '',
										})
									}
								>
									<SelectTrigger className="w-full">
										<SelectValue />
									</SelectTrigger>
									<SelectContent>
										<SelectItem value="NONE">Aucun</SelectItem>
										<SelectItem value="PERCENT">Pourcentage</SelectItem>
										<SelectItem value="FIXED">Montant fixe</SelectItem>
									</SelectContent>
								</Select>
							</FieldBlock>
							{values.depositBasis === 'PERCENT' ||
							values.depositBasis === 'FIXED' ? (
								<FieldBlock
									label={
										values.depositBasis === 'PERCENT'
											? 'Pourcentage (%)'
											: 'Montant (€)'
									}
								>
									<div className="relative">
										<Input
											inputMode="decimal"
											value={values.depositValue}
											onChange={(event) =>
												onChange({ depositValue: event.target.value })
											}
											placeholder={
												values.depositBasis === 'PERCENT' ? 'Ex. 30' : 'Ex. 300'
											}
											className="pr-6"
										/>
										<span className="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-xs text-muted-foreground">
											{values.depositBasis === 'PERCENT' ? '%' : '€'}
										</span>
									</div>
								</FieldBlock>
							) : null}
						</div>
					</SectionCard>

					<SectionCard>
						<SectionHeader
							title="Mentions légales"
							description="Sélectionnez les mentions légales à inclure dans ce devis."
						/>
						<div className="p-5">
							<LegalMentionSelector
								templates={legalMentionTemplates}
								selectedIds={values.legalMentionTemplateIds}
								onChange={(ids) => onChange({ legalMentionTemplateIds: ids })}
								isLoading={isLegalMentionTemplatesLoading}
							/>
						</div>
					</SectionCard>
				</div>

				<aside className="h-fit rounded-lg border bg-card p-5 shadow-sm xl:sticky xl:top-5">
					<p className="text-sm font-semibold">Résumé</p>
					<div className="mt-4 space-y-3 text-sm">
						<SummaryRow label="Référence" value={quote.reference} />
						<SummaryRow label="Statut" value={quoteStatusLabel(status)} />
						<SummaryRow label="Objet" value={values.title || 'Non renseigné'} />
						<SummaryRow label="Lignes" value={String(values.lines.length)} />
						<SummaryRow
							label="Total HT"
							value={formatCents(quote.total_ht_cents)}
						/>
						<SummaryRow
							label="TVA"
							value={formatCents(quote.total_vat_cents)}
						/>
						<SummaryRow
							label="Total TTC"
							value={formatCents(quote.total_ttc_cents)}
							strong
						/>
						{values.depositBasis === 'PERCENT' && values.depositValue ? (
							<SummaryRow label="Acompte" value={`${values.depositValue} %`} />
						) : null}
						{values.depositBasis === 'FIXED' && values.depositValue ? (
							<SummaryRow label="Acompte" value={`${values.depositValue} €`} />
						) : null}
					</div>
				</aside>
			</div>
		</PageShell>
	)
}

function QuoteLineEditor({
	index,
	line,
	catalogItems,
	canRemove,
	isUploading,
	onChange,
	onSelectCatalogItem,
	onRemove,
	onUploadPhoto,
}: {
	index: number
	line: QuoteLineFormValues
	catalogItems: CatalogItem[]
	canRemove: boolean
	isUploading: boolean
	onChange: (patch: Partial<QuoteLineFormValues>) => void
	onSelectCatalogItem: (catalogItemId: string) => void
	onRemove: () => void
	onUploadPhoto: (file: File) => Promise<void>
}) {
	const serviceItems = catalogItems.filter((item) => item.type === 'SERVICE')
	const productItems = catalogItems.filter((item) => item.type === 'PRODUCT')
	const selectedCatalogItem = catalogItems.find(
		(item) => item.id === line.catalogItemId,
	)

	return (
		<div className="bg-card p-4">
			<div className="mb-4 flex items-center justify-between gap-3">
				<div>
					<p className="text-sm font-semibold">
						{line.label.trim() || `Ligne ${index + 1}`}
					</p>
					<p className="text-xs text-muted-foreground">
						{quoteLineSourceLabel(line.catalogItemType)}
					</p>
				</div>
				<div className="flex items-center gap-3">
					<p className="text-sm font-semibold">
						{formatCents(lineTotalCents(line))}
					</p>
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
			</div>

			<div className="grid gap-4 lg:grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)_96px_128px_128px_80px]">
				<div className="lg:col-span-2">
					<FieldBlock label="Catalogue">
						<Select
							value={line.catalogItemId || 'custom'}
							onValueChange={(value) =>
								onSelectCatalogItem(value === 'custom' ? '' : value)
							}
						>
							<SelectTrigger className="w-full">
								<SelectValue placeholder="Ligne libre" />
							</SelectTrigger>
							<SelectContent>
								<SelectItem value="custom">Ligne libre</SelectItem>
								{serviceItems.length > 0 ? (
									<SelectGroup>
										<SelectLabel>Services</SelectLabel>
										{serviceItems.map((item) => (
											<SelectItem key={item.id} value={item.id}>
												{catalogItemOptionLabel(item)}
											</SelectItem>
										))}
									</SelectGroup>
								) : null}
								{productItems.length > 0 ? (
									<SelectGroup>
										<SelectLabel>Produits</SelectLabel>
										{productItems.map((item) => (
											<SelectItem key={item.id} value={item.id}>
												{catalogItemOptionLabel(item)}
											</SelectItem>
										))}
									</SelectGroup>
								) : null}
							</SelectContent>
						</Select>
						{selectedCatalogItem ? (
							<p className="truncate text-xs text-muted-foreground">
								{catalogItemDetail(selectedCatalogItem)}
							</p>
						) : null}
					</FieldBlock>
				</div>
				<FieldBlock label="Libellé">
					<Input
						value={line.label}
						onChange={(event) => onChange({ label: event.target.value })}
					/>
				</FieldBlock>
				<FieldBlock label="Quantité">
					<Input
						inputMode="decimal"
						value={line.quantity}
						onChange={(event) => onChange({ quantity: event.target.value })}
					/>
				</FieldBlock>
				<FieldBlock label="Prix unitaire">
					<Input
						inputMode="decimal"
						value={line.unitPrice}
						onChange={(event) => onChange({ unitPrice: event.target.value })}
					/>
				</FieldBlock>
				<FieldBlock label="TVA">
					<div className="relative">
						<Input
							inputMode="decimal"
							value={line.vatRate}
							onChange={(event) => onChange({ vatRate: event.target.value })}
							className="pr-6"
						/>
						<span className="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-xs text-muted-foreground">
							%
						</span>
					</div>
				</FieldBlock>
				<FieldBlock label="Unité">
					<Select
						value={line.unit}
						onValueChange={(unit) =>
							onChange({ unit: unit as ServiceRateUnit })
						}
					>
						<SelectTrigger className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="HOUR">
								{line.catalogItemType === 'PRODUCT'
									? 'unité'
									: formatUnit('HOUR')}
							</SelectItem>
							<SelectItem value="ML">{formatUnit('ML')}</SelectItem>
							<SelectItem value="M2">{formatUnit('M2')}</SelectItem>
						</SelectContent>
					</Select>
				</FieldBlock>
				<div className="lg:col-span-2">
					<FieldBlock label="Notes">
						<Textarea
							value={line.notes}
							onChange={(event) => onChange({ notes: event.target.value })}
							placeholder="Précisions utiles"
						/>
					</FieldBlock>
				</div>
				<FieldBlock label="Photos">
					<label className="flex h-10 cursor-pointer items-center justify-center gap-2 rounded-lg border bg-card px-3 text-sm font-medium text-primary shadow-xs hover:bg-brand-soft">
						Ajouter
						<input
							type="file"
							accept="image/*"
							className="sr-only"
							disabled={isUploading}
							onChange={(event) => {
								const file = event.target.files?.[0]
								if (file) void onUploadPhoto(file)
								event.target.value = ''
							}}
						/>
					</label>
					{line.photoKeys.length > 0 ? (
						<p className="truncate text-xs text-muted-foreground">
							{line.photoKeys.length} fichier
							{line.photoKeys.length > 1 ? 's' : ''}
						</p>
					) : null}
				</FieldBlock>
			</div>
		</div>
	)
}

function quoteToForm(
	quote: Quote,
	catalogItems: CatalogItem[],
): QuoteFormValues {
	const lines: QuoteLineFormValues[] = quote.lines.map((line, index) => {
		const catalogItem = line.service_rate_id
			? catalogItems.find(
					(item) =>
						item.type === 'SERVICE' && item.sourceId === line.service_rate_id,
				)
			: undefined

		return {
			clientId: line.id || `line-${index + 1}`,
			catalogItemId: catalogItem?.id ?? '',
			catalogItemType: catalogItem?.type ?? 'CUSTOM',
			serviceRateId: line.service_rate_id ?? '',
			label: line.label,
			quantity: line.quantity,
			unit: line.unit,
			unitPrice: centsToEuros(line.unit_price_cents),
			vatRate: line.vat_rate,
			notes: line.notes ?? '',
			photoKeys: line.photo_keys,
		}
	})

	return {
		title: quote.title,
		customerId: quote.customer_id,
		customerContextId: quote.customer_context_id,
		legalMentionTemplateIds: quote.legal_mention_template_ids ?? [],
		lines: lines.length ? lines : [emptyQuoteLine()],
		depositBasis: quote.deposit_basis ?? '',
		depositValue: quote.deposit_value ?? '',
	}
}

function QuoteEditLoading() {
	return (
		<PageShell>
			<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
				<Loader2 className="size-5 animate-spin" />
				Chargement du devis…
			</SectionCard>
		</PageShell>
	)
}

function QuoteEditError({
	title,
	message,
	onRetry,
}: {
	title: string
	message: string
	onRetry?: () => void
}) {
	return (
		<PageShell>
			<SectionCard className="flex min-h-72 flex-col items-center justify-center gap-3 p-8 text-center">
				<AlertCircle className="size-6 text-destructive" />
				<div>
					<p className="font-semibold">{title}</p>
					<p className="mt-1 text-sm text-muted-foreground">{message}</p>
				</div>
				<div className="flex gap-2">
					<Button asChild variant="outline">
						<Link to="/quotes">Retour aux devis</Link>
					</Button>
					{onRetry ? (
						<Button type="button" onClick={onRetry}>
							Réessayer
						</Button>
					) : null}
				</div>
			</SectionCard>
		</PageShell>
	)
}

function FieldBlock({
	label,
	children,
}: {
	label: string
	children: React.ReactNode
}) {
	return (
		<div className="flex min-w-0 flex-col gap-2">
			<Label>{label}</Label>
			{children}
		</div>
	)
}

function SummaryRow({
	label,
	value,
	strong,
}: {
	label: string
	value: string
	strong?: boolean
}) {
	return (
		<div className="flex items-center justify-between gap-4">
			<span className="text-muted-foreground">{label}</span>
			<span className={strong ? 'font-semibold' : 'truncate font-medium'}>
				{value}
			</span>
		</div>
	)
}

function lineTotalCents(line: QuoteLineFormValues): number {
	const quantity = Number(line.quantity.replace(',', '.'))
	if (!Number.isFinite(quantity) || quantity <= 0) return 0
	return Math.round(quantity * eurosToCents(line.unitPrice))
}

function quoteLineSourceLabel(type: QuoteLineFormValues['catalogItemType']) {
	if (type === 'SERVICE') return 'Service catalogue'
	if (type === 'PRODUCT') return 'Produit catalogue'
	return 'Ligne libre'
}

function catalogItemOptionLabel(item: CatalogItem): string {
	const reference = item.sku ? ` · ${item.sku}` : ''
	return `${item.label}${reference} · ${formatCents(item.unitPriceCents)}`
}

function catalogItemDetail(item: CatalogItem): string {
	const unit =
		item.type === 'PRODUCT' && item.unit === 'HOUR'
			? 'unité'
			: formatUnit(item.unit)
	const description = item.description ? ` · ${item.description}` : ''
	return `${item.type === 'PRODUCT' ? 'Produit' : 'Service'} · ${formatCents(
		item.unitPriceCents,
	)} / ${unit}${description}`
}

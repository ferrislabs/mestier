import { Link, useNavigate } from '@tanstack/react-router'
import {
	AlertCircle,
	ArrowLeft,
	FileText,
	Loader2,
	Plus,
	Trash2,
} from 'lucide-react'
import type * as React from 'react'
import { useEffect, useRef, useState } from 'react'
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
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
} from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { type CatalogItem, useCatalogItems } from '#/hooks/use-catalog-items'
import {
	type Customer,
	type CustomerContext,
	useCustomerContexts,
	useCustomers,
	useUploadFile,
} from '#/hooks/use-customers'
import { useFileUrls } from '#/hooks/use-file-url'
import {
	type Quote,
	type QuoteStatus,
	useDeleteQuote,
	useQuote,
	useUpdateQuote,
} from '#/hooks/use-quotes'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { buildOrgPath } from '#/modules/org-path'
import {
	centsToEuros,
	customerDisplayName,
	emptyQuoteLine,
	eurosToCents,
	formatCents,
	type QuoteFormValues,
	type QuoteLineFormValues,
	quoteLineTotalCents,
	quoteStatusLabel,
} from '#/pages/quotes/types'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'
import { QuoteLineEditor } from '#/pages/quotes/ui/quote-line-editor'

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
	const { activeOrganization } = useActiveOrganization()
	const customers = useCustomers(quote.organization_id)
	const catalog = useReferenceCatalog(quote.organization_id, {
		members: false,
		employeeProfiles: false,
		equipment: false,
	})
	const catalogItems = useCatalogItems(
		catalog.serviceRates.data?.data,
		catalog.products.data?.data,
	)
	const updateQuote = useUpdateQuote()
	const deleteQuote = useDeleteQuote(quote.organization_id)
	const uploadFile = useUploadFile()
	const [status, setStatus] = useState<QuoteStatus>(quote.status)
	const [values, setValues] = useState<QuoteFormValues>(() =>
		quoteToForm(quote, catalogItems),
	)
	const customerContexts = useCustomerContexts(
		values.customerId,
		Boolean(values.customerId),
	)

	// Reloads the form when the server's quote changes, and only then. The
	// catalogue is in the dependencies because `quoteToForm` reads it, but it
	// resolves on its own schedule: without this guard, a catalogue arriving
	// after the quote overwrote whatever the user had already typed.
	const loadedVersion = useRef<string | null>(null)
	useEffect(() => {
		const version = `${quote.id}:${quote.updated_at}`
		if (loadedVersion.current === version) return
		loadedVersion.current = version

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

	const totalCents = values.lines.reduce(
		(sum, line) => sum + quoteLineTotalCents(line),
		0,
	)
	const error =
		customers.error?.message ??
		customerContexts.error?.message ??
		catalog.serviceRates.error?.message ??
		catalog.products.error?.message ??
		updateQuote.error?.message ??
		deleteQuote.error?.message ??
		uploadFile.error?.message ??
		null

	const saveQuote = async () => {
		if (!canSave) return
		await updateQuote.mutateAsync({
			path: { quote_id: quote.id },
			body: {
				title: values.title.trim(),
				customer_id: values.customerId,
				customer_context_id: values.customerContextId,
				status,
				lines: values.lines.map((line) => ({
					service_rate_id: line.serviceRateId || null,
					label: line.label.trim(),
					quantity: line.quantity.replace(',', '.').trim(),
					unit: line.unit,
					unit_price_cents: eurosToCents(line.unitPrice),
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
		await navigate({ to: buildOrgPath(activeOrganization.slug, '/crm/quotes') })
	}

	return (
		<QuoteEditUI
			quote={quote}
			values={values}
			status={status}
			customers={customers.data?.data ?? []}
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
			isUploading={uploadFile.isPending}
			totalCents={totalCents}
			canSave={canSave}
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
		/>
	)
}

function QuoteEditUI({
	quote,
	values,
	status,
	customers,
	customerContexts,
	catalogItems,
	error,
	isLoading,
	isSaving,
	isDeleting,
	isUploading,
	totalCents,
	canSave,
	onChange,
	onStatusChange,
	onLineChange,
	onSelectCatalogItem,
	onAddLine,
	onRemoveLine,
	onUploadLinePhoto,
	onSave,
	onDelete,
}: {
	quote: Quote
	values: QuoteFormValues
	status: QuoteStatus
	customers: Customer[]
	customerContexts: CustomerContext[]
	catalogItems: CatalogItem[]
	error: string | null
	isLoading: boolean
	isSaving: boolean
	isDeleting: boolean
	isUploading: boolean
	totalCents: number
	canSave: boolean
	onChange: (patch: Partial<QuoteFormValues>) => void
	onStatusChange: (status: QuoteStatus) => void
	onLineChange: (index: number, patch: Partial<QuoteLineFormValues>) => void
	onSelectCatalogItem: (index: number, catalogItemId: string) => void
	onAddLine: () => void
	onRemoveLine: (index: number) => void
	onUploadLinePhoto: (index: number, file: File) => Promise<void>
	onSave: () => void
	onDelete: () => void
}) {
	const { activeOrganization } = useActiveOrganization()
	// Only one line open at a time, so a long quote stays readable. Opening on
	// the first line matches the create form.
	const [openLineId, setOpenLineId] = useState<string | null>(
		values.lines[0]?.clientId ?? null,
	)
	const photoPreviews = useFileUrls(
		values.lines.flatMap((line) => line.photoKeys),
	)
	const photoUrls = Object.fromEntries(
		photoPreviews.map((preview) => [preview.key, preview.url]),
	)

	return (
		<PageShell>
			<PageHeader
				eyebrow={quote.reference}
				title={quote.title}
				description="Visualisez et modifiez le contenu du devis."
				actions={
					<div className="flex flex-col gap-2 sm:flex-row">
						<Button asChild variant="outline">
							<Link to={buildOrgPath(activeOrganization.slug, '/crm/quotes')}>
								<ArrowLeft />
								Retour
							</Link>
						</Button>
						<AlertDialog>
							<AlertDialogTrigger asChild>
								<Button
									type="button"
									variant="destructive"
									disabled={isDeleting || isSaving}
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
							disabled={!canSave || isSaving || isDeleting}
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
							description="Objet, client et adresse de facturation du devis."
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
							<FieldBlock label="Adresse de facturation">
								<BillingAddressField
									value={values.customerContextId}
									addresses={customerContexts}
									hasCustomer={Boolean(values.customerId)}
									isLoading={isLoading}
									onChange={(customerContextId) =>
										onChange({ customerContextId })
									}
								/>
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
									photos={line.photoKeys.map((key) => ({
										key,
										url: photoUrls[key],
									}))}
									isOpen={openLineId === line.clientId}
									canRemove={values.lines.length > 1}
									isUploading={isUploading}
									onOpenChange={(open) =>
										setOpenLineId(open ? line.clientId : null)
									}
									onChange={(patch) => onLineChange(index, patch)}
									onSelectCatalogItem={(catalogItemId) =>
										onSelectCatalogItem(index, catalogItemId)
									}
									onRemove={() => onRemoveLine(index)}
									onUploadPhoto={(file) => void onUploadLinePhoto(index, file)}
									onRemovePhoto={(key) =>
										onLineChange(index, {
											photoKeys: line.photoKeys.filter(
												(photoKey) => photoKey !== key,
											),
										})
									}
								/>
							))}
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
							value={formatCents(totalCents)}
							strong
						/>
					</div>
				</aside>
			</div>
		</PageShell>
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
			notes: line.notes ?? '',
			photoKeys: line.photo_keys,
		}
	})

	return {
		title: quote.title,
		customerId: quote.customer_id,
		customerContextId: quote.customer_context_id,
		lines: lines.length ? lines : [emptyQuoteLine()],
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
	const { activeOrganization } = useActiveOrganization()

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
						<Link to={buildOrgPath(activeOrganization.slug, '/crm/quotes')}>
							Retour aux devis
						</Link>
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

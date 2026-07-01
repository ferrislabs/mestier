import { Link, useNavigate } from '@tanstack/react-router'
import {
	AlertCircle,
	Copy,
	Download,
	Eye,
	FileText,
	ImagePlus,
	Loader2,
	MoreHorizontal,
	Plus,
	SendHorizonal,
	Trash2,
	Zap,
} from 'lucide-react'
import type * as React from 'react'
import { useEffect, useMemo, useState } from 'react'
import { DocumentPreview } from '#/components/document-preview'
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
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
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
	Sheet,
	SheetContent,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { Textarea } from '#/components/ui/textarea'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useBillingSettings } from '#/hooks/use-billing-settings'
import { type CatalogItem, useCatalogItems } from '#/hooks/use-catalog-items'
import {
	type Customer,
	type CustomerContext,
	useCustomerContexts,
	useCustomers,
	useUploadFile,
} from '#/hooks/use-customers'
import {
	type Invoice,
	type InvoiceStatus,
	useCreateBalanceInvoice,
	useCreateDepositInvoice,
	useDeleteInvoice,
	useInvoice,
	useIssueInvoice,
	useShareInvoice,
	useUpdateInvoice,
	useUpdateInvoiceStatus,
} from '#/hooks/use-invoices'
import {
	type LegalMentionTemplate,
	useLegalMentionTemplates,
} from '#/hooks/use-legal-mentions'
import {
	type ServiceRateUnit,
	useReferenceCatalog,
} from '#/hooks/use-reference-catalog'
import { buildShareUrl, openInvoicePdf } from '#/lib/document-download'
import {
	centsToEuros,
	customerContextDisplayName,
	customerDisplayName,
	emptyInvoiceLine,
	eurosToCents,
	formatCents,
	formatUnit,
	type InvoiceFormValues,
	type InvoiceLineFormValues,
	invoiceStatusLabel,
} from '#/pages/invoices/types'

interface InvoiceEditFeatureProps {
	invoiceId: string
}

export function InvoiceEditFeature({ invoiceId }: InvoiceEditFeatureProps) {
	const invoice = useInvoice(invoiceId)

	if (invoice.isLoading) return <InvoiceEditLoading />

	if (invoice.isError) {
		return (
			<InvoiceEditError
				title="Impossible de charger la facture"
				message={invoice.error.message}
				onRetry={() => void invoice.refetch()}
			/>
		)
	}

	if (!invoice.data?.data) {
		return (
			<InvoiceEditError
				title="Facture introuvable"
				message="Aucune facture ne correspond à cet identifiant."
			/>
		)
	}

	return <InvoiceEditWorkspace invoice={invoice.data.data} />
}

function InvoiceEditWorkspace({ invoice }: { invoice: Invoice }) {
	const navigate = useNavigate()
	const { activeOrganization } = useActiveOrganization()
	const customers = useCustomers(invoice.org_id)
	const legalMentionTemplates = useLegalMentionTemplates(invoice.org_id)
	const billingSettings = useBillingSettings(invoice.org_id)
	const catalog = useReferenceCatalog(invoice.org_id, {
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
	const updateInvoice = useUpdateInvoice(invoice.id)
	const updateInvoiceStatus = useUpdateInvoiceStatus(invoice.id)
	const deleteInvoice = useDeleteInvoice(invoice.org_id)
	const createDepositInvoice = useCreateDepositInvoice()
	const createBalanceInvoice = useCreateBalanceInvoice()
	const issueInvoice = useIssueInvoice(invoice.id)
	const shareInvoice = useShareInvoice()
	const uploadFile = useUploadFile()
	const [values, setValues] = useState<InvoiceFormValues>(() =>
		invoiceToForm(invoice, catalogItems),
	)
	const [depositDialogOpen, setDepositDialogOpen] = useState(false)
	const [depositBasis, setDepositBasis] = useState('PERCENT')
	const [depositValue, setDepositValueState] = useState('')
	const [previewOpen, setPreviewOpen] = useState(false)
	const [shareLabel, setShareLabel] = useState<string | null>(null)
	const [pdfError, setPdfError] = useState<string | null>(null)
	const customerContexts = useCustomerContexts(
		values.customerId,
		Boolean(values.customerId),
	)

	useEffect(() => {
		setValues(invoiceToForm(invoice, catalogItems))
	}, [catalogItems, invoice])

	const isDraft = invoice.status === 'DRAFT'
	const isIssued = Boolean(invoice.reference) && invoice.status !== 'DRAFT'

	const updateValues = (patch: Partial<InvoiceFormValues>) => {
		setValues((current) => ({ ...current, ...patch }))
	}

	const updateLine = (index: number, patch: Partial<InvoiceLineFormValues>) => {
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
				emptyInvoiceLine(`line-${Date.now()}-${current.lines.length}`),
			],
		}))
	}

	const removeLine = (index: number) => {
		setValues((current) => {
			const lines = current.lines.filter(
				(_line, lineIndex) => lineIndex !== index,
			)
			return {
				...current,
				lines: lines.length ? lines : [emptyInvoiceLine()],
			}
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
		isDraft &&
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
		updateInvoice.error?.message ??
		updateInvoiceStatus.error?.message ??
		deleteInvoice.error?.message ??
		createDepositInvoice.error?.message ??
		createBalanceInvoice.error?.message ??
		issueInvoice.error?.message ??
		shareInvoice.error?.message ??
		pdfError ??
		uploadFile.error?.message ??
		null

	const saveInvoice = async () => {
		if (!canSave) return
		await updateInvoice.mutateAsync({
			path: { invoice_id: invoice.id },
			body: {
				title: values.title.trim(),
				customer_id: values.customerId,
				customer_context_id: values.customerContextId,
				legal_mention_template_ids: values.legalMentionTemplateIds,
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

	const markAsSent = async () => {
		await updateInvoiceStatus.mutateAsync({
			path: { invoice_id: invoice.id },
			body: { status: 'SENT' },
		})
	}

	const deleteCurrentInvoice = async () => {
		await deleteInvoice.mutateAsync({
			path: { invoice_id: invoice.id },
		})
		await navigate({ to: '/invoices' })
	}

	const createDeposit = async () => {
		if (!depositValue.trim()) return
		const result = await createDepositInvoice.mutateAsync({
			path: { invoice_id: invoice.id },
			body: { basis: depositBasis, value: depositValue.trim() },
		})
		setDepositDialogOpen(false)
		setDepositValueState('')
		await navigate({
			to: '/invoices/$invoiceId',
			params: { invoiceId: result.data.id },
		})
	}

	const createBalance = async () => {
		const result = await createBalanceInvoice.mutateAsync({
			path: { invoice_id: invoice.id },
		})
		await navigate({
			to: '/invoices/$invoiceId',
			params: { invoiceId: result.data.id },
		})
	}

	const issueCurrentInvoice = async () => {
		await issueInvoice.mutateAsync({
			path: { invoice_id: invoice.id },
		})
	}

	const downloadPdf = async () => {
		setPdfError(null)
		try {
			await openInvoicePdf(invoice.id)
		} catch (err) {
			setPdfError(
				err instanceof Error
					? err.message
					: 'Erreur lors du téléchargement du PDF',
			)
		}
	}

	const shareLinkAndCopy = async () => {
		setShareLabel(null)
		const result = await shareInvoice.mutateAsync({
			path: { invoice_id: invoice.id },
		})
		const url = buildShareUrl(result.data.path)
		try {
			await navigator.clipboard.writeText(url)
			setShareLabel('Lien copié !')
			setTimeout(() => setShareLabel(null), 3000)
		} catch {
			setShareLabel(url)
		}
	}

	const currentCustomer = customers.data?.data.find(
		(c) => c.id === invoice.customer_id,
	)
	const customerName = currentCustomer
		? `${currentCustomer.first_name} ${currentCustomer.last_name}`
		: ''

	return (
		<>
			<Sheet open={previewOpen} onOpenChange={setPreviewOpen}>
				<SheetContent
					side="right"
					className="w-full overflow-y-auto sm:max-w-2xl"
				>
					<SheetHeader>
						<SheetTitle>Aperçu du document</SheetTitle>
					</SheetHeader>
					<div className="p-4">
						<DocumentPreview
							invoice={invoice}
							billingSettings={billingSettings.data ?? null}
							templates={legalMentionTemplates.data?.data ?? []}
							customerName={customerName}
							sellerName={activeOrganization?.name ?? ''}
						/>
					</div>
				</SheetContent>
			</Sheet>

			<InvoiceEditUI
				invoice={invoice}
				values={values}
				customers={customers.data?.data ?? []}
				legalMentionTemplates={legalMentionTemplates.data?.data ?? []}
				isLegalMentionTemplatesLoading={legalMentionTemplates.isLoading}
				customerContexts={customerContexts.data?.data ?? []}
				catalogItems={catalogItems}
				error={error}
				isDraft={isDraft}
				isIssued={isIssued}
				isLoading={
					customers.isLoading ||
					customerContexts.isLoading ||
					catalog.serviceRates.isLoading ||
					catalog.products.isLoading
				}
				isSaving={updateInvoice.isPending}
				isSendingStatus={updateInvoiceStatus.isPending}
				isDeleting={deleteInvoice.isPending}
				isCreatingDeposit={createDepositInvoice.isPending}
				isCreatingBalance={createBalanceInvoice.isPending}
				isIssuing={issueInvoice.isPending}
				isSharing={shareInvoice.isPending}
				isUploading={uploadFile.isPending}
				shareLabel={shareLabel}
				canSave={canSave}
				depositDialogOpen={depositDialogOpen}
				depositBasis={depositBasis}
				depositValue={depositValue}
				onChange={(patch) => {
					if (patch.customerId !== undefined) {
						updateValues({
							customerId: patch.customerId,
							customerContextId: '',
						})
						return
					}
					updateValues(patch)
				}}
				onLineChange={updateLine}
				onSelectCatalogItem={selectCatalogItem}
				onAddLine={addLine}
				onRemoveLine={removeLine}
				onUploadLinePhoto={uploadLinePhoto}
				onSave={saveInvoice}
				onMarkAsSent={markAsSent}
				onDelete={deleteCurrentInvoice}
				onOpenDepositDialog={() => setDepositDialogOpen(true)}
				onCloseDepositDialog={() => {
					setDepositDialogOpen(false)
					setDepositValueState('')
				}}
				onDepositBasisChange={setDepositBasis}
				onDepositValueChange={setDepositValueState}
				onCreateDeposit={() => void createDeposit()}
				onCreateBalance={() => void createBalance()}
				onOpenPreview={() => setPreviewOpen(true)}
				onIssue={() => void issueCurrentInvoice()}
				onDownloadPdf={() => void downloadPdf()}
				onShareLink={() => void shareLinkAndCopy()}
			/>
		</>
	)
}

function InvoiceEditUI({
	invoice,
	values,
	customers,
	legalMentionTemplates,
	isLegalMentionTemplatesLoading,
	customerContexts,
	catalogItems,
	error,
	isDraft,
	isIssued,
	isLoading,
	isSaving,
	isSendingStatus,
	isDeleting,
	isCreatingDeposit,
	isCreatingBalance,
	isIssuing,
	isSharing,
	isUploading,
	shareLabel,
	canSave,
	depositDialogOpen,
	depositBasis,
	depositValue,
	onChange,
	onLineChange,
	onSelectCatalogItem,
	onAddLine,
	onRemoveLine,
	onUploadLinePhoto,
	onSave,
	onMarkAsSent,
	onDelete,
	onOpenDepositDialog,
	onCloseDepositDialog,
	onDepositBasisChange,
	onDepositValueChange,
	onCreateDeposit,
	onCreateBalance,
	onOpenPreview,
	onIssue,
	onDownloadPdf,
	onShareLink,
}: {
	invoice: Invoice
	values: InvoiceFormValues
	customers: Customer[]
	legalMentionTemplates: LegalMentionTemplate[]
	isLegalMentionTemplatesLoading: boolean
	customerContexts: CustomerContext[]
	catalogItems: CatalogItem[]
	error: string | null
	isDraft: boolean
	isIssued: boolean
	isLoading: boolean
	isSaving: boolean
	isSendingStatus: boolean
	isDeleting: boolean
	isCreatingDeposit: boolean
	isCreatingBalance: boolean
	isIssuing: boolean
	isSharing: boolean
	isUploading: boolean
	shareLabel: string | null
	canSave: boolean
	depositDialogOpen: boolean
	depositBasis: string
	depositValue: string
	onChange: (patch: Partial<InvoiceFormValues>) => void
	onLineChange: (index: number, patch: Partial<InvoiceLineFormValues>) => void
	onSelectCatalogItem: (index: number, catalogItemId: string) => void
	onAddLine: () => void
	onRemoveLine: (index: number) => void
	onUploadLinePhoto: (index: number, file: File) => Promise<void>
	onSave: () => void
	onMarkAsSent: () => void
	onDelete: () => void
	onOpenDepositDialog: () => void
	onCloseDepositDialog: () => void
	onDepositBasisChange: (basis: string) => void
	onDepositValueChange: (value: string) => void
	onCreateDeposit: () => void
	onCreateBalance: () => void
	onOpenPreview: () => void
	onIssue: () => void
	onDownloadPdf: () => void
	onShareLink: () => void
}) {
	const isStandard = invoice.invoice_type === 'STANDARD'
	const anyPending =
		isDeleting ||
		isSaving ||
		isSendingStatus ||
		isCreatingDeposit ||
		isCreatingBalance ||
		isIssuing ||
		isSharing

	const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)

	// Determine whether the "…" menu has any applicable items
	const hasMoreMenu = isStandard || isDraft

	return (
		<PageShell>
			<PageHeader
				eyebrow={invoice.reference ?? 'Brouillon'}
				title={invoice.title}
				description="Visualisez et modifiez le contenu de la facture."
				actions={
					<div className="flex flex-wrap items-center justify-end gap-2">
						{/* Aperçu — always available */}
						<Button type="button" variant="outline" onClick={onOpenPreview}>
							<Eye />
							Aperçu
						</Button>

						{/* DRAFT primary actions */}
						{isDraft ? (
							<>
								{/* Issue / emit invoice — DRAFT only, irreversible */}
								<AlertDialog>
									<AlertDialogTrigger asChild>
										<Button
											type="button"
											variant="default"
											disabled={anyPending}
										>
											{isIssuing ? (
												<Loader2 className="animate-spin" />
											) : (
												<Zap />
											)}
											Émettre / Envoyer au client
										</Button>
									</AlertDialogTrigger>
									<AlertDialogContent>
										<AlertDialogHeader>
											<AlertDialogTitle>
												Émettre cette facture ?
											</AlertDialogTitle>
											<AlertDialogDescription>
												Un PDF sera généré et la facture passera en statut «
												Envoyée ». Cette action est irréversible : la facture ne
												pourra plus être modifiée.
											</AlertDialogDescription>
										</AlertDialogHeader>
										<AlertDialogFooter>
											<AlertDialogCancel disabled={isIssuing}>
												Annuler
											</AlertDialogCancel>
											<AlertDialogAction
												disabled={isIssuing}
												onClick={(event) => {
													event.preventDefault()
													void onIssue()
												}}
											>
												{isIssuing ? (
													<Loader2 className="animate-spin" />
												) : (
													<Zap />
												)}
												Émettre
											</AlertDialogAction>
										</AlertDialogFooter>
									</AlertDialogContent>
								</AlertDialog>

								<Button
									type="button"
									disabled={
										!canSave || isSaving || isDeleting || isSendingStatus
									}
									onClick={onSave}
								>
									{isSaving ? (
										<Loader2 className="animate-spin" />
									) : (
										<FileText />
									)}
									Enregistrer
								</Button>
							</>
						) : null}

						{/* Post-issue actions — PDF download + share */}
						{isIssued ? (
							<>
								<Button
									type="button"
									variant="outline"
									disabled={anyPending}
									onClick={onDownloadPdf}
								>
									<Download />
									Télécharger le PDF
								</Button>
								<Button
									type="button"
									variant="outline"
									disabled={anyPending || isSharing}
									onClick={onShareLink}
								>
									{isSharing ? <Loader2 className="animate-spin" /> : <Copy />}
									{shareLabel ?? 'Partager le lien'}
								</Button>
							</>
						) : null}

						{/* "…" overflow menu — secondary actions */}
						{hasMoreMenu ? (
							<DropdownMenu>
								<DropdownMenuTrigger asChild>
									<Button variant="outline" size="icon">
										<MoreHorizontal />
										<span className="sr-only">Plus d'actions</span>
									</Button>
								</DropdownMenuTrigger>
								<DropdownMenuContent align="end">
									{isStandard ? (
										<>
											<DropdownMenuItem
												disabled={anyPending}
												onSelect={onOpenDepositDialog}
											>
												{isCreatingDeposit ? (
													<Loader2 className="animate-spin" />
												) : (
													<Plus />
												)}
												Facture d'acompte
											</DropdownMenuItem>
											<DropdownMenuItem
												disabled={anyPending}
												onSelect={onCreateBalance}
											>
												{isCreatingBalance ? (
													<Loader2 className="animate-spin" />
												) : (
													<FileText />
												)}
												Facture de solde
											</DropdownMenuItem>
										</>
									) : null}
									{isDraft ? (
										<>
											<DropdownMenuItem
												disabled={isSendingStatus || isSaving || isDeleting}
												onSelect={onMarkAsSent}
											>
												{isSendingStatus ? (
													<Loader2 className="animate-spin" />
												) : (
													<SendHorizonal />
												)}
												Marquer comme envoyée
											</DropdownMenuItem>
											<DropdownMenuSeparator />
											<DropdownMenuItem
												variant="destructive"
												disabled={anyPending}
												onSelect={(e) => {
													e.preventDefault()
													setDeleteDialogOpen(true)
												}}
											>
												{isDeleting ? (
													<Loader2 className="animate-spin" />
												) : (
													<Trash2 />
												)}
												Supprimer
											</DropdownMenuItem>
										</>
									) : null}
								</DropdownMenuContent>
							</DropdownMenu>
						) : null}
					</div>
				}
			/>

			{/* Controlled delete confirmation dialog — rendered outside the dropdown */}
			<AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>Supprimer cette facture ?</AlertDialogTitle>
						<AlertDialogDescription>
							La facture brouillon sera supprimée de la liste. Cette action est
							irréversible.
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel disabled={isDeleting}>Annuler</AlertDialogCancel>
						<AlertDialogAction
							disabled={isDeleting}
							className="bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20"
							onClick={(event) => {
								event.preventDefault()
								void onDelete()
							}}
						>
							{isDeleting ? <Loader2 className="animate-spin" /> : <Trash2 />}
							Supprimer
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>

			{depositDialogOpen ? (
				<div className="rounded-lg border bg-card p-5 shadow-sm">
					<p className="mb-4 text-sm font-semibold">Facture d'acompte</p>
					<div className="grid gap-4 md:grid-cols-3">
						<div className="flex flex-col gap-2">
							<Label>Type</Label>
							<Select value={depositBasis} onValueChange={onDepositBasisChange}>
								<SelectTrigger className="w-full">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value="PERCENT">Pourcentage</SelectItem>
									<SelectItem value="FIXED">Montant fixe</SelectItem>
								</SelectContent>
							</Select>
						</div>
						<div className="flex flex-col gap-2">
							<Label>
								{depositBasis === 'PERCENT' ? 'Pourcentage (%)' : 'Montant (€)'}
							</Label>
							<div className="relative">
								<Input
									inputMode="decimal"
									value={depositValue}
									onChange={(event) => onDepositValueChange(event.target.value)}
									placeholder={
										depositBasis === 'PERCENT' ? 'Ex. 30' : 'Ex. 300'
									}
									className="pr-6"
								/>
								<span className="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-xs text-muted-foreground">
									{depositBasis === 'PERCENT' ? '%' : '€'}
								</span>
							</div>
						</div>
						<div className="flex items-end gap-2">
							<Button
								type="button"
								disabled={!depositValue.trim() || isCreatingDeposit}
								onClick={onCreateDeposit}
							>
								{isCreatingDeposit ? (
									<Loader2 className="animate-spin" />
								) : (
									<FileText />
								)}
								Créer
							</Button>
							<Button
								type="button"
								variant="outline"
								onClick={onCloseDepositDialog}
							>
								Annuler
							</Button>
						</div>
					</div>
				</div>
			) : null}

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
							description="Objet, client et contexte associés à la facture."
						/>
						<div className="grid gap-4 p-5 md:grid-cols-2">
							<FieldBlock label="Objet de la facture">
								<Input
									value={values.title}
									onChange={(event) => onChange({ title: event.target.value })}
									placeholder="Ex. Rénovation salle de bain"
									disabled={!isDraft}
								/>
							</FieldBlock>
							<FieldBlock label="Statut">
								<div className="flex h-10 items-center">
									<StatusBadge tone={invoiceStatusTone(invoice.status)}>
										{invoiceStatusLabel(invoice.status)}
									</StatusBadge>
								</div>
							</FieldBlock>
							<FieldBlock label="Client">
								<Select
									value={values.customerId}
									onValueChange={(customerId) => onChange({ customerId })}
									disabled={!isDraft || isLoading}
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
									disabled={!isDraft || !values.customerId || isLoading}
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
								isDraft ? (
									<Button
										type="button"
										variant="outline"
										size="sm"
										onClick={onAddLine}
									>
										<Plus />
										Ajouter
									</Button>
								) : null
							}
						/>
						<div className="divide-y">
							{values.lines.map((line, index) => (
								<InvoiceLineEditor
									key={line.clientId}
									index={index}
									line={line}
									catalogItems={catalogItems}
									canRemove={isDraft && values.lines.length > 1}
									isUploading={isUploading}
									readOnly={!isDraft}
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
							title="Mentions légales"
							description="Sélectionnez les mentions légales à inclure dans cette facture."
						/>
						<div className="p-5">
							<LegalMentionSelector
								templates={legalMentionTemplates}
								selectedIds={values.legalMentionTemplateIds}
								onChange={
									isDraft
										? (ids) => onChange({ legalMentionTemplateIds: ids })
										: () => {}
								}
								isLoading={isLegalMentionTemplatesLoading}
							/>
						</div>
					</SectionCard>
				</div>

				<aside className="h-fit rounded-lg border bg-card p-5 shadow-sm xl:sticky xl:top-5">
					<p className="text-sm font-semibold">Résumé</p>
					<div className="mt-4 space-y-3 text-sm">
						<SummaryRow
							label="Référence"
							value={invoice.reference ?? 'Brouillon'}
						/>
						<SummaryRow
							label="Statut"
							value={invoiceStatusLabel(invoice.status)}
						/>
						<div className="flex items-center justify-between gap-4">
							<span className="text-muted-foreground">Type</span>
							<StatusBadge tone={invoiceTypeTone(invoice.invoice_type)}>
								{invoiceTypeLabel(invoice.invoice_type)}
							</StatusBadge>
						</div>
						<SummaryRow label="Objet" value={values.title || 'Non renseigné'} />
						<SummaryRow label="Lignes" value={String(values.lines.length)} />
						<SummaryRow
							label="Total HT"
							value={formatCents(invoice.total_ht_cents)}
						/>
						<SummaryRow
							label="TVA"
							value={formatCents(invoice.total_vat_cents)}
						/>
						<SummaryRow
							label="Total TTC"
							value={formatCents(invoice.total_ttc_cents)}
							strong
						/>
						{invoice.invoice_type === 'BALANCE' &&
						invoice.deposit_amount_cents != null ? (
							<SummaryRow
								label="Acompte déduit"
								value={formatCents(invoice.deposit_amount_cents)}
							/>
						) : null}
						{invoice.invoice_type === 'STANDARD' ? (
							<SummaryRow
								label="Reste à payer"
								value={formatCents(
									invoice.total_ttc_cents - invoice.amount_paid_cents,
								)}
							/>
						) : null}
						{invoice.invoice_type === 'BALANCE' ? (
							<SummaryRow
								label="Reste à payer"
								value={formatCents(invoice.total_ttc_cents)}
								strong
							/>
						) : null}
					</div>
				</aside>
			</div>
		</PageShell>
	)
}

function InvoiceLineEditor({
	index,
	line,
	catalogItems,
	canRemove,
	isUploading,
	readOnly,
	onChange,
	onSelectCatalogItem,
	onRemove,
	onUploadPhoto,
}: {
	index: number
	line: InvoiceLineFormValues
	catalogItems: CatalogItem[]
	canRemove: boolean
	isUploading: boolean
	readOnly: boolean
	onChange: (patch: Partial<InvoiceLineFormValues>) => void
	onSelectCatalogItem: (catalogItemId: string) => void
	onRemove: () => void
	onUploadPhoto: (file: File) => Promise<void>
}) {
	const serviceItems = catalogItems.filter((item) => item.type === 'SERVICE')
	const productItems = catalogItems.filter((item) => item.type === 'PRODUCT')
	const selectedCatalogItem = catalogItems.find(
		(item) => item.id === line.catalogItemId,
	)
	const lineTotalCents = invoiceLineTotalCents(line)

	return (
		<div className="bg-card p-4">
			<div className="mb-4 flex items-center justify-between gap-3">
				<div>
					<p className="text-sm font-semibold">
						{line.label.trim() || `Ligne ${index + 1}`}
					</p>
					<p className="text-xs text-muted-foreground">
						{lineSourceLabel(line.catalogItemType)}
					</p>
				</div>
				<div className="flex items-center gap-3">
					<p className="text-sm font-semibold">{formatCents(lineTotalCents)}</p>
					{!readOnly ? (
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
					) : null}
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
							disabled={readOnly}
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
						disabled={readOnly}
					/>
				</FieldBlock>
				<FieldBlock label="Quantité">
					<Input
						inputMode="decimal"
						value={line.quantity}
						onChange={(event) => onChange({ quantity: event.target.value })}
						disabled={readOnly}
					/>
				</FieldBlock>
				<FieldBlock label="Prix unitaire">
					<Input
						inputMode="decimal"
						value={line.unitPrice}
						onChange={(event) => onChange({ unitPrice: event.target.value })}
						disabled={readOnly}
					/>
				</FieldBlock>
				<FieldBlock label="TVA">
					<div className="relative">
						<Input
							inputMode="decimal"
							value={line.vatRate}
							onChange={(event) => onChange({ vatRate: event.target.value })}
							className="pr-6"
							disabled={readOnly}
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
						disabled={readOnly}
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
							disabled={readOnly}
						/>
					</FieldBlock>
				</div>
				{!readOnly ? (
					<FieldBlock label="Photos">
						<label className="flex h-10 cursor-pointer items-center justify-center gap-2 rounded-lg border bg-card px-3 text-sm font-medium text-primary shadow-xs hover:bg-brand-soft">
							<ImagePlus className="size-4" />
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
				) : null}
			</div>
		</div>
	)
}

function invoiceToForm(
	invoice: Invoice,
	catalogItems: CatalogItem[],
): InvoiceFormValues {
	const lines: InvoiceLineFormValues[] = invoice.lines.map((line, index) => {
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
		title: invoice.title,
		customerId: invoice.customer_id,
		customerContextId: invoice.customer_context_id,
		legalMentionTemplateIds: invoice.legal_mention_template_ids ?? [],
		lines: lines.length ? lines : [emptyInvoiceLine()],
	}
}

function InvoiceEditLoading() {
	return (
		<PageShell>
			<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
				<Loader2 className="size-5 animate-spin" />
				Chargement de la facture…
			</SectionCard>
		</PageShell>
	)
}

function InvoiceEditError({
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
						<Link to="/invoices">Retour aux factures</Link>
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

function invoiceLineTotalCents(line: InvoiceLineFormValues): number {
	const quantity = Number(line.quantity.replace(',', '.'))
	if (!Number.isFinite(quantity) || quantity <= 0) return 0
	return Math.round(quantity * eurosToCents(line.unitPrice))
}

function lineSourceLabel(type: InvoiceLineFormValues['catalogItemType']) {
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

function invoiceStatusTone(status: InvoiceStatus) {
	if (status === 'PAID') return 'success' as const
	if (status === 'SENT' || status === 'PARTIALLY_PAID') return 'brand' as const
	if (status === 'CANCELLED') return 'error' as const
	return 'neutral' as const
}

function invoiceTypeLabel(type: string) {
	if (type === 'DEPOSIT') return 'Acompte'
	if (type === 'BALANCE') return 'Solde'
	return 'Standard'
}

function invoiceTypeTone(type: string) {
	if (type === 'DEPOSIT') return 'brand' as const
	if (type === 'BALANCE') return 'success' as const
	return 'neutral' as const
}

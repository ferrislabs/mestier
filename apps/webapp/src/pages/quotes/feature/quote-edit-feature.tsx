import { Link, useNavigate } from '@tanstack/react-router'
import {
	AlertCircle,
	ArrowLeft,
	ArrowRightLeft,
	FileText,
	Loader2,
	Send,
	Trash2,
} from 'lucide-react'
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
import { Field } from '#/components/ui/field'
import { Input } from '#/components/ui/input'
import {
	PageHeader,
	PageShell,
	SectionCard,
	StatusBadge,
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
	billingAddressLines,
	centsToEuros,
	customerDisplayName,
	emptyQuoteLine,
	eurosToCents,
	formatCents,
	formatDate,
	type QuoteFormValues,
	type QuoteLineFormValues,
	quoteLineTotalCents,
	quoteReferenceLabel,
	quoteStatusLabel,
} from '#/pages/quotes/types'
import { EditablePaperField } from '#/pages/quotes/ui/editable-paper-field'
import { PaperOptionList } from '#/pages/quotes/ui/paper-option-list'
import {
	QuoteIssuerDetails,
	QuoteIssuerMark,
} from '#/pages/quotes/ui/quote-issuer-block'
import { QuoteLinesTable } from '#/pages/quotes/ui/quote-lines-table'
import { QuoteTotalsFooter } from '#/pages/quotes/ui/quote-totals-footer'
import { LEGAL_IDENTITY_FIELD_LABELS } from '#/pages/settings/types'

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
	const missingLegalIdentityFields =
		activeOrganization.missing_legal_identity_fields
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
			vatRateBp:
				catalogItem.defaultVatRateBp !== null
					? String(catalogItem.defaultVatRateBp)
					: values.lines[index]?.vatRateBp || '',
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

	const hasValidContent =
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

	// Sending is refused server-side when the organization's legal identity
	// is incomplete (#310, enforced again on export by #314) — caught here
	// too, independently of whatever status happens to be locally selected,
	// so the warning and the disabled "Envoyer" button show up before a
	// request is even attempted.
	const identityIncompleteForSending =
		!quote.reference && missingLegalIdentityFields.length > 0
	const blockedBySentIdentity =
		status === 'SENT' && identityIncompleteForSending

	const canSave = hasValidContent && !blockedBySentIdentity
	const canSend = hasValidContent && !identityIncompleteForSending

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

	const buildUpdateBody = (targetStatus: QuoteStatus) => ({
		title: values.title.trim(),
		customer_id: values.customerId,
		customer_context_id: values.customerContextId,
		status: targetStatus,
		lines: values.lines.map((line) => ({
			service_rate_id: line.serviceRateId || null,
			label: line.label.trim(),
			quantity: line.quantity.replace(',', '.').trim(),
			unit: line.unit,
			unit_price_cents: eurosToCents(line.unitPrice),
			vat_rate_bp: line.vatRateBp === '' ? null : Number(line.vatRateBp),
			notes: line.notes.trim() || null,
			photo_keys: line.photoKeys,
		})),
	})

	const saveQuote = async () => {
		if (!canSave) return
		await updateQuote.mutateAsync({
			path: { quote_id: quote.id },
			body: buildUpdateBody(status),
		})
	}

	// Built on the literal `'SENT'` rather than on `setStatus` + `saveQuote`:
	// `setStatus` only lands on the *next* render, so a save fired right
	// after it would still read the old `status` from this render's closure.
	const sendQuote = async () => {
		if (!canSend) return
		setStatus('SENT')
		await updateQuote.mutateAsync({
			path: { quote_id: quote.id },
			body: buildUpdateBody('SENT'),
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
			canSend={canSend}
			identityIncompleteForSending={identityIncompleteForSending}
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
			onSend={sendQuote}
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
	canSend,
	identityIncompleteForSending,
	onChange,
	onStatusChange,
	onLineChange,
	onSelectCatalogItem,
	onAddLine,
	onRemoveLine,
	onUploadLinePhoto,
	onSave,
	onSend,
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
	canSend: boolean
	identityIncompleteForSending: boolean
	onChange: (patch: Partial<QuoteFormValues>) => void
	onStatusChange: (status: QuoteStatus) => void
	onLineChange: (index: number, patch: Partial<QuoteLineFormValues>) => void
	onSelectCatalogItem: (index: number, catalogItemId: string) => void
	onAddLine: () => void
	onRemoveLine: (index: number) => void
	onUploadLinePhoto: (index: number, file: File) => Promise<void>
	onSave: () => void
	onSend: () => void
	onDelete: () => void
}) {
	const { activeOrganization } = useActiveOrganization()
	const vatEnabled = activeOrganization.vat_status?.type === 'subject'
	const missingLegalIdentityFields =
		activeOrganization.missing_legal_identity_fields
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
				eyebrow={quoteReferenceLabel(quote.reference)}
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
						{quote.status === 'ACCEPTED' ? (
							<Button asChild variant="outline">
								<Link
									to={buildOrgPath(
										activeOrganization.slug,
										`/crm/quotes/${quote.id}/handover`,
									)}
								>
									<ArrowRightLeft />
									Transformer en projet
								</Link>
							</Button>
						) : null}
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
										Le devis {quoteReferenceLabel(quote.reference)} sera
										supprimé de la liste. Cette action est irréversible.
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
							disabled={!canSave || isSaving || isDeleting}
							onClick={onSave}
						>
							{isSaving ? <Loader2 className="animate-spin" /> : <FileText />}
							Enregistrer
						</Button>
						{quote.status === 'DRAFT' ? (
							<Button
								type="button"
								disabled={!canSend || isSaving || isDeleting}
								onClick={onSend}
							>
								{isSaving ? <Loader2 className="animate-spin" /> : <Send />}
								Envoyer le devis
							</Button>
						) : null}
					</div>
				}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			{identityIncompleteForSending ? (
				<div className="flex items-start gap-3 rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-3 text-sm text-amber-800 dark:text-amber-300">
					<AlertCircle className="mt-0.5 size-4 shrink-0" />
					<p>
						Ce devis ne peut pas être envoyé : l'identité légale de
						l'organisation est incomplète. Il manque{' '}
						{missingLegalIdentityFields
							.map((field) => LEGAL_IDENTITY_FIELD_LABELS[field] ?? field)
							.join(', ')}
						.{' '}
						<a
							href={`${buildOrgPath(activeOrganization.slug, '/settings')}#organisation`}
							className="font-medium underline underline-offset-2"
						>
							Compléter dans les paramètres
						</a>
					</p>
				</div>
			) : null}

			<div className="mx-auto w-full max-w-4xl bg-muted/40 p-4 sm:p-10">
				<SectionCard className="border shadow-sm">
					<div className="border-b p-6 sm:p-8">
						<div className="flex items-start justify-between gap-4">
							<QuoteIssuerMark organization={activeOrganization} />
							<div className="flex flex-col items-end gap-2 text-right">
								<p className="text-sm font-semibold">
									Devis {quoteReferenceLabel(quote.reference)}
								</p>
								<p className="text-xs text-muted-foreground">
									{formatDate(quote.created_at)}
								</p>
								<EditablePaperField
									label="Statut"
									className="w-auto"
									renderEditor={(close) => (
										<div className="w-40">
											<PaperOptionList
												ariaLabel="Statut"
												value={status}
												options={QUOTE_STATUSES.map((value) => ({
													value,
													label: quoteStatusLabel(value),
												}))}
												onChange={(value) => {
													onStatusChange(value as QuoteStatus)
													close()
												}}
											/>
										</div>
									)}
								>
									<StatusBadge tone={statusTone(status)}>
										{quoteStatusLabel(status)}
									</StatusBadge>
								</EditablePaperField>
							</div>
						</div>

						<div className="mt-6 grid gap-8 md:grid-cols-2">
							<QuoteIssuerDetails organization={activeOrganization} />

							<EditablePaperField
								label="Facturé à"
								renderEditor={(close) => (
									<div className="space-y-4">
										<Field label="Client" htmlFor={null}>
											<PaperOptionList
												ariaLabel="Client"
												value={values.customerId}
												options={customers.map((customer) => ({
													value: customer.id,
													label: customerDisplayName(customer),
												}))}
												onChange={(customerId) => onChange({ customerId })}
												emptyLabel="Aucun client n’existe encore dans cette organisation."
											/>
										</Field>
										<Field label="Adresse de facturation" htmlFor={null}>
											{!values.customerId ? (
												<p className="border px-3 py-2 text-sm text-muted-foreground">
													Choisir un client d’abord
												</p>
											) : isLoading ? (
												<p className="border px-3 py-2 text-sm text-muted-foreground">
													Chargement…
												</p>
											) : (
												<PaperOptionList
													ariaLabel="Adresse de facturation"
													value={values.customerContextId}
													options={customerContexts.map((context) => ({
														value: context.id,
														label: context.label,
														description:
															billingAddressLines(context).join(' · ') ||
															'Adresse non renseignée',
													}))}
													onChange={(customerContextId) =>
														onChange({ customerContextId })
													}
													emptyLabel="Aucune adresse pour ce client."
												/>
											)}
										</Field>
										<Button
											type="button"
											variant="ghost"
											size="sm"
											onClick={close}
										>
											Fermer
										</Button>
									</div>
								)}
							>
								{(() => {
									const selectedCustomer = customers.find(
										(customer) => customer.id === values.customerId,
									)
									const selectedCustomerContext = customerContexts.find(
										(customerContext) =>
											customerContext.id === values.customerContextId,
									)
									if (!selectedCustomer) {
										return (
											<p className="text-sm text-muted-foreground italic">
												Sélectionner un client
											</p>
										)
									}
									return (
										<>
											<p className="font-semibold">
												{customerDisplayName(selectedCustomer)}
											</p>
											{selectedCustomerContext ? (
												billingAddressLines(selectedCustomerContext).map(
													(line) => (
														<p
															key={line}
															className="text-sm text-muted-foreground"
														>
															{line}
														</p>
													),
												)
											) : (
												<p className="text-sm text-warning">
													Adresse de facturation non renseignée
												</p>
											)}
											{[selectedCustomer.email, selectedCustomer.phone]
												.filter(Boolean)
												.join(' · ') ? (
												<p className="text-sm text-muted-foreground">
													{[selectedCustomer.email, selectedCustomer.phone]
														.filter(Boolean)
														.join(' · ')}
												</p>
											) : null}
											{selectedCustomer.registration_number ? (
												<p className="mt-1 text-xs text-muted-foreground">
													SIRET {selectedCustomer.registration_number}
												</p>
											) : null}
										</>
									)
								})()}
							</EditablePaperField>
						</div>

						<div className="mt-8">
							<EditablePaperField
								label="Objet du devis"
								renderEditor={(close) => (
									<div className="space-y-4">
										<Input
											autoFocus
											value={values.title}
											onChange={(event) =>
												onChange({ title: event.target.value })
											}
											placeholder="Ex. Rénovation salle de bain"
										/>
										<Button
											type="button"
											variant="ghost"
											size="sm"
											onClick={close}
										>
											Fermer
										</Button>
									</div>
								)}
							>
								<h1 className="text-2xl font-bold tracking-tight">
									{values.title || 'Objet du devis'}
								</h1>
							</EditablePaperField>
						</div>
					</div>

					<QuoteLinesTable
						lines={values.lines}
						catalogItems={catalogItems}
						photosByLine={Object.fromEntries(
							values.lines.map((line) => [
								line.clientId,
								line.photoKeys.map((key) => ({ key, url: photoUrls[key] })),
							]),
						)}
						isUploading={isUploading}
						openLineId={openLineId}
						vatEnabled={vatEnabled}
						onOpenLineChange={(clientId, open) =>
							setOpenLineId(open ? clientId : null)
						}
						onLineChange={(clientId, patch) => {
							const index = values.lines.findIndex(
								(line) => line.clientId === clientId,
							)
							if (index !== -1) onLineChange(index, patch)
						}}
						onSelectCatalogItem={(clientId, catalogItemId) => {
							const index = values.lines.findIndex(
								(line) => line.clientId === clientId,
							)
							if (index !== -1) onSelectCatalogItem(index, catalogItemId)
						}}
						onRemoveLine={(clientId) => {
							const index = values.lines.findIndex(
								(line) => line.clientId === clientId,
							)
							if (index !== -1) onRemoveLine(index)
						}}
						onAddLine={onAddLine}
						onUploadLinePhoto={(clientId, file) => {
							const index = values.lines.findIndex(
								(line) => line.clientId === clientId,
							)
							if (index !== -1) void onUploadLinePhoto(index, file)
						}}
						onRemoveLinePhoto={(clientId, key) => {
							const index = values.lines.findIndex(
								(line) => line.clientId === clientId,
							)
							const line = values.lines[index]
							if (index === -1 || !line) return
							onLineChange(index, {
								photoKeys: line.photoKeys.filter(
									(photoKey) => photoKey !== key,
								),
							})
						}}
					/>

					{/* The quote's actual totals — read from what was last saved, never
				    recomputed here (CLAUDE.md): the screen and the PDF must never be
				    able to disagree. `totalCents` (from the in-memory draft) only
				    surfaces as the small notice below when it disagrees with what
				    is saved. */}
					<QuoteTotalsFooter
						netCents={quote.net_cents}
						vatBreakdown={quote.vat_breakdown.map((line) => ({
							rateBp: line.rate_bp,
							vatCents: line.vat_cents,
						}))}
						grossCents={quote.gross_cents}
						vatExemptionNotice={
							quote.vat_breakdown.length === 0 &&
							activeOrganization.vat_status?.type === 'not_subject'
								? `TVA non applicable, ${activeOrganization.vat_status.basis}`
								: null
						}
						notice={
							totalCents !== quote.net_cents
								? `Aperçu non enregistré : ${formatCents(totalCents)}`
								: null
						}
					/>
				</SectionCard>
			</div>
		</PageShell>
	)
}

const QUOTE_STATUSES: QuoteStatus[] = [
	'DRAFT',
	'SENT',
	'ACCEPTED',
	'DECLINED',
	'CANCELLED',
]

function statusTone(status: QuoteStatus) {
	if (status === 'ACCEPTED') return 'success' as const
	if (status === 'SENT') return 'brand' as const
	if (status === 'DECLINED' || status === 'CANCELLED') return 'error' as const
	return 'neutral' as const
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
			vatRateBp: line.vat_rate_bp != null ? String(line.vat_rate_bp) : '',
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

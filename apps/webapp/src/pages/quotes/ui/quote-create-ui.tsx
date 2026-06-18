import {
	AlertCircle,
	FileText,
	ImagePlus,
	Plus,
	RefreshCw,
	Trash2,
} from 'lucide-react'
import { useMemo } from 'react'
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
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { Textarea } from '#/components/ui/textarea'
import type { Customer, Property } from '#/hooks/use-customers'
import type { Quote } from '#/hooks/use-quotes'
import type {
	ServiceRate,
	ServiceRateUnit,
} from '#/hooks/use-reference-catalog'
import {
	customerDisplayName,
	formatCents,
	formatDate,
	formatUnit,
	propertyDisplayName,
	type QuoteFormValues,
	type QuoteLineFormValues,
	quoteStatusLabel,
	serviceRateDisplayName,
} from '#/pages/quotes/types'

interface QuoteCreateUIProps {
	values: QuoteFormValues
	customers: Customer[]
	properties: Property[]
	serviceRates: ServiceRate[]
	quotes: Quote[]
	lastCreated: Quote | null
	error?: string | null
	isLoading?: boolean
	isCreating?: boolean
	isUploading?: boolean
	isPropertiesLoading?: boolean
	onRetry?: () => void
	onChange: (patch: Partial<QuoteFormValues>) => void
	onLineChange: (index: number, patch: Partial<QuoteLineFormValues>) => void
	onSelectServiceRate: (index: number, serviceRateId: string) => void
	onAddLine: () => void
	onRemoveLine: (index: number) => void
	onUploadLinePhoto: (index: number, file: File) => Promise<void>
	onSubmit: () => void
}

export function QuoteCreateUI({
	values,
	customers,
	properties,
	serviceRates,
	quotes,
	lastCreated,
	error,
	isLoading,
	isCreating,
	isUploading,
	isPropertiesLoading,
	onRetry,
	onChange,
	onLineChange,
	onSelectServiceRate,
	onAddLine,
	onRemoveLine,
	onUploadLinePhoto,
	onSubmit,
}: QuoteCreateUIProps) {
	const stats = useMemo(() => {
		return {
			total: quotes.length,
			draft: quotes.filter((quote) => quote.status === 'DRAFT').length,
			accepted: quotes.filter((quote) => quote.status === 'ACCEPTED').length,
			revenue: quotes
				.filter((quote) => quote.status === 'ACCEPTED')
				.reduce((sum, quote) => sum + quote.total_cents, 0),
		}
	}, [quotes])

	const selectedCustomer = customers.find((customer) => {
		return customer.id === values.customerId
	})

	const canSubmit =
		Boolean(values.customerId) &&
		Boolean(values.propertyId) &&
		values.lines.length > 0 &&
		values.lines.every((line) => {
			return (
				line.label.trim() &&
				line.quantity.trim() &&
				Number(line.quantity.replace(',', '.')) > 0 &&
				line.unitPrice.trim() &&
				Number(line.unitPrice.replace(',', '.')) >= 0
			)
		})

	return (
		<PageShell>
			<PageHeader
				title="Devis"
				description="Créez des devis détaillés à partir du catalogue de prestations et conservez le total renvoyé par l'API comme référence."
				actions={
					<Button
						type="button"
						variant="outline"
						onClick={onRetry}
						disabled={!onRetry}
					>
						<RefreshCw />
						Actualiser
					</Button>
				}
			/>

			{error ? (
				<SectionCard className="flex flex-col gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive sm:flex-row sm:items-center sm:justify-between">
					<div className="flex items-center gap-3">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{error}</p>
					</div>
					{onRetry ? (
						<Button onClick={onRetry} variant="outline" size="sm">
							Réessayer
						</Button>
					) : null}
				</SectionCard>
			) : null}

			<section>
				<p className="mb-3 text-sm text-muted-foreground">Suivi des devis</p>
				<div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
					<MetricCard
						label="Total devis"
						value={stats.total}
						hint="Tous statuts"
					/>
					<MetricCard
						label="Brouillons"
						value={stats.draft}
						hint="À finaliser"
					/>
					<MetricCard
						label="Acceptés"
						value={stats.accepted}
						hint="Validés client"
					/>
					<MetricCard
						label="Accepté HT"
						value={formatCents(stats.revenue)}
						hint="D'après l'API"
					/>
				</div>
			</section>

			<div className="grid gap-6 xl:grid-cols-[minmax(0,1.45fr)_minmax(360px,0.75fr)]">
				<SectionCard>
					<SectionHeader
						title="Nouveau devis"
						description="Les lignes sont envoyées à l'API, qui calcule le total sauvegardé."
						actions={
							<Button
								type="button"
								onClick={onSubmit}
								disabled={!canSubmit || isCreating}
							>
								<FileText />
								Créer
							</Button>
						}
					/>

					<div className="grid gap-5 p-5 lg:grid-cols-2">
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
								value={values.propertyId}
								onValueChange={(propertyId) => onChange({ propertyId })}
								disabled={!values.customerId || isPropertiesLoading}
							>
								<SelectTrigger className="w-full">
									<SelectValue
										placeholder={
											values.customerId
												? 'Sélectionner un contexte'
												: 'Choisir un client'
										}
									/>
								</SelectTrigger>
								<SelectContent>
									{properties.map((property) => (
										<SelectItem key={property.id} value={property.id}>
											{propertyDisplayName(property)}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						</FieldBlock>
					</div>

					<div className="border-t">
						<div className="flex items-center justify-between gap-3 px-5 py-4">
							<div>
								<h2 className="font-semibold">Lignes</h2>
								<p className="text-xs text-muted-foreground">
									Prestations, quantités, notes et photos.
								</p>
							</div>
							<Button
								type="button"
								variant="outline"
								size="sm"
								onClick={onAddLine}
							>
								<Plus />
								Ajouter
							</Button>
						</div>

						<div className="divide-y">
							{values.lines.map((line, index) => (
								<QuoteLineEditor
									key={line.clientId}
									index={index}
									line={line}
									serviceRates={serviceRates}
									canRemove={values.lines.length > 1}
									isUploading={isUploading}
									onChange={(patch) => onLineChange(index, patch)}
									onSelectServiceRate={(serviceRateId) =>
										onSelectServiceRate(index, serviceRateId)
									}
									onRemove={() => onRemoveLine(index)}
									onUploadPhoto={(file) => onUploadLinePhoto(index, file)}
								/>
							))}
						</div>
					</div>
				</SectionCard>

				<div className="flex flex-col gap-6">
					<SectionCard>
						<SectionHeader
							title="Aperçu API"
							description="Dernier total calculé après enregistrement."
						/>
						<div className="p-5">
							{lastCreated ? (
								<div className="flex flex-col gap-4">
									<div className="flex items-center justify-between gap-4">
										<div className="min-w-0">
											<p className="truncate font-mono text-xs text-muted-foreground">
												{lastCreated.id}
											</p>
											<p className="mt-1 text-2xl font-bold">
												{formatCents(lastCreated.total_cents)}
											</p>
										</div>
										<StatusBadge tone="brand">
											{quoteStatusLabel(lastCreated.status)}
										</StatusBadge>
									</div>
									<p className="text-sm text-muted-foreground">
										{lastCreated.lines.length} ligne
										{lastCreated.lines.length > 1 ? 's' : ''} sauvegardée
										{lastCreated.lines.length > 1 ? 's' : ''}
									</p>
								</div>
							) : (
								<p className="text-sm text-muted-foreground">
									Créez un devis pour afficher le total renvoyé par l'API.
								</p>
							)}
						</div>
					</SectionCard>

					<SectionCard>
						<SectionHeader
							title="Devis récents"
							description={
								selectedCustomer
									? `Client sélectionné : ${customerDisplayName(selectedCustomer)}`
									: 'Tous les devis de l’organisation'
							}
						/>
						{isLoading ? (
							<p className="p-5 text-sm text-muted-foreground">Chargement…</p>
						) : quotes.length === 0 ? (
							<p className="p-5 text-sm text-muted-foreground">
								Aucun devis enregistré.
							</p>
						) : (
							<ul className="divide-y">
								{quotes.slice(0, 8).map((quote) => (
									<li
										key={quote.id}
										className="flex items-center justify-between gap-4 px-5 py-4"
									>
										<div className="min-w-0">
											<div className="flex items-center gap-2">
												<p className="font-semibold">
													{formatCents(quote.total_cents)}
												</p>
												<StatusBadge tone={statusTone(quote.status)}>
													{quoteStatusLabel(quote.status)}
												</StatusBadge>
											</div>
											<p className="mt-1 truncate font-mono text-xs text-muted-foreground">
												{quote.id}
											</p>
										</div>
										<div className="shrink-0 text-right text-xs text-muted-foreground">
											<p>{formatDate(quote.created_at)}</p>
											<p>
												{quote.lines.length} ligne
												{quote.lines.length > 1 ? 's' : ''}
											</p>
										</div>
									</li>
								))}
							</ul>
						)}
					</SectionCard>
				</div>
			</div>
		</PageShell>
	)
}

interface QuoteLineEditorProps {
	index: number
	line: QuoteLineFormValues
	serviceRates: ServiceRate[]
	canRemove: boolean
	isUploading?: boolean
	onChange: (patch: Partial<QuoteLineFormValues>) => void
	onSelectServiceRate: (serviceRateId: string) => void
	onRemove: () => void
	onUploadPhoto: (file: File) => Promise<void>
}

function QuoteLineEditor({
	index,
	line,
	serviceRates,
	canRemove,
	isUploading,
	onChange,
	onSelectServiceRate,
	onRemove,
	onUploadPhoto,
}: QuoteLineEditorProps) {
	return (
		<div className="grid gap-4 p-5 lg:grid-cols-[minmax(190px,1.1fr)_minmax(160px,1fr)_120px_140px]">
			<FieldBlock label={`Prestation ${index + 1}`}>
				<Select
					value={line.serviceRateId || 'custom'}
					onValueChange={(value) => {
						onSelectServiceRate(value === 'custom' ? '' : value)
					}}
				>
					<SelectTrigger className="w-full">
						<SelectValue placeholder="Ligne libre" />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="custom">Ligne libre</SelectItem>
						{serviceRates.map((serviceRate) => (
							<SelectItem key={serviceRate.id} value={serviceRate.id}>
								{serviceRateDisplayName(serviceRate)}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</FieldBlock>

			<FieldBlock label="Libellé">
				<Input
					value={line.label}
					onChange={(event) => onChange({ label: event.target.value })}
					placeholder="Libellé de ligne"
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
					placeholder="0.00"
				/>
			</FieldBlock>

			<FieldBlock label="Unité">
				<Select
					value={line.unit}
					onValueChange={(unit) => onChange({ unit: unit as ServiceRateUnit })}
				>
					<SelectTrigger className="w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="HOUR">{formatUnit('HOUR')}</SelectItem>
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
						placeholder="Précisions utiles pour cadrer la prestation"
					/>
				</FieldBlock>
			</div>

			<div className="lg:col-span-1">
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
						<p className="mt-2 truncate text-xs text-muted-foreground">
							{line.photoKeys.length} fichier
							{line.photoKeys.length > 1 ? 's' : ''}
						</p>
					) : null}
				</FieldBlock>
			</div>

			<div className="flex items-end justify-end">
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
	)
}

interface FieldBlockProps {
	label: string
	children: React.ReactNode
}

function FieldBlock({ label, children }: FieldBlockProps) {
	const id = label.toLowerCase().replaceAll(/\s+/g, '-')
	return (
		<div className="flex min-w-0 flex-col gap-2">
			<Label htmlFor={id}>{label}</Label>
			{children}
		</div>
	)
}

function statusTone(status: Quote['status']) {
	if (status === 'ACCEPTED') return 'success'
	if (status === 'SENT') return 'brand'
	if (status === 'DECLINED' || status === 'CANCELLED') return 'error'
	return 'neutral'
}

export namespace QuoteCreateUI {
	export function Loading() {
		return (
			<PageShell>
				<SectionCard className="flex items-center justify-center p-12 text-sm text-muted-foreground">
					Chargement…
				</SectionCard>
			</PageShell>
		)
	}
}

import type { Product, ServiceRate } from '#/hooks/use-reference-catalog'
import type { CatalogExtras } from '#/pages/catalog/feature/mocks/catalog-extras'
import type { VatRate } from '#/pages/crm-settings/types'
import { centsToInput, parseAmount } from '#/pages/customers/lib/format'

export type CatalogKind = 'service' | 'product'

export type BillingFrequency = 'one_shot' | 'monthly' | 'quarterly' | 'yearly'

export const CATALOG_KIND_LABELS = {
	service: 'Service',
	product: 'Produit',
} as const

export const FREQUENCIES: readonly BillingFrequency[] = [
	'one_shot',
	'monthly',
	'quarterly',
	'yearly',
]

export const FREQUENCY_LABELS: Record<BillingFrequency, string> = {
	one_shot: 'Ponctuel',
	monthly: 'Mensuel',
	quarterly: 'Trimestriel',
	yearly: 'Annuel',
}

export const FREQUENCY_SUFFIX: Record<BillingFrequency, string> = {
	one_shot: '',
	monthly: '/ mois',
	quarterly: '/ trimestre',
	yearly: '/ an',
}

export const IMMUTABLE_NOTE =
	'Modifier un article n’a aucun effet sur les devis et factures existants : les lignes sont copiées au moment où tu les ajoutes.'

export type CatalogRow = {
	id: string
	kind: CatalogKind
	reference: string
	name: string
	description: string
	unit: string
	unitPriceCents: number
	vatRate: VatRate | null
	product: Product | null
	serviceRate: ServiceRate | null
}

const VAT_BP: Record<number, VatRate> = {
	0: 0,
	550: 5.5,
	1000: 10,
	2000: 20,
}

export function vatRateOfBp(bp: number | null | undefined): VatRate | null {
	return bp === null || bp === undefined ? null : (VAT_BP[bp] ?? null)
}

export function bpOfVatRate(rate: VatRate | null): number | null {
	return rate === null ? null : Math.round(rate * 100)
}

export function catalogRows(
	products: readonly Product[],
	serviceRates: readonly ServiceRate[],
): CatalogRow[] {
	return [
		...products.map(
			(product): CatalogRow => ({
				id: product.id,
				kind: 'product',
				reference: product.sku ?? '',
				name: product.name,
				description: product.description ?? '',
				unit: product.unit,
				unitPriceCents: product.unit_price_cents,
				vatRate: vatRateOfBp(product.default_vat_rate_bp),
				product,
				serviceRate: null,
			}),
		),
		...serviceRates.map(
			(serviceRate): CatalogRow => ({
				id: serviceRate.id,
				kind: 'service',
				reference: '',
				name: serviceRate.label,
				description: serviceRate.description ?? '',
				unit: serviceRate.unit,
				unitPriceCents: serviceRate.rate_cents,
				vatRate: vatRateOfBp(serviceRate.default_vat_rate_bp),
				product: null,
				serviceRate,
			}),
		),
	]
}

export type TypeFilter = 'all' | CatalogKind

export type CatalogFilters = {
	query: string
	type: TypeFilter
	category: string | null
	archived: boolean
}

export const DEFAULT_FILTERS: CatalogFilters = {
	query: '',
	type: 'all',
	category: null,
	archived: false,
}

export type CatalogSearch = {
	q?: string
	type?: CatalogKind
	categorie?: string
	archives?: true
}

export function normalizeText(text: string): string {
	return text.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase().trim()
}

export function filterCatalog(
	rows: readonly CatalogRow[],
	extrasOf: (id: string) => CatalogExtras,
	filters: CatalogFilters,
): CatalogRow[] {
	const query = normalizeText(filters.query)
	const kept = rows.filter((row) => {
		const extras = extrasOf(row.id)
		if (extras.archived && !filters.archived) return false
		if (filters.type !== 'all' && row.kind !== filters.type) return false
		if (filters.category !== null && extras.category !== filters.category)
			return false
		if (query === '') return true
		return normalizeText(`${row.name} ${row.reference}`).includes(query)
	})
	const ordered = (list: CatalogRow[]) =>
		[...list].sort(
			(a, b) =>
				extrasOf(a.id).category.localeCompare(extrasOf(b.id).category, 'fr') ||
				a.name.localeCompare(b.name, 'fr'),
		)
	return [
		...ordered(kept.filter((row) => !extrasOf(row.id).archived)),
		...ordered(kept.filter((row) => extrasOf(row.id).archived)),
	]
}

export function isFiltered(filters: CatalogFilters): boolean {
	return (
		filters.query.trim() !== '' ||
		filters.type !== DEFAULT_FILTERS.type ||
		filters.category !== DEFAULT_FILTERS.category ||
		filters.archived !== DEFAULT_FILTERS.archived
	)
}

function plural(count: number, one: string, many: string): string {
	return `${count} ${count > 1 ? many : one}`
}

export function catalogSummary(
	rows: readonly CatalogRow[],
	extrasOf: (id: string) => CatalogExtras,
): string {
	if (rows.length === 0) return 'Aucun article'
	const active = rows.filter((row) => !extrasOf(row.id).archived)
	const archived = rows.length - active.length
	const services = active.filter((row) => row.kind === 'service').length
	const products = active.length - services
	const base =
		active.length === 0
			? 'Aucun article actif'
			: `${plural(active.length, 'article actif', 'articles actifs')} (${plural(services, 'service', 'services')}, ${plural(products, 'produit', 'produits')})`
	return archived === 0
		? base
		: `${base}, ${plural(archived, 'archivé', 'archivés')}`
}

export type MarginTone = 'ok' | 'warn' | 'alert'

export function marginTone(percent: number): MarginTone {
	if (percent >= 35) return 'ok'
	if (percent >= 20) return 'warn'
	return 'alert'
}

export function formatPercent(percent: number): string {
	return `${String(percent).replace('.', ',')} %`
}

export function categoryOptions(
	categories: readonly string[],
	selected: string | null,
): string[] {
	const all = selected === null ? categories : [...categories, selected]
	return [...new Set(all.filter((category) => category.trim() !== ''))].sort(
		(a, b) => a.localeCompare(b, 'fr'),
	)
}

function text(value: unknown): string | undefined {
	const raw =
		typeof value === 'number'
			? String(value)
			: typeof value === 'string'
				? value
				: ''
	return raw.trim() === '' ? undefined : raw
}

export function validateCatalogSearch(
	raw: Record<string, unknown>,
): CatalogSearch {
	const q = text(raw.q)
	const categorie = text(raw.categorie)
	return {
		...(q ? { q } : {}),
		...(raw.type === 'service' || raw.type === 'product'
			? { type: raw.type }
			: {}),
		...(categorie ? { categorie } : {}),
		...(raw.archives === true || raw.archives === 'true'
			? { archives: true as const }
			: {}),
	}
}

export function filtersOf(search: CatalogSearch): CatalogFilters {
	return {
		query: search.q ?? DEFAULT_FILTERS.query,
		type: search.type ?? DEFAULT_FILTERS.type,
		category: search.categorie ?? DEFAULT_FILTERS.category,
		archived: search.archives === true,
	}
}

export function searchOf(filters: CatalogFilters): CatalogSearch {
	return {
		...(filters.query.trim() === '' ? {} : { q: filters.query }),
		...(filters.type === 'all' ? {} : { type: filters.type }),
		...(filters.category === null ? {} : { categorie: filters.category }),
		...(filters.archived ? { archives: true as const } : {}),
	}
}

export type CatalogForm = {
	kind: CatalogKind
	reference: string
	name: string
	description: string
	category: string
	unit: string
	price: string
	cost: string
	vatRate: VatRate | null
	frequency: BillingFrequency
	photoKeys: string[]
	supplierId: string
	manufacturer: string
	technicalSpecs: string
	datasheetKeys: string[]
}

export type CatalogField = 'name' | 'unit' | 'price' | 'cost'

export type FormEvaluation = {
	valid: boolean
	errors: Partial<Record<CatalogField, string>>
	priceCents: number | null
	costCents: number | null
}

const AMOUNT_HINT = 'Saisis un montant en euros, par exemple 520 ou 520,50.'

export function emptyForm(vatRate: VatRate | null): CatalogForm {
	return {
		kind: 'service',
		reference: '',
		name: '',
		description: '',
		category: '',
		unit: 'HOUR',
		price: '',
		cost: '',
		vatRate,
		frequency: 'one_shot',
		photoKeys: [],
		supplierId: '',
		manufacturer: '',
		technicalSpecs: '',
		datasheetKeys: [],
	}
}

export function formOf(row: CatalogRow, extras: CatalogExtras): CatalogForm {
	return {
		kind: row.kind,
		reference: row.reference,
		name: row.name,
		description: row.description,
		category: extras.category,
		unit: row.unit,
		price: centsToInput(row.unitPriceCents),
		cost: extras.cost === null ? '' : centsToInput(extras.cost),
		vatRate: row.vatRate,
		frequency: extras.frequency,
		photoKeys: row.product?.photo_keys ?? [],
		supplierId: row.product?.supplier_id ?? '',
		manufacturer: row.product?.manufacturer ?? '',
		technicalSpecs: row.product?.technical_specs ?? '',
		datasheetKeys: row.product?.datasheet_keys ?? [],
	}
}

export function duplicateOf(
	row: CatalogRow,
	extras: CatalogExtras,
): CatalogForm {
	return {
		...formOf(row, extras),
		reference: '',
		name: `${row.name} (copie)`,
	}
}

export function evaluateForm(form: CatalogForm): FormEvaluation {
	const errors: Partial<Record<CatalogField, string>> = {}
	const priceCents = parseAmount(form.price)
	const costGiven = form.cost.trim() !== ''
	const costCents = costGiven ? parseAmount(form.cost) : null
	if (form.name.trim() === '') errors.name = 'Le nom est obligatoire.'
	if (form.unit.trim() === '') errors.unit = 'L’unité est obligatoire.'
	if (priceCents === null)
		errors.price =
			form.price.trim() === '' ? 'Le prix HT est obligatoire.' : AMOUNT_HINT
	else if (priceCents < 0) errors.price = 'Le prix unitaire doit être positif.'
	if (costGiven && costCents === null) errors.cost = AMOUNT_HINT
	else if (costCents !== null && costCents < 0)
		errors.cost = 'Le prix de revient doit être positif.'
	return {
		valid: Object.keys(errors).length === 0,
		errors,
		priceCents,
		costCents,
	}
}

export function marginPreview(
	form: CatalogForm,
): { amount: number; percent: number } | null {
	const evaluation = evaluateForm(form)
	if (evaluation.priceCents === null || evaluation.costCents === null)
		return null
	return marginOf(evaluation.priceCents, evaluation.costCents)
}

export function marginOf(
	priceCents: number,
	costCents: number | null,
): { amount: number; percent: number } | null {
	if (costCents === null || priceCents <= 0) return null
	const amount = priceCents - costCents
	return { amount, percent: Math.round((amount / priceCents) * 1000) / 10 }
}

export function hasSameReference(
	rows: readonly CatalogRow[],
	reference: string,
	exceptId: string | null,
): boolean {
	const wanted = reference.trim().toLowerCase()
	if (wanted === '') return false
	return rows.some(
		(row) =>
			row.id !== exceptId && row.reference.trim().toLowerCase() === wanted,
	)
}

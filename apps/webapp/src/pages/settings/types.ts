import type { ProductCatalogFormValues } from '#/hooks/use-catalog-items'
import type { LegalMentionTemplate } from '#/hooks/use-legal-mentions'
import type { OrganizationContext } from '#/hooks/use-organization-contexts'
import type {
	Employee,
	Equipment,
	Product,
	ServiceRate,
	ServiceRateUnit,
} from '#/hooks/use-reference-catalog'

export type ReferenceTab =
	| 'employees'
	| 'equipment'
	| 'service-rates'
	| 'products'

export interface OrganizationFormValues {
	name: string
	slug: string
}

export interface EmployeeFormValues {
	name: string
	hourlyRate: string
	userId: string
}

export interface EquipmentFormValues {
	name: string
	hourlyRate: string
}

export interface ServiceRateFormValues {
	label: string
	unit: ServiceRateUnit
	rate: string
	vatRate: string
	customFields: { key: string; value: string }[]
}

export interface BillingFormValues {
	paymentTermsDays: string
	latePenaltyRate: string
	recoveryIndemnityEuros: string
	defaultDepositBasis: string | null
	defaultDepositValue: string | null
	defaultVatRate: string
	iban: string
	bic: string
	siret: string
	rcs: string
	ape: string
	vatIntracom: string
	footer: string
}

export interface LegalMentionFormValues {
	name: string
	body: string
}

export interface EmitterContextFormValues {
	label: string
	address_line: string
	postal_code: string
	city: string
	country: string
	siret: string
	rcs: string
	ape: string
	vat_intracom: string
	iban: string
	bic: string
}

export interface ReferenceCatalogData {
	employees: Employee[]
	equipment: Equipment[]
	serviceRates: ServiceRate[]
	products: Product[]
	legalMentionTemplates: LegalMentionTemplate[]
	organizationContexts: OrganizationContext[]
}

export type {
	LegalMentionTemplate,
	OrganizationContext,
	ProductCatalogFormValues,
	Product,
}

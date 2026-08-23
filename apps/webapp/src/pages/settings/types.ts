import type { LucideIcon } from 'lucide-react'
import type { ComponentType } from 'react'
import type { ModuleId } from '#/modules/types'

/** One entry in the `/settings` anchor column — restored from #112 (removed
 * in #186 when the page was down to a single section; automation needs it
 * back). `moduleId` groups sections under their owning module in the nav;
 * `undefined` lands the section under "Général" — see `nav.ts`. */
export interface SettingsSection {
	id: string
	label: string
	icon: LucideIcon
	moduleId?: ModuleId
	Component: ComponentType
}

export interface OrganizationFormValues {
	name: string
	slug: string
	/** Whether the field app's home screen offers clocking in/out — off by
	 * default since ADR 0002 (see `Organization.field_clock_enabled`). */
	fieldClockEnabled: boolean
}

/** `vatChoice` never carries a blank meaning "either" (#310's `VatStatus`):
 * `'undecided'` is its own explicit state, distinct from `'subject'` and
 * `'not_subject'`, and the form refuses to submit while it holds that
 * value. */
export interface LegalIdentityFormValues {
	legalName: string
	legalForm: string
	registrationNumber: string
	vatChoice: 'undecided' | 'subject' | 'not_subject'
	vatNumber: string
	vatExemptionBasis: string
	shareCapital: string
	addressLine1: string
	addressLine2: string
	addressPostalCode: string
	addressCity: string
	addressCountry: string
	contactEmail: string
	contactPhone: string
	insuranceMention: string
}

/** French labels for the field names `missing_legal_identity_fields`
 * carries — the same vocabulary the backend refusal on a PDF export uses
 * (#314), so "what's missing" reads the same wherever it appears. Also the
 * anchor id each field's input carries, so a mention here can jump straight
 * to it. */
export const LEGAL_IDENTITY_FIELD_LABELS: Record<string, string> = {
	legal_name: 'la raison sociale',
	legal_form: 'la forme juridique',
	registration_number: 'le SIRET',
	vat_status: 'le statut de TVA',
	address_line1: "l'adresse (ligne 1)",
	address_postal_code: 'le code postal',
	address_city: 'la ville',
	address_country: 'le pays',
	insurance_mention: "la mention d'assurance professionnelle",
}

export interface CredentialFormValues {
	kind: string
	name: string
	origin: 'supplied' | 'generated'
	/** Keyed by the chosen auth scheme's field name. Ignored when
	 * `origin === 'generated'` — the backend fabricates the secret itself
	 * and never reads this. */
	data: Record<string, string>
}

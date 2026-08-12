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

import {
	Boxes,
	KeyRound,
	LockKeyhole,
	type LucideIcon,
	Ticket,
} from 'lucide-react'
import { normalizeText } from '#/pages/automation/lib/text'

export interface CredentialAppearance {
	icon: LucideIcon
	hue: number
}

const APPEARANCES: Readonly<Record<string, CredentialAppearance>> = {
	http_basic: { icon: LockKeyhole, hue: 250 },
	bearer_token: { icon: Ticket, hue: 280 },
	http_header: { icon: KeyRound, hue: 200 },
	odoo_api: { icon: Boxes, hue: 300 },
}

const FALLBACK: CredentialAppearance = { icon: KeyRound, hue: 200 }

export function credentialAppearance(kind: string): CredentialAppearance {
	return APPEARANCES[kind] ?? FALLBACK
}

export function searchAuthSchemes<T extends { label: string }>(
	schemes: readonly T[],
	query: string,
): T[] {
	const needle = normalizeText(query)
	return schemes.filter(
		(scheme) => needle === '' || normalizeText(scheme.label).includes(needle),
	)
}

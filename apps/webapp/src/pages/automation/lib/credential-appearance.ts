import {
	FileBadge,
	FileKey,
	Fingerprint,
	KeyRound,
	type LucideIcon,
	ShieldCheck,
	Signature,
	Ticket,
} from 'lucide-react'
import { normalizeText } from '#/pages/automation/lib/text'

export interface CredentialAppearance {
	icon: LucideIcon
	hue: number
}

const RULES: readonly {
	match: RegExp
	appearance: CredentialAppearance
}[] = [
	{ match: /oauth/, appearance: { icon: ShieldCheck, hue: 150 } },
	{ match: /hmac|signature|sign/, appearance: { icon: Signature, hue: 20 } },
	{ match: /jwt/, appearance: { icon: FileKey, hue: 310 } },
	{ match: /cert/, appearance: { icon: FileBadge, hue: 50 } },
	{
		match: /basic|digest|password/,
		appearance: { icon: Fingerprint, hue: 250 },
	},
	{ match: /bearer|token/, appearance: { icon: Ticket, hue: 280 } },
]

const FALLBACK: CredentialAppearance = { icon: KeyRound, hue: 200 }

export function credentialAppearance(kind: string): CredentialAppearance {
	const normalized = normalizeText(kind)
	return (
		RULES.find((rule) => rule.match.test(normalized))?.appearance ?? FALLBACK
	)
}

export function searchAuthSchemes<T extends { kind: string; label: string }>(
	schemes: readonly T[],
	query: string,
): T[] {
	const needle = normalizeText(query)
	return schemes.filter(
		(scheme) =>
			needle === '' ||
			normalizeText(`${scheme.label} ${scheme.kind}`).includes(needle),
	)
}

import type { CredentialKind } from '#/hooks/use-automation'
import { normalizeText } from '#/pages/automation/lib/text'

export function findKind(
	kinds: readonly CredentialKind[],
	code: string,
): CredentialKind | undefined {
	return kinds.find((candidate) => candidate.kind === code)
}

export function kindLabel(
	kinds: readonly CredentialKind[],
	code: string,
): string {
	return findKind(kinds, code)?.label ?? code
}

export function acceptsUsage(kind: CredentialKind, usage: string): boolean {
	return kind.usages.some((candidate) => candidate.usage === usage)
}

export function kindsAccepting(
	kinds: readonly CredentialKind[],
	usage: string,
): CredentialKind[] {
	return kinds.filter((kind) => acceptsUsage(kind, usage))
}

export function credentialAcceptsUsage(
	kinds: readonly CredentialKind[],
	credential: { kind: string },
	usage: string,
): boolean {
	const kind = findKind(kinds, credential.kind)
	return kind !== undefined && acceptsUsage(kind, usage)
}

export function allowsGeneratedOrigin(kind: CredentialKind): boolean {
	return kind.origin_choices.includes('generated')
}

export function usageSentence(kind: CredentialKind): string {
	const names = (usages: CredentialKind['usages']) =>
		usages.map((usage) => usage.label).join(', ')
	const withNode = kind.usages.filter((usage) => usage.has_node)
	return withNode.length > 0
		? `Utilisable dans : ${names(withNode)}.`
		: `Prévue pour : ${names(kind.usages)}. Aucun nœud ne l’utilise encore.`
}

export function webhookCredentialHint(
	kinds: readonly CredentialKind[],
): string {
	const accepted = kindsAccepting(kinds, 'webhook')
	return accepted.length === 0
		? ''
		: `Types acceptés : ${accepted.map((kind) => kind.label).join(', ')}.`
}

export interface KindGroup {
	id: string
	label: string
	kinds: CredentialKind[]
}

export function searchKindGroups(
	kinds: readonly CredentialKind[],
	query: string,
): KindGroup[] {
	const needle = normalizeText(query)
	const groups = new Map<string, KindGroup>()
	for (const kind of kinds) {
		if (!kind.supported) continue
		const text = `${kind.label} ${kind.description} ${kind.category_label}`
		if (needle !== '' && !normalizeText(text).includes(needle)) continue
		const group = groups.get(kind.category)
		if (group) group.kinds.push(kind)
		else
			groups.set(kind.category, {
				id: kind.category,
				label: kind.category_label,
				kinds: [kind],
			})
	}
	return [...groups.values()]
}

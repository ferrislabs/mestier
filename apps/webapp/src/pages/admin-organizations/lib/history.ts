import type { Schemas } from '#/api/api.client'

export function historySentence(
	change: Schemas.AdminFeatureChangeResponse,
	features: ReadonlyArray<Schemas.AdminOrganizationFeatureResponse>,
): string {
	const actor = change.actor_name ?? 'Un administrateur'
	const verb = change.enabled ? 'activé' : 'désactivé'
	const label =
		features.find((feature) => feature.key === change.flag)?.label ??
		change.flag
	return `${actor} a ${verb} ${label}`
}

export function featureStateLine(
	feature: Schemas.AdminOrganizationFeatureResponse,
	formatDate: (iso: string) => string,
): string | null {
	if (!feature.enabled || !feature.enabled_at) return null
	const date = formatDate(feature.enabled_at)
	return feature.enabled_by_name
		? `Activé le ${date} par ${feature.enabled_by_name}`
		: `Activé le ${date}`
}

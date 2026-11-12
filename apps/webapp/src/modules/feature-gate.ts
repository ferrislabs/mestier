import type { FeatureKey } from '#/hooks/use-features'

export interface FeatureGated {
	requiredFeature?: FeatureKey
}

export function isFeatureVisible(
	target: FeatureGated,
	features: readonly string[],
	loaded: boolean,
): boolean {
	if (!target.requiredFeature) return true
	return loaded && features.includes(target.requiredFeature)
}

export function organizationHue(seed: string): number {
	let hash = 0
	for (const char of seed) {
		hash = (hash * 31 + char.charCodeAt(0)) % 360
	}
	return hash
}

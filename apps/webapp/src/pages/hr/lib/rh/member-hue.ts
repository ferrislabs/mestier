export function memberHue(memberId: string): number {
	let hash = 0
	for (const char of memberId) {
		hash = (hash * 31 + char.charCodeAt(0)) % 360
	}
	return hash
}

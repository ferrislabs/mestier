/**
 * A small set of generic colors assigned to a connector's `family` (or any
 * other seed string) by hashing it — never a lookup table keyed by
 * specific family names. A brand-new family from a catalogue addition gets
 * a color the same way an existing one does, with zero frontend change; a
 * fixed name→color map would silently fall back to "no color" for it
 * instead. Shared by the canvas node, the palette, and the connector
 * search list, so the same family always reads as the same color
 * everywhere on the page.
 */
const BADGE_COLORS = [
	'bg-blue-100 text-blue-700',
	'bg-purple-100 text-purple-700',
	'bg-emerald-100 text-emerald-700',
	'bg-orange-100 text-orange-700',
	'bg-pink-100 text-pink-700',
	'bg-teal-100 text-teal-700',
	'bg-amber-100 text-amber-700',
	'bg-indigo-100 text-indigo-700',
]

export function badgeColorFor(seed: string): string {
	let hash = 0
	for (let i = 0; i < seed.length; i++) {
		hash = (hash * 31 + seed.charCodeAt(i)) | 0
	}
	return BADGE_COLORS[Math.abs(hash) % BADGE_COLORS.length]
}

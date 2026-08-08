export interface LabelOption {
	id: string
	name: string
	color: string
}

/**
 * A fixed swatch offered when creating a label from the picker — the form
 * never prompts for a color explicitly (no color-picker component exists in
 * this repo's shadcn set, and the three seeded labels — Réunion,
 * Déplacement, Formation, see the planning remodel design doc — already
 * prove a handful of flat colors read fine as pastilles). Chosen for
 * pairwise contrast against both a light and a dark badge background.
 */
export const LABEL_COLOR_PALETTE = [
	'#2563EB', // blue
	'#16A34A', // green
	'#F59E0B', // amber
	'#DC2626', // red
	'#7C3AED', // violet
	'#0891B2', // cyan
	'#DB2777', // pink
	'#65A30D', // lime
] as const

/**
 * Case-insensitive, trimmed match against an organization's existing
 * labels — the picker's "type a name, create it if absent" flow (see the
 * planning remodel design doc's "Création de label" decision) needs this to
 * decide whether Enter selects an existing label or creates a new one, so a
 * user typing "réunion" reuses the seeded "Réunion" rather than creating a
 * near-duplicate.
 */
export function matchLabelByName<T extends { name: string }>(
	labels: T[],
	typedName: string,
): T | null {
	const normalized = typedName.trim().toLowerCase()
	if (!normalized) return null
	return (
		labels.find((label) => label.name.trim().toLowerCase() === normalized) ??
		null
	)
}

/** Adds `labelId` to the selection if absent, removes it if present — never mutates `selected`. */
export function toggleLabelSelection(
	selected: string[],
	labelId: string,
): string[] {
	return selected.includes(labelId)
		? selected.filter((id) => id !== labelId)
		: [...selected, labelId]
}

/**
 * The color offered for a new label — the first palette entry not already
 * used by an existing one, so a run of quick creations doesn't produce
 * several same-colored pastilles. Wraps around (repeating a color) once
 * every palette entry is taken rather than growing the palette or failing.
 */
export function nextLabelColor(existing: { color: string }[]): string {
	const used = new Set(existing.map((label) => label.color))
	const unused = LABEL_COLOR_PALETTE.find((color) => !used.has(color))
	return (
		unused ?? LABEL_COLOR_PALETTE[existing.length % LABEL_COLOR_PALETTE.length]
	)
}

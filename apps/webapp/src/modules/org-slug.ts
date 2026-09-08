/**
 * Slug rules for an organization, in one place: the onboarding form, the
 * creation dialog and the settings section all derive a slug the same way.
 *
 * Two normalizations, not one, and the difference is deliberate. While the
 * user types, a trailing dash has to survive — trimming it makes
 * `entreprise-dupont` impossible to type, since the dash is erased the moment
 * it is entered. What is sent to the API cannot keep it: the slug is a URL
 * segment.
 */

/** Derives a slug from a display name, accents removed. */
export function slugFromName(name: string): string {
	const withoutAccents = name.normalize('NFD').replace(/[\u0300-\u036f]/g, '')

	return trimDashes(normalizeSlugInput(withoutAccents))
}

/**
 * Normalization applied on every keystroke — see the note above on dashes.
 * A typed space becomes a dash rather than being trimmed away, so the next
 * word can be typed without reaching for the dash key.
 */
export function normalizeSlugInput(value: string): string {
	return value
		.toLowerCase()
		.replace(/\s+/g, '-')
		.replace(/[^a-z0-9-]/g, '')
		.replace(/-{2,}/g, '-')
}

/** Normalization applied before sending the slug to the API. */
export function normalizeSlugForPayload(value: string): string {
	return trimDashes(normalizeSlugInput(value))
}

function trimDashes(value: string): string {
	return value.replace(/^-|-$/g, '')
}

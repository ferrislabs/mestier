export const WEBHOOK_PREFIX = 'webhooks'

export const MAX_WEBHOOK_PATH_LENGTH = 200

const VALID_PATH = /^[a-z0-9_-]+(\/[a-z0-9_-]+)*$/

const KEPT_PREFIX = new RegExp(`^${WEBHOOK_PREFIX}(/|$)`)

export function normalizeWebhookPath(input: string): string {
	return input
		.normalize('NFD')
		.replace(/\p{Diacritic}/gu, '')
		.toLowerCase()
		.trim()
		.replace(/\s+/g, '-')
		.replace(/\/{2,}/g, '/')
		.replace(/^\/+|\/+$/g, '')
		.replace(KEPT_PREFIX, '')
}

export function webhookUrl(origin: string, path: string): string {
	return `${origin}/${WEBHOOK_PREFIX}/${path}`
}

export const PATH_MESSAGES = {
	empty: 'Indiquez la fin de l’URL après /webhooks/.',
	tooLong: `L’URL ne peut pas dépasser ${MAX_WEBHOOK_PATH_LENGTH} caractères.`,
	characters: 'Lettres minuscules, chiffres, « - », « _ » et « / » uniquement.',
	prefixed: 'Le préfixe /webhooks/ est déjà ajouté : retirez-le.',
}

export function webhookPathIssue(path: string): string | null {
	if (path === '') return PATH_MESSAGES.empty
	if (path.length > MAX_WEBHOOK_PATH_LENGTH) return PATH_MESSAGES.tooLong
	if (!VALID_PATH.test(path)) return PATH_MESSAGES.characters
	if (KEPT_PREFIX.test(path)) return PATH_MESSAGES.prefixed
	return null
}

export interface PathHolder {
	id: string
	label: string
	path: string
}

export function duplicatePathIssue(
	id: string,
	path: string,
	holders: readonly PathHolder[],
): string | null {
	const twin = holders.find(
		(holder) => holder.id !== id && holder.path === path,
	)
	return twin
		? `Cette URL est déjà utilisée par « ${twin.label} » dans ce workflow.`
		: null
}

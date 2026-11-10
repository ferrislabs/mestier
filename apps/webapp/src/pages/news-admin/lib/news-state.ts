export type NewsState = 'draft' | 'scheduled' | 'published'

export function newsState(
	publishedAt: string | null | undefined,
	now: Date,
): NewsState {
	if (!publishedAt) return 'draft'
	return new Date(publishedAt).getTime() > now.getTime()
		? 'scheduled'
		: 'published'
}

const LABELS: Record<NewsState, string> = {
	draft: 'Brouillon',
	scheduled: 'Programmée',
	published: 'Publiée',
}

export function newsStateLabel(state: NewsState): string {
	return LABELS[state]
}

export function formatNewsDate(iso: string | null | undefined): string {
	if (!iso) return '–'
	return new Intl.DateTimeFormat('fr-FR', {
		dateStyle: 'medium',
		timeStyle: 'short',
	}).format(new Date(iso))
}

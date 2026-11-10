import type { Schemas } from '#/api/api.client'
import { newsState } from './news-state'

export const MAX_TITLE = 120
export const MAX_ITEMS = 6
export const MAX_ITEM_TITLE = 60
export const MAX_ITEM_TEXT = 240
export const MAX_CTA_LABEL = 40

export type NewsDraftItem = Schemas.NewsItemRequest
export type PublicationMode = 'draft' | 'now' | 'scheduled'

export interface NewsDraft {
	title: string
	imageKey: string | null
	items: NewsDraftItem[]
	ctaEnabled: boolean
	ctaLabel: string
	ctaUrl: string
	mode: PublicationMode
	scheduledAt: string
	existingPublishedAt: string | null
}

export type NewsDraftErrors = Record<string, string>

export function emptyItem(): NewsDraftItem {
	return { icon: 'sparkles', title: '', text: '' }
}

export function emptyDraft(): NewsDraft {
	return {
		title: '',
		imageKey: null,
		items: [emptyItem()],
		ctaEnabled: false,
		ctaLabel: '',
		ctaUrl: '',
		mode: 'draft',
		scheduledAt: '',
		existingPublishedAt: null,
	}
}

function length(value: string): number {
	return [...value.trim()].length
}

function isHttpUrl(value: string): boolean {
	try {
		const url = new URL(value.trim())
		return url.protocol === 'https:' || url.protocol === 'http:'
	} catch {
		return false
	}
}

function bounded(value: string, max: number, label: string): string | null {
	const size = length(value)
	if (size === 0) return `${label} est obligatoire.`
	if (size > max) return `${label} : ${max} caractères maximum.`
	return null
}

export function validateDraft(draft: NewsDraft, now: Date): NewsDraftErrors {
	const errors: NewsDraftErrors = {}
	const title = bounded(draft.title, MAX_TITLE, 'Le titre')
	if (title) errors.title = title

	if (draft.items.length < 1) errors.items = 'Ajoutez au moins un point.'
	if (draft.items.length > MAX_ITEMS) {
		errors.items = `${MAX_ITEMS} points maximum.`
	}
	draft.items.forEach((item, index) => {
		const itemTitle = bounded(item.title, MAX_ITEM_TITLE, 'Le titre du point')
		if (itemTitle) errors[`item.${index}.title`] = itemTitle
		const text = bounded(item.text, MAX_ITEM_TEXT, 'Le texte du point')
		if (text) errors[`item.${index}.text`] = text
	})

	if (draft.ctaEnabled) {
		const label = bounded(draft.ctaLabel, MAX_CTA_LABEL, 'Le libellé')
		if (label) errors['cta.label'] = label
		if (length(draft.ctaUrl) === 0) {
			errors['cta.url'] = "L'adresse est obligatoire."
		} else if (!isHttpUrl(draft.ctaUrl)) {
			errors['cta.url'] = "L'adresse doit commencer par https:// ou http://."
		}
	}

	if (draft.mode === 'scheduled') {
		const date = draft.scheduledAt ? new Date(draft.scheduledAt) : null
		if (!date || Number.isNaN(date.getTime())) {
			errors.scheduledAt = 'Choisissez une date et une heure.'
		} else if (date.getTime() <= now.getTime()) {
			errors.scheduledAt = 'La date doit être dans le futur.'
		}
	}
	return errors
}

export function addItem(items: readonly NewsDraftItem[]): NewsDraftItem[] {
	if (items.length >= MAX_ITEMS) return [...items]
	return [...items, emptyItem()]
}

export function removeItem(
	items: readonly NewsDraftItem[],
	index: number,
): NewsDraftItem[] {
	return items.filter((_, position) => position !== index)
}

export function moveItem(
	items: readonly NewsDraftItem[],
	index: number,
	delta: -1 | 1,
): NewsDraftItem[] {
	const target = index + delta
	const next = [...items]
	const moved = next[index]
	const swapped = next[target]
	if (moved === undefined || swapped === undefined) return next
	next[index] = swapped
	next[target] = moved
	return next
}

export function toRequest(draft: NewsDraft, now: Date): Schemas.NewsRequest {
	let publishedAt: string | null = null
	if (draft.mode === 'now') {
		const existing = draft.existingPublishedAt
		publishedAt =
			existing && newsState(existing, now) === 'published'
				? existing
				: now.toISOString()
	} else if (draft.mode === 'scheduled') {
		publishedAt = new Date(draft.scheduledAt).toISOString()
	}
	return {
		title: draft.title.trim(),
		image_key: draft.imageKey,
		items: draft.items.map((item) => ({
			icon: item.icon,
			title: item.title.trim(),
			text: item.text.trim(),
		})),
		cta: draft.ctaEnabled
			? { label: draft.ctaLabel.trim(), url: draft.ctaUrl.trim() }
			: null,
		published_at: publishedAt,
	}
}

function pad(value: number): string {
	return String(value).padStart(2, '0')
}

function toLocalInput(iso: string): string {
	const date = new Date(iso)
	return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`
}

export function draftFromNews(
	news: Schemas.NewsResponse,
	now: Date,
): NewsDraft {
	const state = newsState(news.published_at, now)
	return {
		title: news.title,
		imageKey: news.image_key ?? null,
		items: news.items.map((item) => ({ ...item })),
		ctaEnabled: Boolean(news.cta),
		ctaLabel: news.cta?.label ?? '',
		ctaUrl: news.cta?.url ?? '',
		mode:
			state === 'draft' ? 'draft' : state === 'scheduled' ? 'scheduled' : 'now',
		scheduledAt:
			state === 'scheduled' && news.published_at
				? toLocalInput(news.published_at)
				: '',
		existingPublishedAt: news.published_at ?? null,
	}
}

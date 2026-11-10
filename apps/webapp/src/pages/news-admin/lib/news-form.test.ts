import { describe, expect, it } from 'vitest'
import {
	addItem,
	draftFromNews,
	emptyDraft,
	moveItem,
	type NewsDraft,
	removeItem,
	toRequest,
	validateDraft,
} from './news-form'

const NOW = new Date('2026-11-10T12:00:00Z')

function valid(): NewsDraft {
	return {
		...emptyDraft(),
		title: 'Nouveautés',
		items: [{ icon: 'rocket', title: 'Rapide', text: 'Plus rapide.' }],
	}
}

describe('validateDraft', () => {
	it('accepts a valid draft', () => {
		expect(validateDraft(valid(), NOW)).toEqual({})
	})

	it('rejects a blank or too long title', () => {
		expect(validateDraft({ ...valid(), title: '  ' }, NOW).title).toBeDefined()
		expect(
			validateDraft({ ...valid(), title: 'a'.repeat(121) }, NOW).title,
		).toBeDefined()
		expect(
			validateDraft({ ...valid(), title: 'a'.repeat(120) }, NOW).title,
		).toBeUndefined()
	})

	it('requires between 1 and 6 points', () => {
		expect(validateDraft({ ...valid(), items: [] }, NOW).items).toBeDefined()
		const seven = Array.from({ length: 7 }, () => ({
			icon: 'star' as const,
			title: 'a',
			text: 'b',
		}))
		expect(validateDraft({ ...valid(), items: seven }, NOW).items).toBeDefined()
		const six = seven.slice(0, 6)
		expect(validateDraft({ ...valid(), items: six }, NOW).items).toBeUndefined()
	})

	it('bounds point title and text', () => {
		const draft = {
			...valid(),
			items: [{ icon: 'star' as const, title: '', text: '' }],
		}
		const errors = validateDraft(draft, NOW)
		expect(errors['item.0.title']).toBeDefined()
		expect(errors['item.0.text']).toBeDefined()
		const long = {
			...valid(),
			items: [
				{ icon: 'star' as const, title: 'a'.repeat(61), text: 'a'.repeat(241) },
			],
		}
		const longErrors = validateDraft(long, NOW)
		expect(longErrors['item.0.title']).toBeDefined()
		expect(longErrors['item.0.text']).toBeDefined()
	})

	it('validates the button only when it is used', () => {
		expect(validateDraft(valid(), NOW)).toEqual({})
		const bad = { ...valid(), ctaEnabled: true, ctaLabel: '', ctaUrl: '' }
		const errors = validateDraft(bad, NOW)
		expect(errors['cta.label']).toBeDefined()
		expect(errors['cta.url']).toBeDefined()
		expect(
			validateDraft(
				{
					...valid(),
					ctaEnabled: true,
					ctaLabel: 'a'.repeat(41),
					ctaUrl: 'https://x.fr',
				},
				NOW,
			)['cta.label'],
		).toBeDefined()
	})

	it('accepts only http and https urls', () => {
		const withUrl = (ctaUrl: string) =>
			validateDraft(
				{ ...valid(), ctaEnabled: true, ctaLabel: 'Lire', ctaUrl },
				NOW,
			)['cta.url']
		expect(withUrl('https://mestier.fr/blog')).toBeUndefined()
		expect(withUrl('http://mestier.fr')).toBeUndefined()
		expect(withUrl('javascript:alert(1)')).toBeDefined()
		expect(withUrl('ftp://mestier.fr')).toBeDefined()
		expect(withUrl('mestier.fr')).toBeDefined()
	})

	it('requires a valid date when scheduling', () => {
		const scheduled = { ...valid(), mode: 'scheduled' as const }
		expect(validateDraft(scheduled, NOW).scheduledAt).toBeDefined()
		expect(
			validateDraft({ ...scheduled, scheduledAt: 'nope' }, NOW).scheduledAt,
		).toBeDefined()
		expect(
			validateDraft({ ...scheduled, scheduledAt: '2026-11-12T09:00' }, NOW)
				.scheduledAt,
		).toBeUndefined()
	})
})

describe('items editing', () => {
	it('adds up to 6 points', () => {
		let items = valid().items
		for (let i = 0; i < 10; i++) items = addItem(items)
		expect(items).toHaveLength(6)
	})

	it('removes and moves points', () => {
		const items = [
			{ icon: 'star' as const, title: 'a', text: 'a' },
			{ icon: 'star' as const, title: 'b', text: 'b' },
			{ icon: 'star' as const, title: 'c', text: 'c' },
		]
		expect(removeItem(items, 1).map((i) => i.title)).toEqual(['a', 'c'])
		expect(moveItem(items, 0, 1).map((i) => i.title)).toEqual(['b', 'a', 'c'])
		expect(moveItem(items, 0, -1)).toEqual(items)
		expect(moveItem(items, 2, 1)).toEqual(items)
	})
})

describe('toRequest', () => {
	it('sends a null date for a draft and trims text', () => {
		const request = toRequest({ ...valid(), title: '  Hello ' }, NOW)
		expect(request.title).toBe('Hello')
		expect(request.published_at).toBeNull()
		expect(request.cta).toBeNull()
		expect(request.image_key).toBeNull()
	})

	it('publishes now', () => {
		expect(toRequest({ ...valid(), mode: 'now' }, NOW).published_at).toBe(
			NOW.toISOString(),
		)
	})

	it('keeps the original publication date of an already published news', () => {
		const draft = {
			...valid(),
			mode: 'now' as const,
			existingPublishedAt: '2026-10-01T08:00:00Z',
		}
		expect(toRequest(draft, NOW).published_at).toBe('2026-10-01T08:00:00Z')
	})

	it('converts a scheduled local date to an instant', () => {
		const request = toRequest(
			{ ...valid(), mode: 'scheduled', scheduledAt: '2026-11-12T09:00' },
			NOW,
		)
		expect(request.published_at).toBe(
			new Date('2026-11-12T09:00').toISOString(),
		)
	})

	it('includes the button when enabled', () => {
		const request = toRequest(
			{
				...valid(),
				ctaEnabled: true,
				ctaLabel: ' Lire ',
				ctaUrl: ' https://x.fr ',
				imageKey: 'k',
			},
			NOW,
		)
		expect(request.cta).toEqual({ label: 'Lire', url: 'https://x.fr' })
		expect(request.image_key).toBe('k')
	})
})

describe('draftFromNews', () => {
	const base = {
		id: 'n1',
		title: 'T',
		items: [{ icon: 'star' as const, title: 'a', text: 'b' }],
		created_at: '2026-11-01T00:00:00Z',
		updated_at: '2026-11-01T00:00:00Z',
	}

	it('restores a draft', () => {
		expect(draftFromNews(base, NOW).mode).toBe('draft')
	})

	it('restores a published news', () => {
		const draft = draftFromNews(
			{ ...base, published_at: '2026-11-02T00:00:00Z' },
			NOW,
		)
		expect(draft.mode).toBe('now')
		expect(draft.existingPublishedAt).toBe('2026-11-02T00:00:00Z')
	})

	it('restores a scheduled news with a local datetime value', () => {
		const draft = draftFromNews(
			{
				...base,
				published_at: '2026-11-20T09:00:00Z',
				cta: { label: 'L', url: 'https://x.fr' },
				image_key: 'img',
			},
			NOW,
		)
		expect(draft.mode).toBe('scheduled')
		expect(draft.scheduledAt).toMatch(/^2026-11-20T\d\d:\d\d$/)
		expect(draft.ctaEnabled).toBe(true)
		expect(draft.imageKey).toBe('img')
	})
})

// MOCK, pas d'endpoint, remplacé au lot B4 (devis : validité, acompte, motif de refus, signataire, responsable, opportunité, notes, remise et fréquence par ligne, révisions, « remplacé »).
import { useCallback, useSyncExternalStore } from 'react'
import {
	mergeQuoteExtras,
	parseQuoteExtras,
	type QuoteExtras,
} from '#/pages/quotes/lib/quote-extras'

const KEY_PREFIX = 'mestier.mock.quote-extras.'

const cache = new Map<string, QuoteExtras>()
const listeners = new Set<() => void>()
let version = 0

function storage(): Storage | null {
	return typeof window === 'undefined' ? null : window.localStorage
}

function emit() {
	version += 1
	for (const listener of listeners) listener()
}

export function resetQuoteExtrasCache() {
	cache.clear()
	emit()
}

export function readQuoteExtras(quoteId: string): QuoteExtras {
	const cached = cache.get(quoteId)
	if (cached) return cached
	const parsed = parseQuoteExtras(
		storage()?.getItem(`${KEY_PREFIX}${quoteId}`) ?? null,
	)
	cache.set(quoteId, parsed)
	return parsed
}

export function saveQuoteExtras(quoteId: string, patch: Partial<QuoteExtras>) {
	const next = mergeQuoteExtras(readQuoteExtras(quoteId), patch)
	cache.set(quoteId, next)
	storage()?.setItem(`${KEY_PREFIX}${quoteId}`, JSON.stringify(next))
	emit()
}

function subscribe(listener: () => void) {
	listeners.add(listener)
	return () => {
		listeners.delete(listener)
	}
}

function currentVersion() {
	return version
}

export function useQuoteExtras(quoteId: string): QuoteExtras {
	useSyncExternalStore(subscribe, currentVersion, currentVersion)
	return readQuoteExtras(quoteId)
}

export function useQuoteExtrasOf(): (quoteId: string) => QuoteExtras {
	const seen = useSyncExternalStore(subscribe, currentVersion, currentVersion)
	return useCallback(
		(quoteId: string) => {
			void seen
			return readQuoteExtras(quoteId)
		},
		[seen],
	)
}

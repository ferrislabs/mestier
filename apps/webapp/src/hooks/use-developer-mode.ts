import { useSyncExternalStore } from 'react'

const STORAGE_KEY = 'mestier.developerMode'

const listeners = new Set<() => void>()

function subscribe(listener: () => void): () => void {
	listeners.add(listener)
	return () => listeners.delete(listener)
}

function getSnapshot(): boolean {
	if (typeof window === 'undefined') return false
	return window.localStorage.getItem(STORAGE_KEY) === 'true'
}

function getServerSnapshot(): boolean {
	return false
}

/**
 * A browser preference, not organization data (#446): resource ids are noise
 * for an artisan and a lifeline when debugging, so they stay hidden by
 * default and a toggle in Réglages > Général reveals them everywhere at
 * once — `useSyncExternalStore` over `localStorage` rather than a context so
 * every already-mounted consumer reacts the moment the toggle flips,
 * without threading a provider through the app shell.
 */
export function useDeveloperMode(): boolean {
	return useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot)
}

export function setDeveloperMode(enabled: boolean): void {
	window.localStorage.setItem(STORAGE_KEY, String(enabled))
	for (const listener of listeners) listener()
}

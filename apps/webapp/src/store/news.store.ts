import { Store } from '@tanstack/store'

export interface NewsStoreState {
	showAll: boolean
}

export const newsStore = new Store<NewsStoreState>({ showAll: false })

export function openAllNews(): void {
	newsStore.setState(() => ({ showAll: true }))
}

export function closeAllNews(): void {
	newsStore.setState(() => ({ showAll: false }))
}

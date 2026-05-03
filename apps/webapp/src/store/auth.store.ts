import { Store } from '@tanstack/store'

export interface AuthStoreState {
	accessToken: string | null
	expiresAt: number | null
	isAuthenticated: boolean
}

const initialState: AuthStoreState = {
	accessToken: null,
	expiresAt: null,
	isAuthenticated: false,
}

export const authStore = new Store<AuthStoreState>(initialState)

export function setAuth(partial: Partial<AuthStoreState>): void {
	authStore.setState((prev) => ({ ...prev, ...partial }))
}

export function clearAuth(): void {
	authStore.setState(() => initialState)
}

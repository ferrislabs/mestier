import { useSyncExternalStore } from 'react'
import {
	forgetCredentialMeta,
	patchCredentialMeta,
	readCredentialMeta,
	rememberCredentialKinds,
	subscribeCredentialMeta,
} from '#/pages/automation/feature/mocks/credential-meta'
import type { CredentialMeta } from '#/pages/automation/lib/credential-meta'

export function useCredentialMeta(): {
	data: Readonly<Record<string, CredentialMeta>>
	isLoading: boolean
} {
	const data = useSyncExternalStore(subscribeCredentialMeta, readCredentialMeta)
	return { data, isLoading: false }
}

export function useCredentialMetaActions() {
	return {
		rememberKinds: rememberCredentialKinds,
		patch: patchCredentialMeta,
		forget: forgetCredentialMeta,
	}
}

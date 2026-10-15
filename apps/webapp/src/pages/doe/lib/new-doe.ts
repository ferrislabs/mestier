export type DoeButtonState = 'loading' | 'none' | { revision: string }

export function doeTabLabel(state: DoeButtonState): string {
	if (typeof state === 'object') return `DOE · indice ${state.revision}`
	return 'DOE'
}

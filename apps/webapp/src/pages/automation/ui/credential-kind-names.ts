import type { CSSProperties } from 'react'

export function kindTransitionStyle(
	part: 'icon' | 'label',
	kind: string,
): CSSProperties {
	return { viewTransitionName: `credential-${part}-${kind}` }
}

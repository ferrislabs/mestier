import type { ReactNode } from 'react'
import { useDeveloperMode } from '#/hooks/use-developer-mode'

interface DeveloperModeOnlyProps {
	children: ReactNode
}

/**
 * Hides `children` outright unless the browser's "mode développeur"
 * preference is on — mirrors `RequirePermission`, but gates on a personal
 * display preference rather than an authorization decision (#446).
 */
export function DeveloperModeOnly({ children }: DeveloperModeOnlyProps) {
	const developerMode = useDeveloperMode()
	if (!developerMode) return null
	return <>{children}</>
}

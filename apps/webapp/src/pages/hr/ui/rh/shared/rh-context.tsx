import { createContext, type ReactNode, useContext } from 'react'
import { buildOrgPath } from '#/modules/org-path'
import type { ProfileFields } from '#/pages/hr/lib/rh/member-profile'
import type { ActionContext, RhState } from '#/pages/hr/lib/rh/types'

export type RhUpdate = (
	update: (current: RhState, context: ActionContext) => RhState,
) => Promise<void>

export interface RhProfileActions {
	save: (memberId: string, fields: ProfileFields) => Promise<void>
	archive: (memberId: string) => Promise<void>
	restore: (memberId: string) => Promise<void>
}

export interface RhContextValue {
	state: RhState
	update: RhUpdate
	profiles: RhProfileActions
	actorId: string
	organizationSlug: string
}

const RhContext = createContext<RhContextValue | null>(null)

export function RhProvider({
	value,
	children,
}: {
	value: RhContextValue
	children: ReactNode
}) {
	return <RhContext.Provider value={value}>{children}</RhContext.Provider>
}

function useRhContext(): RhContextValue {
	const value = useContext(RhContext)
	if (!value) throw new Error('RhProvider manquant')
	return value
}

export function useRhState(): RhState {
	return useRhContext().state
}

export function useRhUpdate(): RhUpdate {
	return useRhContext().update
}

export function useRhProfileActions(): RhProfileActions {
	return useRhContext().profiles
}

export function useRhActorId(): string {
	return useRhContext().actorId
}

export function useRhOrgPath(): (path: string) => string {
	const { organizationSlug } = useRhContext()
	return (path) => buildOrgPath(organizationSlug, path)
}

export function useRhOrgSlug(): string {
	return useRhContext().organizationSlug
}

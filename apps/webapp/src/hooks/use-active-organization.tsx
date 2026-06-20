import { createContext, useContext, useEffect, useMemo, useState } from 'react'
import type { Organization } from '#/hooks/use-organizations'

const ACTIVE_ORGANIZATION_STORAGE_KEY = 'mestier.activeOrganizationId'

interface ActiveOrganizationContextValue {
	organizations: Organization[]
	activeOrganization: Organization
	activeOrganizationId: string
	setActiveOrganizationId: (organizationId: string) => void
}

const ActiveOrganizationContext =
	createContext<ActiveOrganizationContextValue | null>(null)

interface ActiveOrganizationProviderProps {
	organizations: Organization[]
	children: React.ReactNode
}

export function ActiveOrganizationProvider({
	organizations,
	children,
}: ActiveOrganizationProviderProps) {
	const [storedOrganizationId, setStoredOrganizationId] = useState<
		string | null
	>(() => {
		if (typeof window === 'undefined') return null
		return window.localStorage.getItem(ACTIVE_ORGANIZATION_STORAGE_KEY)
	})

	const activeOrganization = (organizations.find(
		(organization) => organization.id === storedOrganizationId,
	) ?? organizations[0]) as Organization

	useEffect(() => {
		if (!activeOrganization) return
		if (storedOrganizationId === activeOrganization.id) return
		setStoredOrganizationId(activeOrganization.id)
	}, [activeOrganization, storedOrganizationId])

	useEffect(() => {
		if (!activeOrganization) return
		window.localStorage.setItem(
			ACTIVE_ORGANIZATION_STORAGE_KEY,
			activeOrganization.id,
		)
	}, [activeOrganization])

	const value = useMemo(
		() => ({
			organizations,
			activeOrganization,
			activeOrganizationId: activeOrganization.id,
			setActiveOrganizationId: setStoredOrganizationId,
		}),
		[activeOrganization, organizations],
	)

	return (
		<ActiveOrganizationContext.Provider value={value}>
			{children}
		</ActiveOrganizationContext.Provider>
	)
}

export function useActiveOrganization() {
	const context = useContext(ActiveOrganizationContext)
	if (!context) {
		throw new Error(
			'useActiveOrganization must be used inside ActiveOrganizationProvider',
		)
	}
	return context
}

import { createContext, useContext, useEffect, useMemo } from 'react'
import type { Organization } from '#/hooks/use-organizations'

const LAST_ORGANIZATION_STORAGE_KEY = 'mestier.lastOrganizationSlug'

/**
 * Dernier tenant visité. Ce n'est plus la source de vérité de l'organisation
 * active — c'est l'URL qui la porte — mais seulement la préférence d'entrée
 * utilisée quand on arrive sur `/`.
 */
export function readLastOrganizationSlug(): string | null {
	if (typeof window === 'undefined') return null
	return window.localStorage.getItem(LAST_ORGANIZATION_STORAGE_KEY)
}

interface OrganizationListContextValue {
	organizations: Organization[]
}

const OrganizationListContext =
	createContext<OrganizationListContextValue | null>(null)

interface OrganizationListProviderProps {
	organizations: Organization[]
	children: React.ReactNode
}

export function OrganizationListProvider({
	organizations,
	children,
}: OrganizationListProviderProps) {
	const value = useMemo(() => ({ organizations }), [organizations])

	return (
		<OrganizationListContext.Provider value={value}>
			{children}
		</OrganizationListContext.Provider>
	)
}

export function useOrganizationList() {
	const context = useContext(OrganizationListContext)
	if (!context) {
		throw new Error(
			'useOrganizationList must be used inside OrganizationListProvider',
		)
	}
	return context.organizations
}

interface ActiveOrganizationContextValue {
	organizations: Organization[]
	activeOrganization: Organization
	activeOrganizationId: string
}

const ActiveOrganizationContext =
	createContext<ActiveOrganizationContextValue | null>(null)

interface ActiveOrganizationProviderProps {
	activeOrganization: Organization
	children: React.ReactNode
}

export function ActiveOrganizationProvider({
	activeOrganization,
	children,
}: ActiveOrganizationProviderProps) {
	const organizations = useOrganizationList()

	useEffect(() => {
		window.localStorage.setItem(
			LAST_ORGANIZATION_STORAGE_KEY,
			activeOrganization.slug,
		)
	}, [activeOrganization])

	const value = useMemo(
		() => ({
			organizations,
			activeOrganization,
			activeOrganizationId: activeOrganization.id,
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

import { useEffect, useMemo } from 'react'

import { EmptyState } from '#/components/empty-state'
import { PageHeader, PageShell } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { buildSettingsNavGroups } from '#/pages/settings/nav'
import { SETTINGS_SECTIONS } from '#/pages/settings/registry'
import { SettingsLayout } from '#/pages/settings/ui/settings-layout'
import { useActiveSection } from '#/pages/settings/use-active-section'

/**
 * Settings, assembled from the section registry — restored (#155) now that
 * automation joins "Organisation": a single section needs no anchor
 * navigation, two do. See `registry.tsx` to add another.
 */
export function SettingsFeature() {
	const { activeOrganization } = useActiveOrganization()
	const groups = useMemo(() => buildSettingsNavGroups(SETTINGS_SECTIONS), [])
	const ids = useMemo(() => SETTINGS_SECTIONS.map((section) => section.id), [])
	const activeId = useActiveSection(ids)

	useEffect(() => {
		const id = window.location.hash.slice(1)
		if (id) document.getElementById(id)?.scrollIntoView()
	}, [])

	if (!activeOrganization) {
		return (
			<PageShell>
				<EmptyState
					title="Organisation indisponible"
					hint="Les paramètres nécessitent une organisation active."
				/>
			</PageShell>
		)
	}

	return (
		<PageShell>
			<PageHeader
				title="Paramètres"
				description="Configurez l'espace de travail et chacun des modules installés."
			/>
			<SettingsLayout
				groups={groups}
				sections={SETTINGS_SECTIONS}
				activeId={activeId}
			/>
		</PageShell>
	)
}

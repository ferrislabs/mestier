import { useMemo } from 'react'

import { PageHeader, PageShell } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { buildSettingsNavGroups } from '#/pages/settings/nav'
import { SETTINGS_SECTIONS } from '#/pages/settings/registry'
import { SettingsLayout } from '#/pages/settings/ui/settings-layout'
import { useActiveSection } from '#/pages/settings/use-active-section'

export function SettingsFeature() {
	const { activeOrganization } = useActiveOrganization()
	const groups = useMemo(() => buildSettingsNavGroups(SETTINGS_SECTIONS), [])
	const ids = useMemo(() => SETTINGS_SECTIONS.map((section) => section.id), [])
	const activeId = useActiveSection(ids)

	return (
		<PageShell>
			<PageHeader
				eyebrow={activeOrganization.name}
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

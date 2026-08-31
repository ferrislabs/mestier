import { AlertCircle } from 'lucide-react'
import { useMemo } from 'react'

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

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-xl border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-medium">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						Les paramètres nécessitent une organisation active.
					</p>
				</div>
			</div>
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

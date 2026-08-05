import { AlertCircle } from 'lucide-react'
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

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Organisation indisponible</p>
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

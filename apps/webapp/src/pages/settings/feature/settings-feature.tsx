import { AlertCircle } from 'lucide-react'

import { PageHeader, PageShell } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { OrganizationSection } from '#/pages/settings/sections/organization-section'

/**
 * Organization settings.
 *
 * The screen now carries only what is configured once: the catalogue and the
 * equipment, edited daily, moved to the modules that consume them. A single
 * section, hence no more anchor navigation — it will come back the day members,
 * roles or billing join it.
 */
export function SettingsFeature() {
	const { activeOrganization } = useActiveOrganization()

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
				eyebrow={activeOrganization.name}
				title="Paramètres"
				description="L'identité de votre espace de travail."
			/>
			<OrganizationSection />
		</PageShell>
	)
}

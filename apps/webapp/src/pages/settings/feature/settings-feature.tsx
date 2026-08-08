import { AlertCircle } from 'lucide-react'

import { PageHeader, PageShell } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { OrganizationSection } from '#/pages/settings/sections/organization-section'

/**
 * Réglages de l'organisation.
 *
 * L'écran ne porte plus que ce qui se configure une fois : le catalogue et le
 * matériel, qui s'éditent au quotidien, ont rejoint les modules qui les
 * consomment. Une seule section, donc plus de navigation par ancres — elle
 * reviendra le jour où membres, rôles ou facturation s'y ajouteront.
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

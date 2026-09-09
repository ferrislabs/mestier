import { Label } from '#/components/ui/label'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import { Switch } from '#/components/ui/switch'
import { setDeveloperMode, useDeveloperMode } from '#/hooks/use-developer-mode'

/**
 * A personal preference, not organization data (#446): stored in this
 * browser only, so it is not part of `OrganizationSection`'s form and has
 * nothing to save — the switch takes effect the moment it moves.
 */
export function DeveloperSection() {
	const developerMode = useDeveloperMode()

	return (
		<div className="flex flex-col gap-6">
			<SectionCard>
				<SectionHeader
					title="Mode développeur"
					description="Une préférence propre à ce navigateur, pas à l'organisation."
				/>
				<div className="flex items-center justify-between gap-4 p-5">
					<div>
						<Label htmlFor="developer-mode" className="text-sm font-medium">
							Afficher les identifiants
						</Label>
						<p className="mt-1 text-sm text-muted-foreground">
							Affiche l'identifiant technique de chaque produit, service,
							compte, équipement, projet et affectation — utile pour retrouver
							une ressource depuis un lien ou un journal, superflu le reste du
							temps.
						</p>
					</div>
					<Switch
						id="developer-mode"
						checked={developerMode}
						onCheckedChange={setDeveloperMode}
					/>
				</div>
			</SectionCard>
		</div>
	)
}

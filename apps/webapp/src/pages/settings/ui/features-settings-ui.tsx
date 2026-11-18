import { Loader2 } from 'lucide-react'
import { SectionCard } from '#/components/ui/surface'
import { Switch } from '#/components/ui/switch'
import type { SelfServiceFeature } from '#/hooks/use-features'
import { SettingsCard } from '#/pages/settings/ui/settings-card'

export interface FeaturesSettingsUIProps {
	isLoading: boolean
	features: SelfServiceFeature[]
	canManage: boolean
	pendingKey: string | null
	loadError: string | null
	onToggle: (key: string, enabled: boolean) => void
}

export function FeaturesSettingsUI({
	isLoading,
	features,
	canManage,
	pendingKey,
	loadError,
	onToggle,
}: FeaturesSettingsUIProps) {
	if (isLoading) {
		return (
			<SectionCard className="flex min-h-40 rounded-lg shadow-none items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
				<Loader2 className="size-5 animate-spin" />
				Chargement…
			</SectionCard>
		)
	}

	return (
		<SettingsCard
			title="Fonctionnalités"
			description="Activez les fonctionnalités dont votre organisation a besoin. Désactiver une fonctionnalité la masque sans supprimer ses données."
		>
			<div className="flex flex-col gap-4 p-5">
				{loadError ? (
					<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
						{loadError}
					</p>
				) : null}
				{features.length === 0 && !loadError ? (
					<p className="text-sm text-muted-foreground">
						Aucune fonctionnalité activable pour le moment.
					</p>
				) : null}
				<ul className="flex flex-col divide-y">
					{features.map((feature) => (
						<li
							key={feature.key}
							className="flex items-start justify-between gap-4 py-3 first:pt-0 last:pb-0"
						>
							<div className="min-w-0">
								<h3 className="font-medium text-sm">{feature.label}</h3>
								<p className="mt-1 text-[13px] text-muted-foreground">
									{feature.description}
								</p>
							</div>
							<div className="flex shrink-0 items-center gap-2 pt-0.5">
								{pendingKey === feature.key ? (
									<Loader2
										className="size-4 animate-spin text-muted-foreground"
										aria-hidden
									/>
								) : null}
								{canManage ? (
									<Switch
										aria-label={feature.label}
										checked={feature.enabled}
										disabled={pendingKey !== null}
										onCheckedChange={(checked) =>
											onToggle(feature.key, checked)
										}
									/>
								) : (
									<span className="text-[13px] text-muted-foreground">
										{feature.enabled ? 'Activée' : 'Désactivée'}
									</span>
								)}
							</div>
						</li>
					))}
				</ul>
				{canManage ? null : (
					<p className="text-xs text-muted-foreground">
						Seuls les membres autorisés à gérer l'organisation peuvent activer
						ou désactiver une fonctionnalité.
					</p>
				)}
			</div>
		</SettingsCard>
	)
}

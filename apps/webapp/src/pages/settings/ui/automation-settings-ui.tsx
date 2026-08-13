import { Loader2, Save } from 'lucide-react'
import { TextField } from '#/components/reference-table'
import { Button } from '#/components/ui/button'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import type { SettingsFormValues } from '#/pages/settings/lib/automation'
import { formatRetrySchedulePreview } from '#/pages/settings/lib/automation'

export interface AutomationSettingsUIProps {
	isLoading: boolean
	values: SettingsFormValues
	/** Parsed live from `values` by the feature — a preview only, the
	 * instance bounds themselves are never checked here. See
	 * `parseSettingsForm`'s doc comment. */
	retrySchedulePreview: number[]
	isPending: boolean
	formError: string | null
	saveError: string | null
	onChange: (patch: Partial<SettingsFormValues>) => void
	onSubmit: () => void
}

export function AutomationSettingsUI({
	isLoading,
	values,
	retrySchedulePreview,
	isPending,
	formError,
	saveError,
	onChange,
	onSubmit,
}: AutomationSettingsUIProps) {
	if (isLoading) {
		return (
			<SectionCard className="flex min-h-40 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
				<Loader2 className="size-5 animate-spin" />
				Chargement…
			</SectionCard>
		)
	}

	return (
		<SectionCard>
			<SectionHeader
				title="Réglages"
				description="Combien de temps garder l’historique, et à quel rythme retenter une étape en échec. Une valeur hors des bornes de l’instance est refusée, jamais corrigée en silence."
			/>
			<form
				className="flex flex-col gap-5 p-5"
				onSubmit={(event) => {
					event.preventDefault()
					onSubmit()
				}}
			>
				<div className="grid grid-cols-1 gap-4 md:grid-cols-2">
					<TextField
						label="Rétention des événements"
						value={values.eventRetentionSeconds}
						onChange={(value) => onChange({ eventRetentionSeconds: value })}
						inputMode="numeric"
						suffix="s"
					/>
					<TextField
						label="Rétention des runs réussis"
						value={values.succeededRunRetentionSeconds}
						onChange={(value) =>
							onChange({ succeededRunRetentionSeconds: value })
						}
						inputMode="numeric"
						suffix="s"
					/>
				</div>

				<div className="flex flex-col gap-2">
					<TextField
						label="Plan de nouvelles tentatives (secondes, séparées par des virgules)"
						value={values.retryScheduleSeconds}
						onChange={(value) => onChange({ retryScheduleSeconds: value })}
						placeholder="5, 30, 120, 600, 3600, 21600"
					/>
					<p className="text-xs text-muted-foreground">
						{retrySchedulePreview.length > 0
							? formatRetrySchedulePreview(retrySchedulePreview)
							: 'Chaque entrée est une tentative — retirer une entrée retire une tentative.'}
					</p>
				</div>

				<TextField
					label="Désactiver après N échecs consécutifs"
					value={values.disableTargetAfter}
					onChange={(value) => onChange({ disableTargetAfter: value })}
					inputMode="numeric"
					placeholder="Laisser vide pour ne jamais désactiver"
				/>

				{formError ? (
					<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
						{formError}
					</p>
				) : null}
				{saveError ? (
					<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
						{saveError}
					</p>
				) : null}

				<div>
					<Button type="submit" disabled={isPending} className="gap-2">
						{isPending ? (
							<Loader2 className="size-4 animate-spin" />
						) : (
							<Save className="size-4" />
						)}
						Enregistrer
					</Button>
				</div>
			</form>
		</SectionCard>
	)
}

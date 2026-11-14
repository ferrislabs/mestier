import { Field } from '#/components/ui/field'
import { Input } from '#/components/ui/input'
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import { Switch } from '#/components/ui/switch'
import { Textarea } from '#/components/ui/textarea'
import { newMetadataEntry } from '#/pages/survey/lib/metadata'
import type {
	SettingsErrors,
	SurveySettings,
} from '#/pages/survey/lib/survey-settings'
import type { Campaign } from '#/pages/survey/types'
import { MetadataEditor } from '#/pages/survey/ui/metadata-editor'
import { NATIVE_SELECT } from '#/pages/survey/ui/node-details-panel'
import { VisibilityPicker } from '#/pages/survey/ui/visibility-picker'

function FieldError({ message }: { message: string | undefined }) {
	return message ? <p className="text-destructive text-xs">{message}</p> : null
}

export function SurveySettingsSheet({
	open,
	settings,
	errors,
	campaigns,
	readOnly,
	visibilityLocked,
	onOpenChange,
	onChange,
}: {
	open: boolean
	settings: SurveySettings
	errors: SettingsErrors
	campaigns: readonly Campaign[]
	readOnly: boolean
	visibilityLocked: boolean
	onOpenChange: (open: boolean) => void
	onChange: (patch: Partial<SurveySettings>) => void
}) {
	return (
		<Sheet open={open} onOpenChange={onOpenChange}>
			<SheetContent className="w-full overflow-y-auto sm:max-w-md">
				<SheetHeader>
					<SheetTitle>Réglages du questionnaire</SheetTitle>
					<SheetDescription>
						{readOnly
							? 'Le questionnaire n’est plus un brouillon : ses réglages sont figés.'
							: 'Enregistrez le questionnaire pour appliquer vos réglages.'}
					</SheetDescription>
				</SheetHeader>
				<div className="grid gap-5 px-4 pb-6">
					<Field label="Titre" htmlFor="settings-title">
						<Input
							id="settings-title"
							value={settings.title}
							disabled={readOnly}
							aria-invalid={Boolean(errors.title)}
							onChange={(event) => onChange({ title: event.target.value })}
						/>
						<FieldError message={errors.title} />
					</Field>
					<Field label="Description" htmlFor="settings-description">
						<Textarea
							id="settings-description"
							value={settings.description}
							disabled={readOnly}
							placeholder="Affichée au répondant avant la première question"
							onChange={(event) =>
								onChange({ description: event.target.value })
							}
						/>
					</Field>
					<Field label="Visibilité" htmlFor={null}>
						<VisibilityPicker
							name="settings-visibility"
							value={settings.visibility}
							disabled={readOnly || visibilityLocked}
							onChange={(visibility) => onChange({ visibility })}
						/>
						<p className="text-muted-foreground text-xs">
							La visibilité ne change plus après l’ouverture.
						</p>
					</Field>
					<div className="flex items-start justify-between gap-3">
						<label htmlFor="settings-multiple" className="text-sm">
							<span className="block font-medium">
								Autoriser plusieurs réponses
							</span>
							<span className="block text-muted-foreground text-xs">
								Par défaut, une même personne ne répond qu’une fois.
							</span>
						</label>
						<Switch
							id="settings-multiple"
							checked={settings.allowMultiple}
							disabled={readOnly}
							onCheckedChange={(allowMultiple) => onChange({ allowMultiple })}
						/>
					</div>
					<Field label="Plafond de réponses" htmlFor="settings-cap">
						<Input
							id="settings-cap"
							type="number"
							min={1}
							placeholder="Sans plafond"
							value={settings.responseCap}
							disabled={readOnly}
							aria-invalid={Boolean(errors.responseCap)}
							onChange={(event) =>
								onChange({ responseCap: event.target.value })
							}
						/>
						<FieldError message={errors.responseCap} />
					</Field>
					<Field label="Campagne" htmlFor="settings-campaign">
						<select
							id="settings-campaign"
							className={NATIVE_SELECT}
							value={settings.campaignId}
							disabled={readOnly}
							onChange={(event) => onChange({ campaignId: event.target.value })}
						>
							<option value="">Aucune : questionnaire indépendant</option>
							{campaigns.map((campaign) => (
								<option key={campaign.id} value={campaign.id}>
									{campaign.name}
								</option>
							))}
						</select>
						<p className="text-muted-foreground text-xs">
							Une campagne fixe la fenêtre pendant laquelle les réponses sont
							acceptées.
						</p>
					</Field>
					<Field label="Métadonnées privées" htmlFor={null}>
						<p className="text-muted-foreground text-xs">
							Jamais montrées au répondant, elles accompagnent les événements
							pour vos automatisations.
						</p>
						<MetadataEditor
							entries={settings.metadata}
							errors={errors.metadata ?? []}
							readOnly={readOnly}
							onChange={(metadata) => onChange({ metadata })}
							onAdd={() =>
								onChange({
									metadata: [...settings.metadata, newMetadataEntry()],
								})
							}
						/>
					</Field>
					<p className="text-muted-foreground text-xs">
						Le message affiché à la fin se règle sur chaque nœud Fin du canevas.
					</p>
				</div>
			</SheetContent>
		</Sheet>
	)
}

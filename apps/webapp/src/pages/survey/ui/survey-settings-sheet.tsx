import { Lock } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import { Checkbox } from '#/components/ui/checkbox'
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
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from '#/components/ui/tooltip'
import { newMetadataEntry } from '#/pages/survey/lib/metadata'
import {
	type SettingsErrors,
	type SurveySettings,
	toggleCampaign,
} from '#/pages/survey/lib/survey-settings'
import type { Campaign } from '#/pages/survey/types'
import { MetadataEditor } from '#/pages/survey/ui/metadata-editor'
import { VisibilityPicker } from '#/pages/survey/ui/visibility-picker'

const VISIBILITY_LOCKED =
	'La visibilité d’un questionnaire ouvert ne peut pas changer : dupliquez-le.'

function FieldError({ message }: { message: string | undefined }) {
	return message ? <p className="text-destructive text-xs">{message}</p> : null
}

function SettingsFields({
	settings,
	errors,
	campaigns,
	readOnly,
	visibilityLocked,
	onChange,
}: {
	settings: SurveySettings
	errors: SettingsErrors
	campaigns: readonly Campaign[]
	readOnly: boolean
	visibilityLocked: boolean
	onChange: (patch: Partial<SurveySettings>) => void
}) {
	return (
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
					onChange={(event) => onChange({ description: event.target.value })}
				/>
			</Field>
			<Field label="Visibilité" htmlFor={null}>
				<div className="flex items-start gap-2">
					<div className="min-w-0 flex-1">
						<VisibilityPicker
							name="settings-visibility"
							value={settings.visibility}
							disabled={readOnly || visibilityLocked}
							onChange={(visibility) => onChange({ visibility })}
						/>
					</div>
					{visibilityLocked ? (
						<Tooltip>
							<TooltipTrigger asChild>
								<Button
									type="button"
									variant="ghost"
									size="icon"
									className="size-11 shrink-0 sm:size-8"
									aria-label="Pourquoi la visibilité est-elle figée ?"
									aria-describedby="settings-visibility-locked"
								>
									<Lock aria-hidden />
								</Button>
							</TooltipTrigger>
							<TooltipContent>{VISIBILITY_LOCKED}</TooltipContent>
						</Tooltip>
					) : null}
				</div>
				<p
					id="settings-visibility-locked"
					className="text-muted-foreground text-xs"
				>
					{visibilityLocked
						? VISIBILITY_LOCKED
						: 'La visibilité ne change plus après l’ouverture.'}
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
					onChange={(event) => onChange({ responseCap: event.target.value })}
				/>
				<FieldError message={errors.responseCap} />
			</Field>
			<fieldset className="m-0 grid min-w-0 gap-2 border-0 p-0">
				<legend className="mb-2 font-medium text-sm">Campagnes</legend>
				{campaigns.length === 0 ? (
					<p className="text-muted-foreground text-xs">
						Aucune campagne : le questionnaire est indépendant.
					</p>
				) : (
					<ul className="grid gap-1">
						{campaigns.map((campaign) => {
							const id = `settings-campaign-${campaign.id}`
							return (
								<li key={campaign.id} className="flex items-center gap-2">
									<Checkbox
										id={id}
										checked={settings.campaignIds.includes(campaign.id)}
										disabled={readOnly}
										onCheckedChange={(checked) =>
											onChange({
												campaignIds: toggleCampaign(
													settings,
													campaign.id,
													checked === true,
												),
											})
										}
									/>
									<label
										htmlFor={id}
										className="min-h-11 content-center text-sm sm:min-h-8"
									>
										{campaign.name}
									</label>
								</li>
							)
						})}
					</ul>
				)}
				<p className="text-muted-foreground text-xs">
					Chaque campagne ouvre une fenêtre pendant laquelle les réponses sont
					acceptées. Sans campagne, le questionnaire est indépendant.
				</p>
			</fieldset>
			<Field label="Métadonnées privées" htmlFor={null}>
				<p className="text-muted-foreground text-xs">
					Jamais montrées au répondant, elles accompagnent les événements pour
					vos automatisations.
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
		</div>
	)
}

const readOnlyHint = (readOnly: boolean) =>
	readOnly
		? 'Vous n’avez pas la permission de modifier ces réglages.'
		: 'Enregistrez le questionnaire pour appliquer vos réglages.'

export function SurveySettingsPanel({
	top,
	...props
}: {
	top?: ReactNode
	settings: SurveySettings
	errors: SettingsErrors
	campaigns: readonly Campaign[]
	readOnly: boolean
	visibilityLocked: boolean
	onChange: (patch: Partial<SurveySettings>) => void
}) {
	return (
		<aside
			aria-label="Réglages du questionnaire"
			className="w-96 shrink-0 overflow-y-auto border-l bg-card"
		>
			{top}
			<div className="grid gap-1 p-4 pb-3">
				<h2 className="font-semibold text-base">Réglages du questionnaire</h2>
				<p className="text-muted-foreground text-sm">
					{readOnlyHint(props.readOnly)}
				</p>
			</div>
			<SettingsFields {...props} />
		</aside>
	)
}

export function SurveySettingsSheet({
	open,
	onOpenChange,
	...fields
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
					<SheetDescription>{readOnlyHint(fields.readOnly)}</SheetDescription>
				</SheetHeader>
				<SettingsFields {...fields} />
			</SheetContent>
		</Sheet>
	)
}

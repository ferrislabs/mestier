import { Loader2, Save } from 'lucide-react'
import { useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type DoeSettings,
	useDoeSettings,
	useUpdateDoeSettings,
} from '#/hooks/use-doe-models'
import { mutationErrorMessage } from '#/lib/api-error'
import { DoeStyleEditorFeature } from '#/pages/doe-style/feature/doe-style-editor-feature'
import {
	type CompleteDoeStyle,
	completeStyle,
	isStyleValid,
	normalizeStyle,
	sameStyle,
} from '#/pages/doe-style/lib/style'

export function DocumentsSection() {
	const { activeOrganization } = useActiveOrganization()
	const settings = useDoeSettings(activeOrganization.id)

	return (
		<div className="flex flex-col gap-6">
			<SectionCard>
				<SectionHeader
					title="Mise en page des documents"
					description="Couleurs, polices, couverture, en-tête, pied de page et libellés des documents PDF, dont le DOE. Un modèle de DOE peut les remplacer."
				/>
				{settings.isLoading ? (
					<div className="flex items-center gap-2 p-5 text-sm text-muted-foreground">
						<Loader2 className="size-4 animate-spin" />
						Chargement…
					</div>
				) : settings.data?.data ? (
					<DocumentsForm
						key={`${activeOrganization.id}-${JSON.stringify(settings.data.data.style)}`}
						organizationId={activeOrganization.id}
						settings={settings.data.data}
					/>
				) : (
					<p className="p-5 text-sm text-destructive">
						{mutationErrorMessage(settings.error) ?? 'Réglages indisponibles.'}
					</p>
				)}
			</SectionCard>
		</div>
	)
}

function DocumentsForm({
	organizationId,
	settings,
}: {
	organizationId: string
	settings: DoeSettings
}) {
	const saved = completeStyle(settings.style)
	const [style, setStyle] = useState<CompleteDoeStyle>(saved)
	const update = useUpdateDoeSettings()

	const valid = isStyleValid(style)
	const changed = !sameStyle(style, saved)

	const save = () =>
		update.mutateAsync({
			path: { organization_id: organizationId },
			body: { style: normalizeStyle(style) },
		})

	return (
		<div className="flex flex-col gap-6 p-5">
			<DoeStyleEditorFeature
				organizationId={organizationId}
				value={style}
				onChange={(patch) => setStyle((current) => ({ ...current, ...patch }))}
			/>

			{update.error ? (
				<p className="text-sm text-destructive">
					{mutationErrorMessage(update.error)}
				</p>
			) : null}
			{update.isSuccess && !changed ? (
				<output className="text-sm text-success">Enregistré</output>
			) : null}

			<div>
				<RequirePermission permission="MANAGE_REFERENCE">
					<Button
						type="button"
						disabled={!valid || !changed || update.isPending}
						onClick={() => {
							void save().catch(() => {})
						}}
					>
						{update.isPending ? <Loader2 className="animate-spin" /> : <Save />}
						Enregistrer
					</Button>
				</RequirePermission>
			</div>
		</div>
	)
}

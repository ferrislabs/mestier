import { type FormEvent, type ReactNode, useId, useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	type CompleteDoeStyle,
	isStyleValid,
	sameStyle,
} from '#/pages/doe-style/lib/style'
import { saveStatus } from '#/pages/projects/lib/save-status'
import { SettingsSection } from '#/pages/projects/ui/settings-section'
import { useReportDirty } from '#/pages/projects/ui/unsaved-guard'

const SAVED_LABEL =
	'Style enregistré. Le DOE du projet l’utilisera à son prochain aperçu PDF.'

export interface StyleCardProps {
	draft: CompleteDoeStyle
	stored: CompleteDoeStyle
	hasOwnStyle: boolean
	editor: ReactNode
	isSaving: boolean
	error: string | null
	onReplace: (style: CompleteDoeStyle) => void
	onSubmit: () => Promise<void>
	onClear: () => Promise<void>
	onDirtyChange: (dirty: boolean) => void
}

export function StyleCard({
	draft,
	stored,
	hasOwnStyle,
	editor,
	isSaving,
	error,
	onReplace,
	onSubmit,
	onClear,
	onDirtyChange,
}: StyleCardProps) {
	const baseId = useId()
	const [saved, setSaved] = useState(false)
	const dirty = !sameStyle(draft, stored)
	const canSave = dirty && isStyleValid(draft) && !isSaving
	useReportDirty(dirty, onDirtyChange)

	function submit(event: FormEvent) {
		event.preventDefault()
		if (!canSave) return
		void onSubmit()
			.then(() => setSaved(true))
			.catch(() => {})
	}

	return (
		<SettingsSection
			id={`${baseId}-style`}
			title="Mise en page du DOE de ce projet"
			description="Couleurs, polices, couverture, en-tête, pied de page, libellés et éléments du PDF, pour ce projet seulement. Sans style propre, le DOE suit son modèle, sinon l’organisation."
		>
			<form onSubmit={submit} noValidate>
				<div className="px-5 py-5">{editor}</div>
				{error ? (
					<p role="alert" className="px-5 pb-3 text-[13px] text-state-alert">
						{error}
					</p>
				) : null}
				<div className="flex flex-wrap items-center justify-between gap-3 border-t px-5 py-3">
					<output className="text-[13px] text-muted-foreground">
						{saveStatus({
							dirty,
							saved: saved && !dirty,
							savedLabel: SAVED_LABEL,
						})}
					</output>
					<div className="flex flex-wrap items-center gap-2">
						<Button
							type="button"
							variant="ghost"
							size="sm"
							disabled={!hasOwnStyle || isSaving}
							onClick={() => {
								setSaved(false)
								void onClear().catch(() => {})
							}}
						>
							Revenir au style de l’organisation
						</Button>
						<Button
							type="button"
							variant="outline"
							size="sm"
							disabled={!dirty}
							onClick={() => {
								setSaved(false)
								onReplace(stored)
							}}
						>
							Annuler les modifications
						</Button>
						<Button type="submit" size="sm" disabled={!canSave}>
							Enregistrer
						</Button>
					</div>
				</div>
			</form>
		</SettingsSection>
	)
}

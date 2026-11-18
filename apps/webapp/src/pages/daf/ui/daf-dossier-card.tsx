import { Loader2, Save } from 'lucide-react'
import { useId } from 'react'
import { FormField } from '#/components/form-field'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import type { DafDossierValues } from '#/pages/daf/lib/dossier-form'
import { EditorCard } from '#/pages/doe/ui/editor-card'

export function DafDossierCard({
	values,
	isStored,
	isDirty,
	isSaving,
	error,
	onChange,
	onSave,
}: {
	values: DafDossierValues
	isStored: boolean
	isDirty: boolean
	isSaving: boolean
	error: string | null
	onChange: (patch: Partial<DafDossierValues>) => void
	onSave: () => void
}) {
	const base = useId()
	const id = (field: string) => `${base}-${field}`
	const canSave =
		isDirty || !isStored
			? values.title.trim() !== '' && values.issuer.trim() !== ''
			: false

	return (
		<EditorCard
			title="Dossier"
			description="L’en-tête commun à toutes les DAF du projet."
		>
			<div className="grid gap-4 p-5 md:grid-cols-2">
				<FormField id={id('title')} label="Titre" required>
					<Input
						id={id('title')}
						value={values.title}
						onChange={(event) => onChange({ title: event.target.value })}
					/>
				</FormField>
				<FormField id={id('issuer')} label="Émetteur" required>
					<Input
						id={id('issuer')}
						value={values.issuer}
						onChange={(event) => onChange({ issuer: event.target.value })}
					/>
				</FormField>
				<FormField id={id('owner')} label="Maîtrise d’ouvrage">
					<Input
						id={id('owner')}
						value={values.projectOwner}
						onChange={(event) => onChange({ projectOwner: event.target.value })}
					/>
				</FormField>
				<FormField id={id('manager')} label="Maîtrise d’œuvre">
					<Input
						id={id('manager')}
						value={values.projectManager}
						onChange={(event) =>
							onChange({ projectManager: event.target.value })
						}
					/>
				</FormField>
			</div>
			<RequirePermission permission="MANAGE_PLANNING">
				<div className="flex flex-wrap items-center gap-3 border-t px-5 py-3">
					<Button
						type="button"
						size="sm"
						disabled={!canSave || isSaving}
						onClick={onSave}
					>
						{isSaving ? <Loader2 className="animate-spin" /> : <Save />}
						Enregistrer le dossier
					</Button>
					{isStored ? null : (
						<p className="text-[13px] text-muted-foreground">
							Pré-rempli, pas encore enregistré.
						</p>
					)}
					{error ? (
						<p role="alert" className="text-[13px] text-destructive">
							{error}
						</p>
					) : null}
				</div>
			</RequirePermission>
		</EditorCard>
	)
}

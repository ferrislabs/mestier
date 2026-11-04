import { Loader2, Save } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Label } from '#/components/ui/label'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import { Textarea } from '#/components/ui/textarea'

const FIELD_ID = 'support-display-fields'
const HELPER_ID = 'support-display-fields-helper'

export interface SupportSettingsUIProps {
	isLoading: boolean
	value: string
	canManage: boolean
	isPending: boolean
	formError: string | null
	saveError: string | null
	onChange: (value: string) => void
	onSubmit: () => void
}

export function SupportSettingsUI({
	isLoading,
	value,
	canManage,
	isPending,
	formError,
	saveError,
	onChange,
	onSubmit,
}: SupportSettingsUIProps) {
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
				title="Champs affichés"
				description="Ces clés de champs personnalisés, dans cet ordre, s’affichent sur les cartes des tickets et en colonnes du tableau."
			/>
			<form
				className="flex flex-col gap-5 p-5"
				onSubmit={(event) => {
					event.preventDefault()
					onSubmit()
				}}
			>
				<div className="flex flex-col gap-2">
					<Label htmlFor={FIELD_ID}>Clés des champs personnalisés</Label>
					<Textarea
						id={FIELD_ID}
						aria-describedby={HELPER_ID}
						rows={8}
						value={value}
						readOnly={!canManage}
						onChange={(event) => onChange(event.target.value)}
					/>
					<p id={HELPER_ID} className="text-xs text-muted-foreground">
						Une clé par ligne, huit au plus. L’ordre des lignes est l’ordre
						d’affichage.
					</p>
					{canManage ? null : (
						<p className="text-xs text-muted-foreground">
							Seuls les membres autorisés à traiter les tickets peuvent modifier
							ce réglage.
						</p>
					)}
				</div>

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

				{canManage ? (
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
				) : null}
			</form>
		</SectionCard>
	)
}

import { Loader2, Save, X } from 'lucide-react'
import { useState } from 'react'
import { FileUploadButton } from '#/components/file-upload-button'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useUploadFile } from '#/hooks/use-customers'
import {
	type DoeSettings,
	useDoeSettings,
	useUpdateDoeSettings,
} from '#/hooks/use-doe-models'
import { useFileUrls } from '#/hooks/use-file-url'
import { mutationErrorMessage } from '#/lib/api-error'
import { cn } from '#/lib/utils'

export const ACCENT_SWATCHES = ['#0f3d36', '#6b1460', '#1d4e89', '#8a3b12']

const ACCENT_PATTERN = /^#[0-9a-f]{6}$/i

export function isValidAccent(value: string): boolean {
	return ACCENT_PATTERN.test(value.trim())
}

export function DocumentsSection() {
	const { activeOrganization } = useActiveOrganization()
	const settings = useDoeSettings(activeOrganization.id)

	return (
		<div className="flex flex-col gap-6">
			<SectionCard>
				<SectionHeader
					title="Mise en page des documents"
					description="La couleur d'accent et le logo des documents PDF, dont le DOE."
				/>
				{settings.isLoading ? (
					<div className="flex items-center gap-2 p-5 text-sm text-muted-foreground">
						<Loader2 className="size-4 animate-spin" />
						Chargement…
					</div>
				) : settings.data?.data ? (
					<DocumentsForm
						key={`${activeOrganization.id}-${settings.data.data.accent}-${settings.data.data.logo_key ?? ''}`}
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
	const [accent, setAccent] = useState(settings.accent)
	const [logoKey, setLogoKey] = useState<string | null>(
		settings.logo_key ?? null,
	)
	const update = useUpdateDoeSettings()
	const upload = useUploadFile()
	const [logoPreview] = useFileUrls(logoKey ? [logoKey] : [])

	const valid = isValidAccent(accent)
	const changed =
		accent.trim().toLowerCase() !== settings.accent.toLowerCase() ||
		logoKey !== (settings.logo_key ?? null)

	const save = () =>
		update.mutateAsync({
			path: { organization_id: organizationId },
			body: { accent: accent.trim().toLowerCase(), logo_key: logoKey },
		})

	return (
		<div className="flex flex-col gap-6 p-5">
			<div className="flex flex-col gap-3">
				<Label>Couleur d'accent</Label>
				<div className="flex flex-wrap items-center gap-3">
					{ACCENT_SWATCHES.map((swatch) => (
						<button
							key={swatch}
							type="button"
							aria-label={`Couleur ${swatch}`}
							aria-pressed={accent.toLowerCase() === swatch}
							onClick={() => setAccent(swatch)}
							className={cn(
								'size-8 rounded-full border-2 transition',
								accent.toLowerCase() === swatch
									? 'border-foreground'
									: 'border-transparent',
							)}
							style={{ backgroundColor: swatch }}
						/>
					))}
					<div className="flex flex-col gap-1">
						<Input
							value={accent}
							aria-label="Couleur personnalisée"
							aria-invalid={!valid}
							className="w-32 font-mono"
							onChange={(event) => setAccent(event.target.value)}
						/>
					</div>
					<span
						className="size-8 rounded border"
						style={{ backgroundColor: valid ? accent.trim() : 'transparent' }}
					/>
				</div>
				{valid ? null : (
					<p role="alert" className="text-xs text-destructive">
						Format attendu : #rrggbb, par exemple #0f3d36.
					</p>
				)}
			</div>

			<div className="flex flex-col gap-3">
				<Label>Logo</Label>
				<div className="flex items-center gap-3">
					{logoKey ? (
						logoPreview?.url ? (
							<img
								src={logoPreview.url}
								alt="Logo de l'organisation"
								className="h-12 max-w-40 rounded border object-contain"
							/>
						) : (
							<span className="text-sm text-muted-foreground">Logo envoyé</span>
						)
					) : (
						<span className="text-sm text-muted-foreground">Aucun logo</span>
					)}
					<RequirePermission permission="MANAGE_REFERENCE">
						<FileUploadButton
							accept="image/*"
							isUploading={upload.isPending}
							onFiles={(files) => {
								const file = files[0]
								if (!file) return
								void upload
									.mutateAsync(file)
									.then((response) => setLogoKey(response.data.key))
									.catch(() => {})
							}}
						>
							Choisir un logo
						</FileUploadButton>
						{logoKey ? (
							<Button
								type="button"
								variant="ghost"
								size="sm"
								onClick={() => setLogoKey(null)}
							>
								<X />
								Retirer le logo
							</Button>
						) : null}
					</RequirePermission>
				</div>
			</div>

			{update.error || upload.error ? (
				<p className="text-sm text-destructive">
					{mutationErrorMessage(update.error) ??
						mutationErrorMessage(upload.error)}
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

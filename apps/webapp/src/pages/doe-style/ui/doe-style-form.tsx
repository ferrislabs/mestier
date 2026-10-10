import { Eye, Loader2, X } from 'lucide-react'
import { FileUploadButton } from '#/components/file-upload-button'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Switch } from '#/components/ui/switch'
import {
	type CompleteDoeStyle,
	type DoeFont,
	FONT_LABELS,
	FONTS,
	isValidHex,
	type LogoPosition,
	type LogoSize,
	type StyleErrors,
	TEMPLATE_VARIABLES,
} from '#/pages/doe-style/lib/style'

interface DoeStyleFormProps {
	value: CompleteDoeStyle
	errors: StyleErrors
	logoUrl: string | undefined
	isUploadingLogo: boolean
	previewing: boolean
	previewDisabled: boolean
	onChange: (patch: Partial<CompleteDoeStyle>) => void
	onPreview: () => void
	onLogoUpload: (file: File) => void
	onLogoRemove: () => void
}

const POSITION_LABELS: Record<LogoPosition, string> = {
	left: 'À gauche',
	center: 'Au centre',
	right: 'À droite',
}

const SIZE_LABELS: Record<LogoSize, string> = {
	small: 'Petit',
	medium: 'Moyen',
	large: 'Grand',
}

export function DoeStyleForm({
	value,
	errors,
	logoUrl,
	isUploadingLogo,
	previewing,
	previewDisabled,
	onChange,
	onPreview,
	onLogoUpload,
	onLogoRemove,
}: DoeStyleFormProps) {
	return (
		<div className="flex flex-col gap-6">
			<Group title="Couleurs">
				<div className="grid gap-4 md:grid-cols-3">
					<ColorField
						label="Couleur principale"
						value={value.primary}
						error={errors.primary}
						onChange={(primary) => onChange({ primary })}
					/>
					<ColorField
						label="Couleur secondaire"
						value={value.secondary}
						error={errors.secondary}
						onChange={(secondary) => onChange({ secondary })}
					/>
					<ColorField
						label="Texte sur les bandeaux"
						value={value.on_primary}
						error={errors.on_primary}
						onChange={(on_primary) => onChange({ on_primary })}
					/>
				</div>
			</Group>

			<Group title="Polices">
				<div className="grid gap-4 md:grid-cols-2">
					<FontField
						label="Police des titres"
						value={value.heading_font}
						onChange={(heading_font) => onChange({ heading_font })}
					/>
					<FontField
						label="Police du texte"
						value={value.body_font}
						onChange={(body_font) => onChange({ body_font })}
					/>
				</div>
			</Group>

			<Group title="Couverture">
				<div className="flex flex-col gap-3">
					<Label>Logo</Label>
					<div className="flex flex-wrap items-center gap-3">
						{value.logo_key ? (
							logoUrl ? (
								<img
									src={logoUrl}
									alt="Logo de l'organisation"
									className="h-12 max-w-40 rounded border object-contain"
								/>
							) : (
								<span className="text-sm text-muted-foreground">
									Logo envoyé
								</span>
							)
						) : (
							<span className="text-sm text-muted-foreground">Aucun logo</span>
						)}
						<RequirePermission permission="MANAGE_REFERENCE">
							<FileUploadButton
								accept="image/*"
								isUploading={isUploadingLogo}
								onFiles={(files) => {
									const file = files[0]
									if (file) onLogoUpload(file)
								}}
							>
								Choisir un logo
							</FileUploadButton>
							{value.logo_key ? (
								<Button
									type="button"
									variant="ghost"
									size="sm"
									onClick={onLogoRemove}
								>
									<X />
									Retirer le logo
								</Button>
							) : null}
						</RequirePermission>
					</div>
				</div>
				<div className="grid gap-4 md:grid-cols-2">
					<div className="flex flex-col gap-2">
						<Label>Position du logo</Label>
						<Select
							value={value.cover_logo_position}
							onValueChange={(next) =>
								onChange({ cover_logo_position: next as LogoPosition })
							}
						>
							<SelectTrigger aria-label="Position du logo" className="w-full">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{(Object.keys(POSITION_LABELS) as LogoPosition[]).map((key) => (
									<SelectItem key={key} value={key}>
										{POSITION_LABELS[key]}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</div>
					<div className="flex flex-col gap-2">
						<Label>Taille du logo</Label>
						<Select
							value={value.cover_logo_size}
							onValueChange={(next) =>
								onChange({ cover_logo_size: next as LogoSize })
							}
						>
							<SelectTrigger aria-label="Taille du logo" className="w-full">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{(Object.keys(SIZE_LABELS) as LogoSize[]).map((key) => (
									<SelectItem key={key} value={key}>
										{SIZE_LABELS[key]}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</div>
				</div>
				<SwitchField
					label="Afficher la photo de couverture du DOE"
					checked={value.cover_photo}
					onChange={(cover_photo) => onChange({ cover_photo })}
				/>
			</Group>

			<Group title="En-tête et pied de page">
				<div className="grid gap-4 md:grid-cols-2">
					<TextInput
						label="Texte d'en-tête"
						value={value.header_text}
						error={errors.header_text}
						onChange={(header_text) => onChange({ header_text })}
					/>
					<TextInput
						label="Texte de pied de page"
						value={value.footer_text}
						error={errors.footer_text}
						onChange={(footer_text) => onChange({ footer_text })}
					/>
				</div>
				<p className="text-xs text-muted-foreground">
					Variables permises : {TEMPLATE_VARIABLES.join(', ')}. Un champ vide
					n'affiche rien. L'indice et le numéro de page restent à droite.
				</p>
				<SwitchField
					label="Logo dans l'en-tête"
					checked={value.header_logo}
					onChange={(header_logo) => onChange({ header_logo })}
				/>
			</Group>

			<Group title="Libellés">
				<div className="grid gap-4 md:grid-cols-2">
					<TextInput
						label="Titre de couverture"
						value={value.cover_title}
						error={errors.cover_title}
						onChange={(cover_title) => onChange({ cover_title })}
					/>
					<TextInput
						label="Titre des intervenants"
						value={value.parties_title}
						error={errors.parties_title}
						onChange={(parties_title) => onChange({ parties_title })}
					/>
					<TextInput
						label="Titre du sommaire"
						value={value.contents_title}
						error={errors.contents_title}
						onChange={(contents_title) => onChange({ contents_title })}
					/>
					<TextInput
						label="Titre du tableau de fourniture"
						value={value.supply_table_title}
						error={errors.supply_table_title}
						onChange={(supply_table_title) => onChange({ supply_table_title })}
					/>
				</div>
			</Group>

			<div>
				<RequirePermission permission="MANAGE_REFERENCE">
					<Button
						type="button"
						variant="outline"
						disabled={previewDisabled || previewing}
						onClick={onPreview}
					>
						{previewing ? <Loader2 className="animate-spin" /> : <Eye />}
						Aperçu PDF
					</Button>
				</RequirePermission>
			</div>
		</div>
	)
}

function Group({
	title,
	children,
}: {
	title: string
	children: React.ReactNode
}) {
	return (
		<fieldset className="flex flex-col gap-4 rounded-lg border p-4">
			<legend className="px-1 text-sm font-medium">{title}</legend>
			{children}
		</fieldset>
	)
}

function ColorField({
	label,
	value,
	error,
	onChange,
}: {
	label: string
	value: string
	error: string | undefined
	onChange: (value: string) => void
}) {
	const valid = isValidHex(value)
	return (
		<div className="flex flex-col gap-2">
			<Label>{label}</Label>
			<div className="flex items-center gap-2">
				<input
					type="color"
					aria-label={`${label} (sélecteur)`}
					value={valid ? value.trim().toLowerCase() : '#000000'}
					className="h-9 w-10 cursor-pointer rounded border bg-transparent p-0.5"
					onChange={(event) => onChange(event.target.value)}
				/>
				<Input
					value={value}
					aria-label={label}
					aria-invalid={Boolean(error)}
					className="w-32 font-mono"
					onChange={(event) => onChange(event.target.value)}
				/>
			</div>
			{error ? (
				<p role="alert" className="text-xs text-destructive">
					{error}
				</p>
			) : null}
		</div>
	)
}

function FontField({
	label,
	value,
	onChange,
}: {
	label: string
	value: DoeFont
	onChange: (value: DoeFont) => void
}) {
	return (
		<div className="flex flex-col gap-2">
			<Label>{label}</Label>
			<Select value={value} onValueChange={(next) => onChange(next as DoeFont)}>
				<SelectTrigger aria-label={label} className="w-full">
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					{FONTS.map((font) => (
						<SelectItem key={font} value={font}>
							{FONT_LABELS[font]}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
		</div>
	)
}

function TextInput({
	label,
	value,
	error,
	onChange,
}: {
	label: string
	value: string
	error: string | undefined
	onChange: (value: string) => void
}) {
	return (
		<div className="flex flex-col gap-2">
			<Label>{label}</Label>
			<Input
				value={value}
				aria-label={label}
				aria-invalid={Boolean(error)}
				onChange={(event) => onChange(event.target.value)}
			/>
			{error ? (
				<p role="alert" className="text-xs text-destructive">
					{error}
				</p>
			) : null}
		</div>
	)
}

function SwitchField({
	label,
	checked,
	onChange,
}: {
	label: string
	checked: boolean
	onChange: (checked: boolean) => void
}) {
	return (
		<div className="flex items-center gap-3">
			<Switch aria-label={label} checked={checked} onCheckedChange={onChange} />
			<span className="text-sm">{label}</span>
		</div>
	)
}

import { FlipHorizontal2, FlipVertical2, Library, X } from 'lucide-react'
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
import { Toggle } from '#/components/ui/toggle'
import {
	type Decoration,
	type DecorationLayer,
	type DecorationPages,
	LAYER_LABELS,
	layerOf,
	PAGES_LABELS,
	pagesOf,
} from '#/pages/doe-style/lib/decorations'
import { normalizeRotation, rotationOf } from '#/pages/doe-style/lib/transform'
import { ColorField } from '#/pages/doe-style/ui/color-field'

interface DecorationPropertiesProps {
	decoration: Decoration
	index: number
	imageUrl: string | undefined
	isUploadingImage: boolean
	onChange: (next: Decoration) => void
	onImageUpload: (file: File) => void
	onPickFromLibrary?: () => void
}

const FIT_LABELS = { contain: 'Contenir', cover: 'Couvrir' } as const

export function DecorationProperties({
	decoration,
	index,
	imageUrl,
	isUploadingImage,
	onChange,
	onImageUpload,
	onPickFromLibrary,
}: DecorationPropertiesProps) {
	return (
		<div className="grid gap-4">
			<div className="grid grid-cols-2 gap-4">
				<MmField
					label="X (mm)"
					value={decoration.x}
					onChange={(x) => onChange({ ...decoration, x })}
				/>
				<MmField
					label="Y (mm)"
					value={decoration.y}
					onChange={(y) => onChange({ ...decoration, y })}
				/>
				<MmField
					label="Largeur (mm)"
					value={decoration.width}
					onChange={(width) => onChange({ ...decoration, width })}
				/>
				<MmField
					label="Hauteur (mm)"
					value={decoration.height}
					onChange={(height) => onChange({ ...decoration, height })}
				/>
			</div>

			<div className="grid grid-cols-[1fr_auto] items-end gap-4">
				<div className="flex flex-col gap-2">
					<Label>Angle (°)</Label>
					<Input
						type="number"
						inputMode="numeric"
						step={1}
						aria-label="Angle (°)"
						value={rotationOf(decoration)}
						onChange={(event) => {
							if (event.target.value === '') return
							onChange({
								...decoration,
								rotation: normalizeRotation(Number(event.target.value)),
							})
						}}
					/>
				</div>
				<div className="flex gap-1">
					<Toggle
						variant="outline"
						aria-label="Miroir horizontal"
						title="Miroir horizontal"
						pressed={Boolean(decoration.flip_x)}
						onPressedChange={(flip_x) => onChange({ ...decoration, flip_x })}
					>
						<FlipHorizontal2 aria-hidden />
					</Toggle>
					<Toggle
						variant="outline"
						aria-label="Miroir vertical"
						title="Miroir vertical"
						pressed={Boolean(decoration.flip_y)}
						onPressedChange={(flip_y) => onChange({ ...decoration, flip_y })}
					>
						<FlipVertical2 aria-hidden />
					</Toggle>
				</div>
			</div>

			<div className="grid gap-4">
				<div className="flex flex-col gap-2">
					<Label>Pages concernées</Label>
					<Select
						value={pagesOf(decoration)}
						onValueChange={(pages) =>
							onChange({ ...decoration, pages: pages as DecorationPages })
						}
					>
						<SelectTrigger aria-label="Pages concernées" className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{(Object.keys(PAGES_LABELS) as DecorationPages[]).map((key) => (
								<SelectItem key={key} value={key}>
									{PAGES_LABELS[key]}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</div>
				<div className="flex flex-col gap-2">
					<Label>Couche</Label>
					<Select
						value={layerOf(decoration)}
						onValueChange={(layer) =>
							onChange({ ...decoration, layer: layer as DecorationLayer })
						}
					>
						<SelectTrigger aria-label="Couche" className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{(Object.keys(LAYER_LABELS) as DecorationLayer[]).map((key) => (
								<SelectItem key={key} value={key}>
									{LAYER_LABELS[key]}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</div>
			</div>

			{decoration.kind === 'rectangle' ? (
				<div className="grid gap-4">
					<ColorField
						label="Couleur"
						value={decoration.fill}
						error={undefined}
						onChange={(fill) => onChange({ ...decoration, fill })}
					/>
					<OpacityField
						value={decoration.opacity}
						onChange={(opacity) => onChange({ ...decoration, opacity })}
					/>
					<MmField
						label="Rayon des angles (mm)"
						value={decoration.radius ?? 0}
						onChange={(radius) => onChange({ ...decoration, radius })}
					/>
				</div>
			) : (
				<div className="grid gap-4">
					<div className="flex flex-wrap items-center gap-3">
						{decoration.key ? (
							imageUrl ? (
								<img
									src={imageUrl}
									alt={`Visuel de l'élément ${index + 1}`}
									className="h-16 max-w-32 rounded border object-contain"
								/>
							) : (
								<span className="text-sm text-muted-foreground">
									Image envoyée
								</span>
							)
						) : (
							<span className="text-sm text-muted-foreground">
								Aucune image
							</span>
						)}
					</div>
					<div className="flex flex-wrap items-center gap-2">
						<RequirePermission permission="MANAGE_REFERENCE">
							<FileUploadButton
								accept="image/*"
								isUploading={isUploadingImage}
								onFiles={(files) => {
									const file = files[0]
									if (file) onImageUpload(file)
								}}
							>
								{decoration.key ? "Remplacer l'image" : 'Choisir une image'}
							</FileUploadButton>
						</RequirePermission>
						{onPickFromLibrary ? (
							<RequirePermission permission="MANAGE_PLANNING">
								<Button
									type="button"
									variant="outline"
									size="sm"
									onClick={onPickFromLibrary}
								>
									<Library aria-hidden />
									Depuis la photothèque
								</Button>
							</RequirePermission>
						) : null}
						{decoration.key ? (
							<RequirePermission permission="MANAGE_REFERENCE">
								<Button
									type="button"
									variant="ghost"
									size="sm"
									onClick={() => onChange({ ...decoration, key: '' })}
								>
									<X />
									Retirer l'image
								</Button>
							</RequirePermission>
						) : null}
					</div>
					<div className="flex flex-col gap-2">
						<Label>Ajustement</Label>
						<Select
							value={decoration.fit ?? 'contain'}
							onValueChange={(fit) =>
								onChange({ ...decoration, fit: fit as 'contain' | 'cover' })
							}
						>
							<SelectTrigger aria-label="Ajustement" className="w-full">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{(
									Object.keys(FIT_LABELS) as Array<keyof typeof FIT_LABELS>
								).map((key) => (
									<SelectItem key={key} value={key}>
										{FIT_LABELS[key]}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</div>
					<OpacityField
						value={decoration.opacity}
						onChange={(opacity) => onChange({ ...decoration, opacity })}
					/>
					<p className="text-xs text-muted-foreground">
						L'opacité d'une image n'agit que derrière le contenu (elle est
						rendue par un voile blanc). Devant le contenu, l'image reste
						pleinement opaque.
					</p>
				</div>
			)}
		</div>
	)
}

function MmField({
	label,
	value,
	onChange,
}: {
	label: string
	value: number
	onChange: (value: number) => void
}) {
	return (
		<div className="flex flex-col gap-2">
			<Label>{label}</Label>
			<Input
				type="number"
				inputMode="decimal"
				step={0.5}
				min={0}
				aria-label={label}
				value={Number.isFinite(value) ? value : ''}
				onChange={(event) => {
					if (event.target.value !== '') onChange(Number(event.target.value))
				}}
			/>
		</div>
	)
}

function OpacityField({
	value,
	onChange,
}: {
	value: number | undefined
	onChange: (value: number) => void
}) {
	return (
		<div className="flex flex-col gap-2">
			<Label>Opacité (%)</Label>
			<Input
				type="number"
				inputMode="numeric"
				step={5}
				min={0}
				max={100}
				aria-label="Opacité (%)"
				value={Math.round((value ?? 1) * 100)}
				onChange={(event) => {
					if (event.target.value !== '') {
						onChange(Number(event.target.value) / 100)
					}
				}}
			/>
		</div>
	)
}

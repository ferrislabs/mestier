import {
	ArrowDown,
	ArrowUp,
	Copy,
	Image as ImageIcon,
	Minus,
	Square,
	Trash2,
	X,
} from 'lucide-react'
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
import { cn } from '#/lib/utils'
import {
	addDecoration,
	type Decoration,
	type DecorationKind,
	type DecorationLayer,
	type DecorationPages,
	decorationLabel,
	duplicateDecoration,
	LAYER_LABELS,
	layerOf,
	MAX_DECORATIONS,
	PAGES_LABELS,
	pagesOf,
	removeDecoration,
	reorderDecoration,
	reorderedIndex,
	updateDecoration,
} from '#/pages/doe-style/lib/decorations'
import { ColorField } from '#/pages/doe-style/ui/color-field'

interface DoeDecorationsSectionProps {
	decorations: Decoration[]
	selectedIndex: number | null
	errors: Array<string | undefined>
	accentColor: string
	imageUrl: string | undefined
	isUploadingImage: boolean
	onChange: (decorations: Decoration[]) => void
	onSelect: (index: number | null) => void
	onImageUpload: (index: number, file: File) => void
}

const FIT_LABELS = { contain: 'Contenir', cover: 'Couvrir' } as const

const ADD_BUTTONS: Array<{
	kind: DecorationKind
	label: string
	icon: typeof Square
}> = [
	{ kind: 'image', label: 'Ajouter une image', icon: ImageIcon },
	{ kind: 'rectangle', label: 'Ajouter un rectangle', icon: Square },
	{ kind: 'line', label: 'Ajouter une ligne', icon: Minus },
]

export function DoeDecorationsSection({
	decorations,
	selectedIndex,
	errors,
	accentColor,
	imageUrl,
	isUploadingImage,
	onChange,
	onSelect,
	onImageUpload,
}: DoeDecorationsSectionProps) {
	const full = decorations.length >= MAX_DECORATIONS
	const selected =
		selectedIndex === null ? undefined : decorations[selectedIndex]
	const order = decorations.map((_, index) => index).reverse()

	return (
		<div className="grid gap-4">
			<p className="text-xs text-muted-foreground">
				Images, rectangles et lignes répétés sur les pages du DOE. Glissez-les
				sur l'aperçu, ou ajustez-les aux flèches du clavier.{' '}
				{decorations.length} sur {MAX_DECORATIONS}.
			</p>
			<div className="flex flex-wrap gap-2">
				{ADD_BUTTONS.map(({ kind, label, icon: Icon }) => (
					<Button
						key={kind}
						type="button"
						variant="outline"
						size="sm"
						disabled={full}
						onClick={() => {
							onChange(addDecoration(decorations, kind, accentColor))
							onSelect(decorations.length)
						}}
					>
						<Icon aria-hidden />
						{label}
					</Button>
				))}
			</div>

			{decorations.length === 0 ? (
				<p className="text-sm text-muted-foreground">Aucun élément.</p>
			) : (
				<ul aria-label="Éléments, du dessus au dessous" className="grid gap-1">
					{order.map((index) => {
						const decoration = decorations[index] as Decoration
						const error = errors[index]
						return (
							<li key={`${index}-${decoration.kind}`}>
								<button
									type="button"
									aria-pressed={index === selectedIndex}
									onClick={() => onSelect(index)}
									className={cn(
										'flex w-full items-center justify-between gap-2 rounded-md border px-3 py-1.5 text-left text-sm outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50',
										index === selectedIndex
											? 'border-primary bg-primary/5'
											: 'hover:bg-muted',
										error ? 'border-destructive' : null,
									)}
								>
									<span>
										{index + 1}. {decorationLabel(decoration)}
									</span>
									<span className="text-xs text-muted-foreground">
										{PAGES_LABELS[pagesOf(decoration)]}
									</span>
								</button>
							</li>
						)
					})}
				</ul>
			)}

			{selected && selectedIndex !== null ? (
				<fieldset className="grid gap-4 rounded-md border p-3">
					<legend className="px-1 font-medium text-sm">
						Élément {selectedIndex + 1} : {decorationLabel(selected)}
					</legend>
					<div className="flex flex-wrap gap-1">
						<Button
							type="button"
							variant="ghost"
							size="sm"
							disabled={selectedIndex >= decorations.length - 1}
							onClick={() => {
								onChange(
									reorderDecoration(decorations, selectedIndex, 'forward'),
								)
								onSelect(reorderedIndex(decorations, selectedIndex, 'forward'))
							}}
						>
							<ArrowUp aria-hidden />
							Monter
						</Button>
						<Button
							type="button"
							variant="ghost"
							size="sm"
							disabled={selectedIndex <= 0}
							onClick={() => {
								onChange(
									reorderDecoration(decorations, selectedIndex, 'backward'),
								)
								onSelect(reorderedIndex(decorations, selectedIndex, 'backward'))
							}}
						>
							<ArrowDown aria-hidden />
							Descendre
						</Button>
						<Button
							type="button"
							variant="ghost"
							size="sm"
							disabled={full}
							onClick={() => {
								onChange(duplicateDecoration(decorations, selectedIndex))
								onSelect(selectedIndex + 1)
							}}
						>
							<Copy aria-hidden />
							Dupliquer
						</Button>
						<Button
							type="button"
							variant="ghost"
							size="sm"
							onClick={() => {
								onChange(removeDecoration(decorations, selectedIndex))
								onSelect(null)
							}}
						>
							<Trash2 aria-hidden />
							Supprimer
						</Button>
					</div>

					<div className="grid grid-cols-2 gap-4 md:grid-cols-4">
						<MmField
							label="X (mm)"
							value={selected.x}
							onChange={(x) =>
								onChange(
									updateDecoration(decorations, selectedIndex, {
										...selected,
										x,
									}),
								)
							}
						/>
						<MmField
							label="Y (mm)"
							value={selected.y}
							onChange={(y) =>
								onChange(
									updateDecoration(decorations, selectedIndex, {
										...selected,
										y,
									}),
								)
							}
						/>
						<MmField
							label="Largeur (mm)"
							value={selected.width}
							onChange={(width) =>
								onChange(
									updateDecoration(decorations, selectedIndex, {
										...selected,
										width,
									}),
								)
							}
						/>
						<MmField
							label="Hauteur (mm)"
							value={selected.height}
							onChange={(height) =>
								onChange(
									updateDecoration(decorations, selectedIndex, {
										...selected,
										height,
									}),
								)
							}
						/>
					</div>

					<div className="grid gap-4 md:grid-cols-2">
						<div className="flex flex-col gap-2">
							<Label>Pages concernées</Label>
							<Select
								value={pagesOf(selected)}
								onValueChange={(pages) =>
									onChange(
										updateDecoration(decorations, selectedIndex, {
											...selected,
											pages: pages as DecorationPages,
										}),
									)
								}
							>
								<SelectTrigger aria-label="Pages concernées" className="w-full">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									{(Object.keys(PAGES_LABELS) as DecorationPages[]).map(
										(key) => (
											<SelectItem key={key} value={key}>
												{PAGES_LABELS[key]}
											</SelectItem>
										),
									)}
								</SelectContent>
							</Select>
						</div>
						<div className="flex flex-col gap-2">
							<Label>Couche</Label>
							<Select
								value={layerOf(selected)}
								onValueChange={(layer) =>
									onChange(
										updateDecoration(decorations, selectedIndex, {
											...selected,
											layer: layer as DecorationLayer,
										}),
									)
								}
							>
								<SelectTrigger aria-label="Couche" className="w-full">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									{(Object.keys(LAYER_LABELS) as DecorationLayer[]).map(
										(key) => (
											<SelectItem key={key} value={key}>
												{LAYER_LABELS[key]}
											</SelectItem>
										),
									)}
								</SelectContent>
							</Select>
						</div>
					</div>

					{selected.kind === 'rectangle' ? (
						<div className="grid gap-4 md:grid-cols-3">
							<ColorField
								label="Couleur"
								value={selected.fill}
								error={undefined}
								onChange={(fill) =>
									onChange(
										updateDecoration(decorations, selectedIndex, {
											...selected,
											fill,
										}),
									)
								}
							/>
							<OpacityField
								value={selected.opacity}
								onChange={(opacity) =>
									onChange(
										updateDecoration(decorations, selectedIndex, {
											...selected,
											opacity,
										}),
									)
								}
							/>
							<MmField
								label="Rayon des angles (mm)"
								value={selected.radius ?? 0}
								onChange={(radius) =>
									onChange(
										updateDecoration(decorations, selectedIndex, {
											...selected,
											radius,
										}),
									)
								}
							/>
						</div>
					) : (
						<div className="grid gap-4">
							<div className="flex flex-wrap items-center gap-3">
								{selected.key ? (
									imageUrl ? (
										<img
											src={imageUrl}
											alt={`Visuel de l'élément ${selectedIndex + 1}`}
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
								<RequirePermission permission="MANAGE_REFERENCE">
									<FileUploadButton
										accept="image/*"
										isUploading={isUploadingImage}
										onFiles={(files) => {
											const file = files[0]
											if (file) onImageUpload(selectedIndex, file)
										}}
									>
										Choisir une image
									</FileUploadButton>
									{selected.key ? (
										<Button
											type="button"
											variant="ghost"
											size="sm"
											onClick={() =>
												onChange(
													updateDecoration(decorations, selectedIndex, {
														...selected,
														key: '',
													}),
												)
											}
										>
											<X />
											Retirer l'image
										</Button>
									) : null}
								</RequirePermission>
							</div>
							<div className="grid gap-4 md:grid-cols-2">
								<div className="flex flex-col gap-2">
									<Label>Ajustement</Label>
									<Select
										value={selected.fit ?? 'contain'}
										onValueChange={(fit) =>
											onChange(
												updateDecoration(decorations, selectedIndex, {
													...selected,
													fit: fit as 'contain' | 'cover',
												}),
											)
										}
									>
										<SelectTrigger aria-label="Ajustement" className="w-full">
											<SelectValue />
										</SelectTrigger>
										<SelectContent>
											{(
												Object.keys(FIT_LABELS) as Array<
													keyof typeof FIT_LABELS
												>
											).map((key) => (
												<SelectItem key={key} value={key}>
													{FIT_LABELS[key]}
												</SelectItem>
											))}
										</SelectContent>
									</Select>
								</div>
								<OpacityField
									value={selected.opacity}
									onChange={(opacity) =>
										onChange(
											updateDecoration(decorations, selectedIndex, {
												...selected,
												opacity,
											}),
										)
									}
								/>
							</div>
							<p className="text-xs text-muted-foreground">
								L'opacité d'une image n'agit que derrière le contenu (elle est
								rendue par un voile blanc). Devant le contenu, l'image reste
								pleinement opaque.
							</p>
						</div>
					)}
				</fieldset>
			) : null}
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

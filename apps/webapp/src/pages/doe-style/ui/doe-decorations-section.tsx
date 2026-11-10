import {
	ArrowDown,
	ArrowUp,
	Copy,
	Image as ImageIcon,
	Maximize2,
	Minus,
	Square,
	Trash2,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import {
	addDecoration,
	type Decoration,
	type DecorationKind,
	decorationLabel,
	duplicateDecoration,
	MAX_DECORATIONS,
	PAGES_LABELS,
	pagesOf,
	removeDecoration,
	reorderDecoration,
	reorderedIndex,
	updateDecoration,
} from '#/pages/doe-style/lib/decorations'
import { DecorationProperties } from '#/pages/doe-style/ui/decoration-properties'

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
	onOpenEditor: () => void
}

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
	onOpenEditor,
}: DoeDecorationsSectionProps) {
	const full = decorations.length >= MAX_DECORATIONS
	const selected =
		selectedIndex === null ? undefined : decorations[selectedIndex]
	const order = decorations.map((_, index) => index).reverse()

	return (
		<div className="grid gap-4">
			<div className="flex flex-wrap items-start justify-between gap-3">
				<p className="min-w-0 flex-1 text-xs text-muted-foreground">
					Images, rectangles et lignes répétés sur les pages du DOE. Glissez-les
					sur l'aperçu, ou ajustez-les aux flèches du clavier.{' '}
					{decorations.length} sur {MAX_DECORATIONS}.
				</p>
				<Button
					type="button"
					size="sm"
					variant="secondary"
					onClick={onOpenEditor}
				>
					<Maximize2 aria-hidden />
					Ouvrir l'éditeur
				</Button>
			</div>
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

					<DecorationProperties
						decoration={selected}
						index={selectedIndex}
						imageUrl={imageUrl}
						isUploadingImage={isUploadingImage}
						onChange={(next) =>
							onChange(updateDecoration(decorations, selectedIndex, next))
						}
						onImageUpload={(file) => onImageUpload(selectedIndex, file)}
					/>
				</fieldset>
			) : null}
		</div>
	)
}

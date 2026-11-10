import {
	ArrowDown,
	ArrowUp,
	BringToFront,
	Copy,
	FlipHorizontal2,
	FlipVertical2,
	Image as ImageIcon,
	Library,
	Loader2,
	Minus,
	Redo2,
	RotateCcw,
	RotateCw,
	SendToBack,
	Square,
	Trash2,
	Undo2,
} from 'lucide-react'
import type { ReactNode } from 'react'
import { FileUploadButton } from '#/components/file-upload-button'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogTitle,
} from '#/components/ui/dialog'
import { Separator } from '#/components/ui/separator'
import { cn } from '#/lib/utils'
import {
	type Decoration,
	type DecorationLayer,
	decorationLabel,
	layerOf,
	MAX_DECORATIONS,
	PAGES_LABELS,
	pagesOf,
	type Rect,
} from '#/pages/doe-style/lib/decorations'
import { rotationOf } from '#/pages/doe-style/lib/transform'
import { DecorationOverlay } from '#/pages/doe-style/ui/decoration-overlay'
import { DecorationProperties } from '#/pages/doe-style/ui/decoration-properties'
import {
	PREVIEW_PAGE_LABELS,
	svgDataUrl,
} from '#/pages/doe-style/ui/doe-style-preview'

interface LayoutEditorProps {
	decorations: Decoration[]
	errors: Array<string | undefined>
	selectedIndex: number | null
	pages: string[]
	activePage: number
	previewLoading: boolean
	message: string | null
	imageUrl: string | undefined
	canUndo: boolean
	canRedo: boolean
	isUploading: boolean
	invalid: boolean
	onSelect: (index: number | null) => void
	onSelectPage: (index: number) => void
	onAddShape: (kind: 'rectangle' | 'line') => void
	onAddImageFile: (file: File) => void
	onOpenLibrary: (mode: 'add' | 'replace') => void
	onReplaceImageFile: (file: File) => void
	onUpdateSelected: (next: Decoration) => void
	onCommitRect: (index: number, rect: Rect) => void
	onRotateTo: (index: number, rotation: number) => void
	onRotateBy: (delta: number) => void
	onToggleFlip: (axis: 'x' | 'y') => void
	onSetLayer: (layer: DecorationLayer) => void
	onMove: (direction: 'forward' | 'backward') => void
	onDuplicate: () => void
	onRemove: () => void
	onUndo: () => void
	onRedo: () => void
	onApply: () => void
	onCancel: () => void
	onKeyDown: (event: React.KeyboardEvent<HTMLElement>) => void
}

function ToolButton({
	label,
	icon,
	disabled,
	pressed,
	onClick,
}: {
	label: string
	icon: ReactNode
	disabled?: boolean
	pressed?: boolean
	onClick: () => void
}) {
	return (
		<Button
			type="button"
			variant={pressed ? 'secondary' : 'ghost'}
			size="icon-sm"
			aria-label={label}
			aria-pressed={pressed}
			title={label}
			disabled={disabled}
			onClick={onClick}
		>
			{icon}
		</Button>
	)
}

export function LayoutEditor({
	decorations,
	errors,
	selectedIndex,
	pages,
	activePage,
	previewLoading,
	message,
	imageUrl,
	canUndo,
	canRedo,
	isUploading,
	invalid,
	onSelect,
	onSelectPage,
	onAddShape,
	onAddImageFile,
	onOpenLibrary,
	onReplaceImageFile,
	onUpdateSelected,
	onCommitRect,
	onRotateTo,
	onRotateBy,
	onToggleFlip,
	onSetLayer,
	onMove,
	onDuplicate,
	onRemove,
	onUndo,
	onRedo,
	onApply,
	onCancel,
	onKeyDown,
}: LayoutEditorProps) {
	const full = decorations.length >= MAX_DECORATIONS
	const selected =
		selectedIndex === null ? undefined : decorations[selectedIndex]
	const current = Math.min(activePage, Math.max(pages.length - 1, 0))
	const page = pages[current]
	const order = decorations.map((_, index) => index).reverse()
	const none = selected === undefined

	return (
		<Dialog open onOpenChange={(next) => (next ? undefined : onCancel())}>
			<DialogContent
				showCloseButton={false}
				onKeyDown={onKeyDown}
				onEscapeKeyDown={(event) => {
					if (selectedIndex !== null) {
						event.preventDefault()
						onSelect(null)
					}
				}}
				className="top-0 left-0 grid h-dvh w-screen max-w-none translate-x-0 translate-y-0 grid-rows-[auto_auto_minmax(0,1fr)] gap-0 rounded-none border-0 p-0 sm:max-w-none"
			>
				<header className="flex flex-wrap items-center justify-between gap-3 border-b px-4 py-2">
					<div className="grid">
						<DialogTitle className="text-base">
							Éditeur de mise en page
						</DialogTitle>
						<DialogDescription className="text-xs">
							Les changements ne sont gardés dans le style qu'avec « Valider ».
						</DialogDescription>
					</div>
					<div className="flex items-center gap-2">
						{message ? (
							<p
								role="alert"
								className="max-w-80 truncate text-destructive text-xs"
							>
								{message}
							</p>
						) : null}
						{previewLoading ? (
							<output aria-label="Aperçu en cours de mise à jour">
								<Loader2 className="size-4 animate-spin text-muted-foreground" />
							</output>
						) : null}
						<ToolButton
							label="Annuler la dernière action"
							icon={<Undo2 aria-hidden />}
							disabled={!canUndo}
							onClick={onUndo}
						/>
						<ToolButton
							label="Rétablir"
							icon={<Redo2 aria-hidden />}
							disabled={!canRedo}
							onClick={onRedo}
						/>
						<Separator orientation="vertical" className="mx-1 h-6" />
						<Button
							type="button"
							variant="outline"
							size="sm"
							onClick={onCancel}
						>
							Annuler
						</Button>
						<Button
							type="button"
							size="sm"
							disabled={invalid}
							onClick={onApply}
						>
							Valider
						</Button>
					</div>
				</header>

				<div
					role="toolbar"
					aria-label="Outils"
					className="flex flex-wrap items-center gap-1 border-b px-4 py-1.5"
				>
					<RequirePermission permission="MANAGE_REFERENCE">
						<FileUploadButton
							accept="image/*"
							disabled={full}
							isUploading={isUploading}
							onFiles={(files) => {
								const file = files[0]
								if (file) onAddImageFile(file)
							}}
						>
							<ImageIcon aria-hidden />
							Image depuis l'ordinateur
						</FileUploadButton>
					</RequirePermission>
					<RequirePermission permission="MANAGE_PLANNING">
						<Button
							type="button"
							variant="outline"
							size="sm"
							disabled={full}
							onClick={() => onOpenLibrary('add')}
						>
							<Library aria-hidden />
							Image depuis la photothèque
						</Button>
					</RequirePermission>
					<Button
						type="button"
						variant="outline"
						size="sm"
						disabled={full}
						onClick={() => onAddShape('rectangle')}
					>
						<Square aria-hidden />
						Rectangle
					</Button>
					<Button
						type="button"
						variant="outline"
						size="sm"
						disabled={full}
						onClick={() => onAddShape('line')}
					>
						<Minus aria-hidden />
						Ligne
					</Button>
					<Separator orientation="vertical" className="mx-2 h-6" />
					<ToolButton
						label="Pivoter de -15°"
						icon={<RotateCcw aria-hidden />}
						disabled={none}
						onClick={() => onRotateBy(-15)}
					/>
					<ToolButton
						label="Pivoter de +15°"
						icon={<RotateCw aria-hidden />}
						disabled={none}
						onClick={() => onRotateBy(15)}
					/>
					<ToolButton
						label="Miroir horizontal"
						icon={<FlipHorizontal2 aria-hidden />}
						disabled={none}
						pressed={Boolean(selected?.flip_x)}
						onClick={() => onToggleFlip('x')}
					/>
					<ToolButton
						label="Miroir vertical"
						icon={<FlipVertical2 aria-hidden />}
						disabled={none}
						pressed={Boolean(selected?.flip_y)}
						onClick={() => onToggleFlip('y')}
					/>
					<ToolButton
						label="Devant le contenu"
						icon={<BringToFront aria-hidden />}
						disabled={none}
						pressed={selected ? layerOf(selected) === 'front' : false}
						onClick={() => onSetLayer('front')}
					/>
					<ToolButton
						label="Derrière le contenu"
						icon={<SendToBack aria-hidden />}
						disabled={none}
						pressed={selected ? layerOf(selected) === 'behind' : false}
						onClick={() => onSetLayer('behind')}
					/>
					<ToolButton
						label="Dupliquer"
						icon={<Copy aria-hidden />}
						disabled={none || full}
						onClick={onDuplicate}
					/>
					<ToolButton
						label="Supprimer"
						icon={<Trash2 aria-hidden />}
						disabled={none}
						onClick={onRemove}
					/>
					<p className="ml-auto hidden text-muted-foreground text-xs xl:block">
						Flèches : 1 mm (Maj : 10 mm) · Alt : sans repères · Maj en pivotant
						: angle libre
					</p>
				</div>

				<div className="grid min-h-0 grid-cols-[15rem_minmax(0,1fr)_18rem]">
					<aside
						aria-label="Pages et calques"
						className="flex min-h-0 flex-col gap-4 overflow-y-auto border-r p-3"
					>
						<section className="grid gap-2">
							<h3 className="font-medium text-sm">Pages</h3>
							<div className="grid grid-cols-3 gap-2">
								{pages.map((svg, index) => (
									<button
										// biome-ignore lint/suspicious/noArrayIndexKey: pages are positional
										key={index}
										type="button"
										aria-label={`Afficher : ${PREVIEW_PAGE_LABELS[index] ?? `page ${index + 1}`}`}
										aria-pressed={index === current}
										onClick={() => onSelectPage(index)}
										className={cn(
											'flex flex-col gap-1 rounded-md border p-1 text-left',
											index === current
												? 'border-primary ring-2 ring-primary/30'
												: 'border-transparent hover:border-border',
										)}
									>
										<span className="block aspect-[210/297] overflow-hidden rounded-sm bg-white shadow ring-1 ring-black/5">
											<img
												src={svgDataUrl(svg)}
												alt=""
												className="size-full object-contain"
											/>
										</span>
										<span className="truncate text-[11px] text-muted-foreground">
											{PREVIEW_PAGE_LABELS[index] ?? `Page ${index + 1}`}
										</span>
									</button>
								))}
							</div>
						</section>
						<section className="grid gap-2">
							<h3 className="font-medium text-sm">
								Calques ({decorations.length} sur {MAX_DECORATIONS})
							</h3>
							{decorations.length === 0 ? (
								<p className="text-muted-foreground text-sm">Aucun élément.</p>
							) : (
								<ul
									aria-label="Éléments, du dessus au dessous"
									className="grid gap-1"
								>
									{order.map((index) => {
										const decoration = decorations[index] as Decoration
										const isSelected = index === selectedIndex
										return (
											<li
												key={`${index}-${decoration.kind}`}
												className="grid gap-1"
											>
												<button
													type="button"
													aria-pressed={isSelected}
													onClick={() => onSelect(index)}
													className={cn(
														'flex w-full items-center justify-between gap-2 rounded-md border px-2 py-1.5 text-left text-sm outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50',
														isSelected
															? 'border-primary bg-primary/5'
															: 'hover:bg-muted',
														errors[index] ? 'border-destructive' : null,
													)}
												>
													<span className="truncate">
														{index + 1}. {decorationLabel(decoration)}
														{rotationOf(decoration) !== 0
															? ` (${rotationOf(decoration)}°)`
															: ''}
													</span>
													<span className="shrink-0 text-muted-foreground text-xs">
														{PAGES_LABELS[pagesOf(decoration)]}
													</span>
												</button>
												{isSelected ? (
													<div className="flex gap-1">
														<Button
															type="button"
															variant="ghost"
															size="xs"
															disabled={index >= decorations.length - 1}
															onClick={() => onMove('forward')}
														>
															<ArrowUp aria-hidden />
															Monter
														</Button>
														<Button
															type="button"
															variant="ghost"
															size="xs"
															disabled={index <= 0}
															onClick={() => onMove('backward')}
														>
															<ArrowDown aria-hidden />
															Descendre
														</Button>
														<Button
															type="button"
															variant="ghost"
															size="xs"
															disabled={full}
															onClick={onDuplicate}
														>
															<Copy aria-hidden />
															Dupliquer
														</Button>
														<Button
															type="button"
															variant="ghost"
															size="icon-xs"
															aria-label="Supprimer"
															onClick={onRemove}
														>
															<Trash2 aria-hidden />
														</Button>
													</div>
												) : null}
											</li>
										)
									})}
								</ul>
							)}
						</section>
					</aside>

					<main
						aria-label="Page"
						className="flex min-h-0 items-start justify-center overflow-auto bg-muted/40 p-10"
					>
						<div
							style={{
								height: 'calc(100dvh - 16rem)',
								aspectRatio: '210 / 297',
							}}
							className="relative shrink-0"
						>
							<div
								className={cn(
									'size-full overflow-hidden rounded-sm bg-white shadow-lg ring-1 ring-black/5 transition-opacity',
									previewLoading ? 'opacity-80' : 'opacity-100',
								)}
							>
								{page ? (
									<img
										src={svgDataUrl(page)}
										alt={`Aperçu : ${PREVIEW_PAGE_LABELS[current] ?? `page ${current + 1}`}`}
										className="size-full object-contain"
									/>
								) : (
									<div className="flex size-full items-center justify-center text-muted-foreground text-sm">
										{previewLoading
											? "Génération de l'aperçu…"
											: 'Aperçu indisponible'}
									</div>
								)}
							</div>
							{page ? (
								<DecorationOverlay
									decorations={decorations}
									pageIndex={current}
									selectedIndex={selectedIndex}
									onSelect={onSelect}
									onCommit={onCommitRect}
									onRotate={onRotateTo}
									snapping
								/>
							) : null}
						</div>
					</main>

					<aside
						aria-label="Propriétés"
						className="min-h-0 overflow-y-auto border-l p-3"
					>
						<h3 className="mb-3 font-medium text-sm">Propriétés</h3>
						{selected && selectedIndex !== null ? (
							<div className="grid gap-3">
								<p className="text-muted-foreground text-xs">
									Élément {selectedIndex + 1} : {decorationLabel(selected)}
								</p>
								{errors[selectedIndex] ? (
									<p role="alert" className="text-destructive text-xs">
										{errors[selectedIndex]}
									</p>
								) : null}
								<DecorationProperties
									decoration={selected}
									index={selectedIndex}
									imageUrl={imageUrl}
									isUploadingImage={isUploading}
									onChange={onUpdateSelected}
									onImageUpload={onReplaceImageFile}
									onPickFromLibrary={() => onOpenLibrary('replace')}
								/>
							</div>
						) : (
							<p className="text-muted-foreground text-sm">
								Sélectionnez un élément sur la page ou dans les calques.
							</p>
						)}
					</aside>
				</div>
			</DialogContent>
		</Dialog>
	)
}

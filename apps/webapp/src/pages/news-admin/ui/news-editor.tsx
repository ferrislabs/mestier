import { ArrowDown, ArrowLeft, ArrowUp, Plus, Trash2, X } from 'lucide-react'
import { FileUploadButton } from '#/components/file-upload-button'
import { FormField } from '#/components/form-field'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from '#/components/ui/alert-dialog'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Textarea } from '#/components/ui/textarea'
import { cn } from '#/lib/utils'
import {
	NEWS_ICON_OPTIONS,
	newsIconComponent,
} from '#/pages/news/lib/news-icons'
import { NewsContent } from '#/pages/news/ui/news-content'
import {
	MAX_CTA_LABEL,
	MAX_ITEM_TEXT,
	MAX_ITEM_TITLE,
	MAX_ITEMS,
	MAX_TITLE,
	type NewsDraft,
	type NewsDraftErrors,
	type NewsDraftItem,
	type PublicationMode,
} from '#/pages/news-admin/lib/news-form'

interface NewsEditorProps {
	draft: NewsDraft
	errors: NewsDraftErrors
	isNew: boolean
	imageUrl?: string
	uploading: boolean
	saving: boolean
	deleting: boolean
	error: string | null
	deleteOpen: boolean
	onDeleteOpenChange: (open: boolean) => void
	onChange: (patch: Partial<NewsDraft>) => void
	onItemChange: (index: number, patch: Partial<NewsDraftItem>) => void
	onItemAdd: () => void
	onItemRemove: (index: number) => void
	onItemMove: (index: number, delta: -1 | 1) => void
	onImageUpload: (file: File) => void
	onImageRemove: () => void
	onSave: () => void
	onDelete: () => void
	onBack: () => void
}

const MODES: ReadonlyArray<{ value: PublicationMode; label: string }> = [
	{ value: 'draft', label: 'Brouillon' },
	{ value: 'now', label: 'Publier maintenant' },
	{ value: 'scheduled', label: 'Programmer' },
]

export function NewsEditor({
	draft,
	errors,
	isNew,
	imageUrl,
	uploading,
	saving,
	deleting,
	error,
	deleteOpen,
	onDeleteOpenChange,
	onChange,
	onItemChange,
	onItemAdd,
	onItemRemove,
	onItemMove,
	onImageUpload,
	onImageRemove,
	onSave,
	onDelete,
	onBack,
}: NewsEditorProps) {
	const preview = {
		title: draft.title.trim() || 'Titre de la news',
		items: draft.items.map((item) => ({
			...item,
			title: item.title || 'Titre du point',
			text: item.text || 'Phrase de description.',
		})),
		cta:
			draft.ctaEnabled && draft.ctaLabel.trim()
				? { label: draft.ctaLabel.trim(), url: draft.ctaUrl.trim() }
				: null,
	}

	return (
		<div className="mx-auto flex w-full max-w-6xl flex-col gap-4 p-6">
			<div className="flex items-center justify-between gap-3">
				<Button type="button" variant="ghost" size="sm" onClick={onBack}>
					<ArrowLeft />
					Retour à la liste
				</Button>
				<div className="flex items-center gap-2">
					{isNew ? null : (
						<Button
							type="button"
							variant="outline"
							size="sm"
							disabled={deleting}
							onClick={() => onDeleteOpenChange(true)}
						>
							<Trash2 />
							Supprimer
						</Button>
					)}
					<Button type="button" disabled={saving || uploading} onClick={onSave}>
						{saving ? 'Enregistrement…' : 'Enregistrer'}
					</Button>
				</div>
			</div>
			{error ? (
				<p role="alert" className="text-sm text-state-alert">
					{error}
				</p>
			) : null}
			<div className="grid items-start gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,28rem)]">
				<form
					className="flex flex-col gap-6"
					onSubmit={(event) => {
						event.preventDefault()
						onSave()
					}}
				>
					<FormField
						id="news-title"
						label="Titre"
						required
						error={errors.title}
						hint={`${MAX_TITLE} caractères maximum`}
					>
						<Input
							id="news-title"
							value={draft.title}
							maxLength={MAX_TITLE * 2}
							aria-invalid={Boolean(errors.title)}
							onChange={(event) => onChange({ title: event.target.value })}
						/>
					</FormField>

					<div className="grid gap-2">
						<span className="font-medium text-sm">Image d'en-tête</span>
						<div className="flex flex-wrap items-center gap-2">
							<FileUploadButton
								accept="image/*"
								isUploading={uploading}
								onFiles={(files) => {
									const file = files[0]
									if (file) onImageUpload(file)
								}}
							>
								{draft.imageKey ? "Remplacer l'image" : 'Choisir une image'}
							</FileUploadButton>
							{draft.imageKey ? (
								<Button
									type="button"
									variant="ghost"
									size="sm"
									onClick={onImageRemove}
								>
									<X />
									Retirer l'image
								</Button>
							) : (
								<span className="text-muted-foreground text-sm">
									Sans image, un dégradé est affiché.
								</span>
							)}
						</div>
						{imageUrl ? (
							<img
								src={imageUrl}
								alt=""
								className="h-20 w-40 rounded-md border object-cover"
							/>
						) : null}
					</div>

					<fieldset className="grid gap-3">
						<legend className="mb-1 font-medium text-sm">Points</legend>
						{errors.items ? (
							<p className="text-[12px] text-state-alert">{errors.items}</p>
						) : null}
						{draft.items.map((item, index) => (
							<div
								// biome-ignore lint/suspicious/noArrayIndexKey: points have no id and are reordered in place
								key={index}
								className="grid gap-3 rounded-lg border bg-card p-3"
							>
								<div className="flex items-center justify-between gap-2">
									<span className="font-medium text-sm">Point {index + 1}</span>
									<div className="flex items-center gap-1">
										<Button
											type="button"
											variant="ghost"
											size="icon-sm"
											aria-label={`Monter le point ${index + 1}`}
											disabled={index === 0}
											onClick={() => onItemMove(index, -1)}
										>
											<ArrowUp />
										</Button>
										<Button
											type="button"
											variant="ghost"
											size="icon-sm"
											aria-label={`Descendre le point ${index + 1}`}
											disabled={index === draft.items.length - 1}
											onClick={() => onItemMove(index, 1)}
										>
											<ArrowDown />
										</Button>
										<Button
											type="button"
											variant="ghost"
											size="icon-sm"
											aria-label={`Supprimer le point ${index + 1}`}
											onClick={() => onItemRemove(index)}
										>
											<Trash2 />
										</Button>
									</div>
								</div>
								<fieldset className="m-0 grid min-w-0 grid-cols-8 gap-1 border-0 p-0">
									<legend className="sr-only">{`Icône du point ${index + 1}`}</legend>
									{NEWS_ICON_OPTIONS.map((option) => {
										const Icon = newsIconComponent(option.value)
										const active = option.value === item.icon
										return (
											<button
												key={option.value}
												type="button"
												aria-pressed={active}
												aria-label={option.label}
												title={option.label}
												className={cn(
													'flex size-8 items-center justify-center rounded-md border transition-colors focus-visible:ring-2 focus-visible:ring-ring',
													active
														? 'border-primary bg-primary text-primary-foreground'
														: 'text-muted-foreground hover:bg-accent',
												)}
												onClick={() =>
													onItemChange(index, { icon: option.value })
												}
											>
												<Icon className="size-4" />
											</button>
										)
									})}
								</fieldset>
								<FormField
									id={`news-item-${index}-title`}
									label="Titre du point"
									required
									error={errors[`item.${index}.title`]}
									hint={`${MAX_ITEM_TITLE} caractères maximum`}
								>
									<Input
										id={`news-item-${index}-title`}
										value={item.title}
										onChange={(event) =>
											onItemChange(index, { title: event.target.value })
										}
									/>
								</FormField>
								<FormField
									id={`news-item-${index}-text`}
									label="Texte du point"
									required
									error={errors[`item.${index}.text`]}
									hint={`${MAX_ITEM_TEXT} caractères maximum`}
								>
									<Textarea
										id={`news-item-${index}-text`}
										rows={2}
										value={item.text}
										onChange={(event) =>
											onItemChange(index, { text: event.target.value })
										}
									/>
								</FormField>
							</div>
						))}
						<div>
							<Button
								type="button"
								variant="outline"
								size="sm"
								disabled={draft.items.length >= MAX_ITEMS}
								onClick={onItemAdd}
							>
								<Plus />
								Ajouter un point
							</Button>
						</div>
					</fieldset>

					<fieldset className="grid gap-3">
						<legend className="mb-1 font-medium text-sm">Bouton</legend>
						<label className="flex items-center gap-2 text-sm">
							<input
								type="checkbox"
								checked={draft.ctaEnabled}
								onChange={(event) =>
									onChange({ ctaEnabled: event.target.checked })
								}
							/>
							Afficher un bouton en pied de fenêtre
						</label>
						{draft.ctaEnabled ? (
							<div className="grid gap-3 sm:grid-cols-2">
								<FormField
									id="news-cta-label"
									label="Libellé"
									required
									error={errors['cta.label']}
									hint={`${MAX_CTA_LABEL} caractères maximum`}
								>
									<Input
										id="news-cta-label"
										value={draft.ctaLabel}
										onChange={(event) =>
											onChange({ ctaLabel: event.target.value })
										}
									/>
								</FormField>
								<FormField
									id="news-cta-url"
									label="Adresse"
									required
									error={errors['cta.url']}
								>
									<Input
										id="news-cta-url"
										type="url"
										placeholder="https://"
										value={draft.ctaUrl}
										onChange={(event) =>
											onChange({ ctaUrl: event.target.value })
										}
									/>
								</FormField>
							</div>
						) : null}
					</fieldset>

					<fieldset className="grid gap-3">
						<legend className="mb-1 font-medium text-sm">Publication</legend>
						<div className="flex flex-wrap gap-4">
							{MODES.map((mode) => (
								<label
									key={mode.value}
									className="flex items-center gap-2 text-sm"
								>
									<input
										type="radio"
										name="news-mode"
										checked={draft.mode === mode.value}
										onChange={() => onChange({ mode: mode.value })}
									/>
									{mode.label}
								</label>
							))}
						</div>
						{draft.mode === 'scheduled' ? (
							<FormField
								id="news-scheduled-at"
								label="Date et heure de publication"
								required
								error={errors.scheduledAt}
								className="max-w-xs"
							>
								<Input
									id="news-scheduled-at"
									type="datetime-local"
									value={draft.scheduledAt}
									onChange={(event) =>
										onChange({ scheduledAt: event.target.value })
									}
								/>
							</FormField>
						) : null}
					</fieldset>
				</form>

				<aside aria-label="Aperçu" className="grid gap-2 lg:sticky lg:top-4">
					<span className="font-medium text-muted-foreground text-sm">
						Aperçu
					</span>
					<NewsContent
						news={preview}
						imageUrl={imageUrl}
						index={0}
						count={1}
						onIndexChange={() => {}}
						className="border shadow-sm"
					/>
				</aside>
			</div>

			<AlertDialog open={deleteOpen} onOpenChange={onDeleteOpenChange}>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>Supprimer cette news ?</AlertDialogTitle>
						<AlertDialogDescription>
							La news disparaît pour tous les utilisateurs. Cette action est
							définitive.
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel>Annuler</AlertDialogCancel>
						<AlertDialogAction disabled={deleting} onClick={onDelete}>
							Supprimer
						</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>
		</div>
	)
}

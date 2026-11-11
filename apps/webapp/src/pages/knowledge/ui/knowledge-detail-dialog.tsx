import { Download, Loader2, X } from 'lucide-react'
import { useId, useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type {
	KnowledgeArticle,
	KnowledgeLink,
	KnowledgeResourceType,
} from '#/hooks/use-knowledge'
import {
	RESOURCE_TYPE_LABELS,
	type ResourceOption,
	resourceLabel,
} from '#/pages/knowledge/feature/use-resource-options'
import { fileTypeLabel, formatSize } from '#/pages/knowledge/lib/files'
import { TagsEditor } from '#/pages/knowledge/ui/tags-editor'

export interface ArticleDraft {
	title: string
	tags: string[]
	tagInput: string
}

interface KnowledgeDetailDialogProps {
	open: boolean
	article: KnowledgeArticle
	canManage: boolean
	draft: ArticleDraft
	isDirty: boolean
	isSaving: boolean
	isDownloading: boolean
	isLinking: boolean
	error: string | null
	resources: readonly ResourceOption[]
	onDraftChange: (patch: Partial<ArticleDraft>) => void
	onAddTags: () => void
	onRemoveTag: (tag: string) => void
	onSave: () => void
	onDelete: () => void
	onDownload: () => void
	onLink: (type: KnowledgeResourceType, id: string) => void
	onUnlink: (link: KnowledgeLink) => void
	onOpenChange: (open: boolean) => void
}

const TEXT_PREVIEW_CHARS = 1200

export function ResourcePicker({
	resources,
	disabled,
	onPick,
}: {
	resources: readonly ResourceOption[]
	disabled: boolean
	onPick: (type: KnowledgeResourceType, id: string) => void
}) {
	const [value, setValue] = useState('')
	return (
		<div className="flex gap-2">
			<Select value={value} onValueChange={setValue}>
				<SelectTrigger aria-label="Rattacher à" className="min-w-0 flex-1">
					<SelectValue placeholder="Un projet, un client ou un devis" />
				</SelectTrigger>
				<SelectContent>
					{resources.map((resource) => (
						<SelectItem
							key={`${resource.type}:${resource.id}`}
							value={`${resource.type}:${resource.id}`}
						>
							{RESOURCE_TYPE_LABELS[resource.type]} · {resource.label}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
			<Button
				type="button"
				variant="outline"
				disabled={disabled || value === ''}
				onClick={() => {
					const [type, ...rest] = value.split(':')
					onPick(type as KnowledgeResourceType, rest.join(':'))
					setValue('')
				}}
			>
				Rattacher
			</Button>
		</div>
	)
}

export function KnowledgeDetailDialog({
	open,
	article,
	canManage,
	draft,
	isDirty,
	isSaving,
	isDownloading,
	isLinking,
	error,
	resources,
	onDraftChange,
	onAddTags,
	onRemoveTag,
	onSave,
	onDelete,
	onDownload,
	onLink,
	onUnlink,
	onOpenChange,
}: KnowledgeDetailDialogProps) {
	const baseId = useId()
	const links = article.links ?? []
	const text = article.extracted_text ?? ''
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[90dvh] overflow-y-auto sm:max-w-2xl">
				<DialogHeader>
					<DialogTitle>Fichier</DialogTitle>
					<DialogDescription>
						{article.file
							? `${fileTypeLabel(article.file.mime_type)} · ${formatSize(article.file.size_bytes)}`
							: ''}
					</DialogDescription>
				</DialogHeader>
				<form
					noValidate
					className="grid gap-4"
					onSubmit={(event) => {
						event.preventDefault()
						if (canManage && isDirty && !isSaving) onSave()
					}}
				>
					<div className="grid gap-1">
						<Label htmlFor={`${baseId}-title`}>Titre</Label>
						<Input
							id={`${baseId}-title`}
							value={draft.title}
							maxLength={200}
							readOnly={!canManage}
							autoComplete="off"
							onChange={(event) => onDraftChange({ title: event.target.value })}
						/>
					</div>
					<div className="grid gap-2">
						<Button
							type="button"
							variant="outline"
							className="justify-self-start"
							disabled={isDownloading}
							onClick={onDownload}
						>
							{isDownloading ? (
								<Loader2 className="animate-spin" />
							) : (
								<Download aria-hidden />
							)}
							Télécharger le fichier
						</Button>
						{text !== '' ? (
							<details className="rounded-md border px-3 py-2 text-sm">
								<summary className="cursor-pointer text-muted-foreground">
									Texte lu dans le document
								</summary>
								<p className="mt-2 whitespace-pre-wrap text-[13px]">
									{text.length > TEXT_PREVIEW_CHARS
										? `${text.slice(0, TEXT_PREVIEW_CHARS)}…`
										: text}
								</p>
							</details>
						) : (
							<p className="text-[13px] text-muted-foreground">
								Aucun texte lu dans ce fichier : il se retrouve par son titre et
								ses étiquettes.
							</p>
						)}
					</div>
					<TagsEditor
						tags={draft.tags}
						input={draft.tagInput}
						disabled={!canManage}
						onInputChange={(tagInput) => onDraftChange({ tagInput })}
						onAdd={onAddTags}
						onRemove={onRemoveTag}
					/>
					<div className="grid gap-2">
						<Label>Rattaché à</Label>
						{links.length > 0 ? (
							<ul className="flex flex-wrap gap-1.5">
								{links.map((link) => (
									<li
										key={`${link.resource_type}:${link.resource_id}`}
										className="flex items-center gap-1 rounded-full bg-muted py-0.5 pr-1 pl-2.5 text-sm"
									>
										{RESOURCE_TYPE_LABELS[link.resource_type]} ·{' '}
										{resourceLabel(
											resources,
											link.resource_type,
											link.resource_id,
										)}
										{canManage ? (
											<button
												type="button"
												aria-label={`Détacher ${resourceLabel(resources, link.resource_type, link.resource_id)}`}
												onClick={() => onUnlink(link)}
												className="rounded-full p-0.5 outline-none hover:bg-background focus-visible:ring-[3px] focus-visible:ring-ring/50"
											>
												<X aria-hidden className="size-3" />
											</button>
										) : null}
									</li>
								))}
							</ul>
						) : (
							<p className="text-[13px] text-muted-foreground">
								Rattaché à aucun projet, client ni devis.
							</p>
						)}
						{canManage ? (
							<ResourcePicker
								resources={resources}
								disabled={isLinking}
								onPick={onLink}
							/>
						) : null}
					</div>
					{error ? (
						<p role="alert" className="text-[13px] text-state-alert">
							{error}
						</p>
					) : null}
					<DialogFooter className="sm:justify-between">
						{canManage ? (
							<Button type="button" variant="destructive" onClick={onDelete}>
								Supprimer
							</Button>
						) : (
							<span />
						)}
						<div className="flex gap-2">
							<Button
								type="button"
								variant="outline"
								onClick={() => onOpenChange(false)}
							>
								Fermer
							</Button>
							{canManage ? (
								<Button type="submit" disabled={!isDirty || isSaving}>
									{isSaving ? <Loader2 className="animate-spin" /> : null}
									Enregistrer
								</Button>
							) : null}
						</div>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}

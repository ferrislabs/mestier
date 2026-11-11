import { FolderInput, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Label } from '#/components/ui/label'
import type {
	KnowledgeFolder,
	KnowledgeLink,
	KnowledgeResourceType,
} from '#/hooks/use-knowledge'
import {
	RESOURCE_TYPE_LABELS,
	type ResourceOption,
	resourceLabel,
} from '#/pages/knowledge/feature/use-resource-options'
import { ResourcePicker } from '#/pages/knowledge/ui/knowledge-detail-dialog'
import { TagsEditor } from '#/pages/knowledge/ui/tags-editor'

interface DocumentInfoPanelProps {
	canManage: boolean
	tags: readonly string[]
	tagInput: string
	path: readonly KnowledgeFolder[]
	links: readonly KnowledgeLink[]
	resources: readonly ResourceOption[]
	isLinking: boolean
	error: string | null
	onTagInputChange: (input: string) => void
	onAddTags: () => void
	onRemoveTag: (tag: string) => void
	onMove: () => void
	onLink: (type: KnowledgeResourceType, id: string) => void
	onUnlink: (link: KnowledgeLink) => void
}

export function DocumentInfoPanel({
	canManage,
	tags,
	tagInput,
	path,
	links,
	resources,
	isLinking,
	error,
	onTagInputChange,
	onAddTags,
	onRemoveTag,
	onMove,
	onLink,
	onUnlink,
}: DocumentInfoPanelProps) {
	return (
		<aside
			aria-label="Infos du document"
			className="grid content-start gap-5 rounded-lg border bg-card p-4"
		>
			<h2 className="font-semibold text-base">Infos</h2>
			<TagsEditor
				tags={tags}
				input={tagInput}
				disabled={!canManage}
				onInputChange={onTagInputChange}
				onAdd={onAddTags}
				onRemove={onRemoveTag}
			/>
			<div className="grid gap-2">
				<Label>Dossier</Label>
				<p className="text-sm">
					{['Drive', ...path.map((folder) => folder.name)].join(' / ')}
				</p>
				{canManage ? (
					<Button
						type="button"
						variant="outline"
						size="sm"
						className="justify-self-start"
						onClick={onMove}
					>
						<FolderInput aria-hidden />
						Déplacer
					</Button>
				) : null}
			</div>
			<div className="grid gap-2">
				<Label>Rattaché à</Label>
				{links.length > 0 ? (
					<ul className="flex flex-wrap gap-1.5">
						{links.map((link) => {
							const name = resourceLabel(
								resources,
								link.resource_type,
								link.resource_id,
							)
							return (
								<li
									key={`${link.resource_type}:${link.resource_id}`}
									className="flex items-center gap-1 rounded-full bg-muted py-0.5 pr-1 pl-2.5 text-sm"
								>
									{RESOURCE_TYPE_LABELS[link.resource_type]} · {name}
									{canManage ? (
										<button
											type="button"
											aria-label={`Détacher ${name}`}
											onClick={() => onUnlink(link)}
											className="rounded-full p-0.5 outline-none hover:bg-background focus-visible:ring-[3px] focus-visible:ring-ring/50"
										>
											<X aria-hidden className="size-3" />
										</button>
									) : null}
								</li>
							)
						})}
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
		</aside>
	)
}

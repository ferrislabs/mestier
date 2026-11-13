import { X } from 'lucide-react'
import { Label } from '#/components/ui/label'
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import type {
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
	open: boolean
	canManage: boolean
	tags: readonly string[]
	tagInput: string
	links: readonly KnowledgeLink[]
	resources: readonly ResourceOption[]
	isLinking: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onTagInputChange: (input: string) => void
	onAddTags: () => void
	onRemoveTag: (tag: string) => void
	onLink: (type: KnowledgeResourceType, id: string) => void
	onUnlink: (link: KnowledgeLink) => void
}

export function DocumentInfoPanel({
	open,
	canManage,
	tags,
	tagInput,
	links,
	resources,
	isLinking,
	error,
	onOpenChange,
	onTagInputChange,
	onAddTags,
	onRemoveTag,
	onLink,
	onUnlink,
}: DocumentInfoPanelProps) {
	return (
		<Sheet open={open} onOpenChange={onOpenChange}>
			<SheetContent className="gap-0 sm:max-w-md">
				<SheetHeader className="border-b px-6 py-5">
					<SheetTitle>Infos du document</SheetTitle>
					<SheetDescription>
						Étiquettes et rattachements aux projets, clients et devis.
					</SheetDescription>
				</SheetHeader>
				<div className="grid content-start gap-6 overflow-y-auto px-6 py-5">
					<TagsEditor
						tags={tags}
						input={tagInput}
						disabled={!canManage}
						onInputChange={onTagInputChange}
						onAdd={onAddTags}
						onRemove={onRemoveTag}
					/>
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
				</div>
			</SheetContent>
		</Sheet>
	)
}

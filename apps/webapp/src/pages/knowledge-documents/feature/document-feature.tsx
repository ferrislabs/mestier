import { useNavigate } from '@tanstack/react-router'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useChatDirectory } from '#/hooks/use-chat'
import {
	type KnowledgeArticle,
	useDeleteKnowledgeArticle,
	useKnowledgeFolders,
	useLinkKnowledgeArticle,
	useMoveKnowledgeArticles,
	usePatchKnowledgeArticle,
	useUnlinkKnowledgeArticle,
} from '#/hooks/use-knowledge'
import { useKnowledgeDocument } from '#/hooks/use-knowledge-documents'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import {
	RESOURCE_TYPE_LABELS,
	resourceLabel,
	useResourceOptions,
} from '#/pages/knowledge/feature/use-resource-options'
import { pathTo } from '#/pages/knowledge/lib/folders'
import { FolderPickerDialog } from '#/pages/knowledge/ui/folder-picker-dialog'
import { DocumentEditor } from '#/pages/knowledge-documents/feature/document-editor'
import type { DocumentEnvironment } from '#/pages/knowledge-documents/feature/document-environment'
import { useDocumentSession } from '#/pages/knowledge-documents/feature/use-document-session'
import { toEditorBlocks } from '#/pages/knowledge-documents/lib/blocks'
import type { MentionKind } from '#/pages/knowledge-documents/lib/document-api-types'
import {
	type BannerKind,
	DocumentBanner,
} from '#/pages/knowledge-documents/ui/document-banner'
import { DocumentInfoPanel } from '#/pages/knowledge-documents/ui/document-info-panel'
import {
	DocumentLoading,
	DocumentPage,
	DocumentUnavailable,
} from '#/pages/knowledge-documents/ui/document-page'
import { DocumentProperties } from '#/pages/knowledge-documents/ui/document-properties'
import { SaveIndicator } from '#/pages/knowledge-documents/ui/save-indicator'
import { addTags, removeTag } from '#/pages/photo-library/lib/tags'

const RESOURCE_PATHS: Record<MentionKind, string> = {
	customer: '/crm/clients',
	project: '/projet',
	quote: '/crm/devis',
}

export function DocumentFeature({ articleId }: { articleId: string }) {
	const { activeOrganization } = useActiveOrganization()
	return (
		<DocumentLoader
			key={`${activeOrganization.id}-${articleId}`}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			articleId={articleId}
		/>
	)
}

interface LoaderProps {
	organizationId: string
	organizationSlug: string
	articleId: string
}

function useDriveNavigation(organizationSlug: string) {
	const navigate = useNavigate()
	return useCallback(
		(folderId: string | null) => {
			void navigate({
				to: buildOrgPath(organizationSlug, '/connaissances'),
				search: folderId ? { dossier: folderId } : {},
			})
		},
		[navigate, organizationSlug],
	)
}

function DocumentLoader({
	organizationId,
	organizationSlug,
	articleId,
}: LoaderProps) {
	const query = useKnowledgeDocument(organizationId, articleId)
	const openDrive = useDriveNavigation(organizationSlug)
	const [loaded, setLoaded] = useState<KnowledgeArticle | null>(null)
	const [generation, setGeneration] = useState(0)

	useEffect(() => {
		if (query.data && loaded === null) setLoaded(query.data)
	}, [query.data, loaded])

	const reload = useCallback(async () => {
		const result = await query.refetch()
		if (result.data) {
			setLoaded(result.data)
			setGeneration((current) => current + 1)
		}
	}, [query.refetch])

	if (query.isError && loaded === null) {
		return (
			<DocumentUnavailable
				message={
					mutationErrorMessage(query.error) ?? 'Ce document est introuvable.'
				}
				onBack={() => openDrive(null)}
			/>
		)
	}
	if (loaded === null) return <DocumentLoading />
	if (loaded.kind !== 'document') {
		return (
			<DocumentUnavailable
				message="Cet élément n’est pas un document."
				onBack={() => openDrive(loaded.folder_id ?? null)}
			/>
		)
	}
	return (
		<DocumentWorkspace
			key={generation}
			organizationId={organizationId}
			organizationSlug={organizationSlug}
			loaded={loaded}
			live={query.data ?? loaded}
			onReload={() => {
				void reload()
			}}
		/>
	)
}

interface WorkspaceProps {
	organizationId: string
	organizationSlug: string
	loaded: KnowledgeArticle
	live: KnowledgeArticle
	onReload: () => void
}

function DocumentWorkspace({
	organizationId,
	organizationSlug,
	loaded,
	live,
	onReload,
}: WorkspaceProps) {
	const navigate = useNavigate()
	const openDrive = useDriveNavigation(organizationSlug)
	const canManage = useHasPermission('MANAGE_KNOWLEDGE')
	const [infoOpen, setInfoOpen] = useState(false)
	const [titleDraft, setTitleDraft] = useState<string | null>(null)
	const [tagInput, setTagInput] = useState('')
	const [moving, setMoving] = useState<string | null | undefined>(undefined)
	const [confirmDelete, setConfirmDelete] = useState(false)

	const session = useDocumentSession({
		organizationId,
		articleId: loaded.id,
		canEdit: canManage,
		loadedRevision: loaded.revision ?? 0,
	})
	const directory = useChatDirectory(organizationId)
	const folders = useKnowledgeFolders(organizationId)
	const resources = useResourceOptions(organizationId)
	const patch = usePatchKnowledgeArticle(organizationId)
	const remove = useDeleteKnowledgeArticle(organizationId)
	const link = useLinkKnowledgeArticle(organizationId)
	const unlink = useUnlinkKnowledgeArticle(organizationId)
	const move = useMoveKnowledgeArticles(organizationId)

	const folderId = live.folder_id ?? null
	const path = useMemo(
		() => pathTo(folders.data ?? [], folderId),
		[folders.data, folderId],
	)
	const linkLabels = useMemo(
		() =>
			(live.links ?? []).map((target) => ({
				key: `${target.resource_type}:${target.resource_id}`,
				label: `${RESOURCE_TYPE_LABELS[target.resource_type]} · ${resourceLabel(
					resources,
					target.resource_type,
					target.resource_id,
				)}`,
			})),
		[live.links, resources],
	)
	const initialBlocks = useMemo(
		() => toEditorBlocks(loaded.content),
		[loaded.content],
	)

	const environment = useMemo<DocumentEnvironment>(
		() => ({
			organizationId,
			folderId,
			editable: session.state.phase === 'editing',
			resourceHref: (kind, id) =>
				buildOrgPath(organizationSlug, `${RESOURCE_PATHS[kind]}/${id}`),
			openResource: (kind, id) => {
				void navigate({
					to: buildOrgPath(organizationSlug, `${RESOURCE_PATHS[kind]}/${id}`),
				})
			},
			openTable: (tableId, viewId) => {
				void navigate({
					to: buildOrgPath(
						organizationSlug,
						`/connaissances/tables/${tableId}`,
					),
					search: { vue: viewId },
				})
			},
		}),
		[organizationId, organizationSlug, folderId, navigate, session.state.phase],
	)

	const holderName =
		directory.data?.find((entry) => entry.user_id === session.state.holderId)
			?.display_name ?? 'un collègue'
	const phase = session.state.phase
	const banner: BannerKind | null =
		phase === 'locked'
			? { kind: 'locked', holderName }
			: phase === 'conflict' && session.state.conflict
				? { kind: 'conflict', conflict: session.state.conflict }
				: phase === 'failed'
					? { kind: 'failed' }
					: null

	const commitTitle = () => {
		if (titleDraft === null) return
		const next = titleDraft.trim()
		setTitleDraft(null)
		if (next !== '' && next !== live.title) {
			patch.mutate({ id: live.id, patch: { title: next } })
		}
	}

	const savePatchedTags = (tags: string[]) =>
		patch.mutate({ id: live.id, patch: { tags } })

	const confirmedDelete = async () => {
		try {
			await remove.mutateAsync(live.id)
			setConfirmDelete(false)
			openDrive(folderId)
		} catch {}
	}

	const confirmedMove = async () => {
		if (moving === undefined) return
		try {
			await move.mutateAsync({ articleIds: [live.id], folderId: moving })
			setMoving(undefined)
		} catch {}
	}

	return (
		<>
			<DocumentPage
				title={titleDraft ?? live.title}
				canManage={canManage}
				path={path}
				indicator={
					<SaveIndicator
						state={session.state.saveState}
						error={session.state.saveError}
					/>
				}
				properties={
					<DocumentProperties
						canManage={canManage}
						tags={live.tags}
						links={linkLabels}
						onOpenInfo={() => setInfoOpen(true)}
					/>
				}
				banner={
					banner ? <DocumentBanner banner={banner} onReload={onReload} /> : null
				}
				onTitleChange={setTitleDraft}
				onTitleCommit={commitTitle}
				onOpenFolder={openDrive}
				onOpenInfo={() => setInfoOpen(true)}
				onMove={() => setMoving(folderId)}
				onDelete={() => setConfirmDelete(true)}
			>
				<DocumentEditor
					initialBlocks={initialBlocks}
					editable={phase === 'editing'}
					environment={environment}
					onChange={session.change}
				/>
			</DocumentPage>
			<DocumentInfoPanel
				open={infoOpen}
				canManage={canManage}
				tags={live.tags}
				tagInput={tagInput}
				links={live.links ?? []}
				resources={resources}
				isLinking={link.isPending || unlink.isPending}
				error={mutationErrorMessage(patch.error ?? link.error ?? unlink.error)}
				onOpenChange={setInfoOpen}
				onTagInputChange={setTagInput}
				onAddTags={() => {
					savePatchedTags(addTags(live.tags, tagInput))
					setTagInput('')
				}}
				onRemoveTag={(tag) => savePatchedTags(removeTag(live.tags, tag))}
				onLink={(resourceType, resourceId) =>
					link.mutate({ articleId: live.id, resourceType, resourceId })
				}
				onUnlink={(target) =>
					unlink.mutate({
						articleId: live.id,
						resourceType: target.resource_type,
						resourceId: target.resource_id,
					})
				}
			/>
			<FolderPickerDialog
				open={moving !== undefined}
				title="Déplacer le document"
				description="Choisissez le dossier de destination."
				folders={folders.data ?? []}
				selected={moving ?? null}
				isAllowed={() => true}
				error={mutationErrorMessage(move.error)}
				isSaving={move.isPending}
				onSelect={setMoving}
				onConfirm={() => {
					void confirmedMove()
				}}
				onOpenChange={(open) => {
					if (!open) {
						setMoving(undefined)
						move.reset()
					}
				}}
			/>
			<ConfirmDialog
				open={confirmDelete}
				onOpenChange={setConfirmDelete}
				title="Supprimer ce document ?"
				description="Il disparaît de la base de connaissances. Les projets, clients et devis auxquels il était rattaché ne changent pas."
				confirmLabel="Supprimer"
				destructive
				disabled={remove.isPending}
				onConfirm={() => {
					void confirmedDelete()
				}}
			>
				{remove.error ? (
					<p role="alert" className="text-[13px] text-state-alert">
						{mutationErrorMessage(remove.error)}
					</p>
				) : null}
			</ConfirmDialog>
		</>
	)
}

import { createReactBlockSpec } from '@blocknote/react'
import { useState } from 'react'
import { useFileUrls } from '#/hooks/use-file-url'
import { downloadKnowledgeFile } from '#/hooks/use-knowledge'
import {
	useDriveFileInfo,
	useDriveImageUrl,
} from '#/hooks/use-knowledge-documents'
import { useKnowledgeTable, useTableRows } from '#/hooks/use-knowledge-tables'
import { usePhotoLibraryPhoto } from '#/hooks/use-photo-library'
import { fileTypeLabel, formatSize } from '#/pages/knowledge/lib/files'
import { useDocumentEnvironment } from '#/pages/knowledge-documents/feature/document-environment'
import {
	FilePickerFeature,
	ImagePickerFeature,
	TableViewPickerFeature,
} from '#/pages/knowledge-documents/feature/document-pickers'
import type { CalloutTone } from '#/pages/knowledge-documents/lib/document-api-types'
import { CalloutBlock } from '#/pages/knowledge-documents/ui/callout-block'
import { FileBlock } from '#/pages/knowledge-documents/ui/file-block'
import {
	ImageBlock,
	type ImageBlockState,
} from '#/pages/knowledge-documents/ui/image-block'
import {
	TableViewBlock,
	type TableViewBlockState,
} from '#/pages/knowledge-documents/ui/table-view-block'
import { orderedColumns } from '#/pages/knowledge-tables/lib/columns'
import {
	normalizeConfig,
	visibleColumns,
} from '#/pages/knowledge-tables/lib/view-config'

export const createCalloutBlock = createReactBlockSpec(
	{
		type: 'callout',
		propSchema: { tone: { default: 'info' } },
		content: 'inline',
	},
	{
		render: ({ block, editor, contentRef }) => {
			const environment = useDocumentEnvironment()
			return (
				<CalloutBlock
					tone={block.props.tone as CalloutTone}
					editable={environment.editable}
					contentRef={contentRef}
					onToneChange={(tone) =>
						editor.updateBlock(block, { props: { tone } })
					}
				/>
			)
		},
	},
)

export const createImageBlock = createReactBlockSpec(
	{
		type: 'image',
		propSchema: {
			source: { default: 'drive' },
			id: { default: '' },
			caption: { default: '' },
			width: { default: 100 },
		},
		content: 'none',
	},
	{
		render: ({ block, editor }) => {
			const environment = useDocumentEnvironment()
			const [picking, setPicking] = useState(false)
			const { source, id, caption, width } = block.props
			const isPhoto = source === 'photo'
			const drive = useDriveImageUrl(
				environment.organizationId,
				isPhoto ? '' : id,
			)
			const photo = usePhotoLibraryPhoto(
				environment.organizationId,
				isPhoto && id !== '' ? id : null,
			)
			const signed = useFileUrls(photo.data ? [photo.data.file_key] : [])[0]

			let state: ImageBlockState = 'loading'
			let url: string | null = null
			if (id === '') {
				state = 'empty'
			} else if (isPhoto) {
				if (photo.isError) state = 'missing'
				else if (signed?.url) {
					state = 'ready'
					url = signed.url
				}
			} else if (drive.isError) {
				state = 'missing'
			} else if (drive.data) {
				state = 'ready'
				url = drive.data
			}

			return (
				<>
					<ImageBlock
						state={state}
						url={url}
						caption={caption}
						width={width}
						editable={environment.editable}
						onPick={() => setPicking(true)}
						onCaptionChange={(next) =>
							editor.updateBlock(block, { props: { caption: next } })
						}
						onWidthChange={(next) =>
							editor.updateBlock(block, { props: { width: next } })
						}
					/>
					{picking ? (
						<ImagePickerFeature
							organizationId={environment.organizationId}
							folderId={environment.folderId}
							onPick={(nextSource, nextId) => {
								editor.updateBlock(block, {
									props: { source: nextSource, id: nextId },
								})
								setPicking(false)
							}}
							onClose={() => setPicking(false)}
						/>
					) : null}
				</>
			)
		},
	},
)

export const createFileBlock = createReactBlockSpec(
	{
		type: 'file',
		propSchema: { articleId: { default: '' } },
		content: 'none',
	},
	{
		render: ({ block, editor }) => {
			const environment = useDocumentEnvironment()
			const [picking, setPicking] = useState(false)
			const [isDownloading, setIsDownloading] = useState(false)
			const { articleId } = block.props
			const info = useDriveFileInfo(environment.organizationId, articleId)

			const state =
				articleId === ''
					? 'empty'
					: info.isError
						? 'missing'
						: info.data
							? 'ready'
							: 'loading'

			return (
				<>
					<FileBlock
						state={state}
						title={info.data?.title ?? ''}
						detail={
							info.data
								? `${fileTypeLabel(info.data.mimeType)} · ${formatSize(info.data.sizeBytes)}`
								: ''
						}
						editable={environment.editable}
						isDownloading={isDownloading}
						onPick={() => setPicking(true)}
						onDownload={() => {
							if (!info.data) return
							setIsDownloading(true)
							void downloadKnowledgeFile(environment.organizationId, {
								id: articleId,
								title: info.data.title,
							}).finally(() => setIsDownloading(false))
						}}
					/>
					{picking ? (
						<FilePickerFeature
							organizationId={environment.organizationId}
							onPick={(nextId) => {
								editor.updateBlock(block, { props: { articleId: nextId } })
								setPicking(false)
							}}
							onClose={() => setPicking(false)}
						/>
					) : null}
				</>
			)
		},
	},
)

export const createTableViewBlock = createReactBlockSpec(
	{
		type: 'tableView',
		propSchema: { tableId: { default: '' }, viewId: { default: '' } },
		content: 'none',
	},
	{
		render: ({ block, editor }) => {
			const environment = useDocumentEnvironment()
			const [picking, setPicking] = useState(false)
			const { tableId, viewId } = block.props
			const detail = useKnowledgeTable(environment.organizationId, tableId)
			const view = detail.data?.views.find((entry) => entry.id === viewId)
			const rows = useTableRows(
				environment.organizationId,
				tableId,
				view ? { kind: 'view', viewId: view.id } : null,
			)

			let state: TableViewBlockState = 'loading'
			if (tableId === '' || viewId === '') state = 'empty'
			else if (detail.isError || (detail.data && !view)) state = 'missing'
			else if (detail.data && view) state = 'ready'

			const columns = orderedColumns(detail.data?.columns ?? [])
			const hidden = view ? normalizeConfig(view.config).hidden : []
			const pages = rows.data?.pages ?? []

			return (
				<>
					<TableViewBlock
						state={state}
						tableName={detail.data?.name ?? ''}
						viewName={view?.name ?? ''}
						columns={visibleColumns(columns, hidden)}
						rows={pages.flatMap((page) => page.rows)}
						total={pages.at(-1)?.total ?? 0}
						isLoadingRows={rows.isLoading}
						editable={environment.editable}
						onOpen={() => environment.openTable(tableId, viewId)}
						onPick={() => setPicking(true)}
					/>
					{picking ? (
						<TableViewPickerFeature
							organizationId={environment.organizationId}
							onPick={(nextTable, nextView) => {
								editor.updateBlock(block, {
									props: { tableId: nextTable, viewId: nextView },
								})
								setPicking(false)
							}}
							onClose={() => setPicking(false)}
						/>
					) : null}
				</>
			)
		},
	},
)

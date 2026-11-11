import { useMemo, useState } from 'react'
import { useFileUrls } from '#/hooks/use-file-url'
import {
	useKnowledgeFolders,
	useUploadKnowledgeFile,
} from '#/hooks/use-knowledge'
import { useDriveFiles, useDriveImages } from '#/hooks/use-knowledge-documents'
import {
	useKnowledgeTable,
	useKnowledgeTables,
} from '#/hooks/use-knowledge-tables'
import { type LibraryPhoto, usePhotoLibrary } from '#/hooks/use-photo-library'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	mimeTypeOf,
	refusalOf,
	titleFromFileName,
} from '#/pages/knowledge/lib/files'
import { useDebounced } from '#/pages/knowledge-documents/feature/use-debounced'
import type { ImageSource } from '#/pages/knowledge-documents/lib/document-api-types'
import { ChoiceDialog } from '#/pages/knowledge-documents/ui/choice-dialog'
import {
	ImagePickerDialog,
	type ImagePickerTab,
} from '#/pages/knowledge-documents/ui/image-picker-dialog'
import { EMPTY_FILTERS } from '#/pages/photo-library/lib/filters'

const SEARCH_DELAY_MS = 300

interface ImagePickerProps {
	organizationId: string
	folderId: string | null
	onPick: (source: ImageSource, id: string) => void
	onClose: () => void
}

export function ImagePickerFeature({
	organizationId,
	folderId,
	onPick,
	onClose,
}: ImagePickerProps) {
	const [tab, setTab] = useState<ImagePickerTab>('drive')
	const [query, setQuery] = useState('')
	const [uploadError, setUploadError] = useState<string | null>(null)
	const debounced = useDebounced(query, SEARCH_DELAY_MS)

	const folders = useKnowledgeFolders(organizationId)
	const folderName =
		folders.data?.find((folder) => folder.id === folderId)?.name ?? 'Drive'
	const drive = useDriveImages(organizationId, debounced, tab === 'drive')
	const library = usePhotoLibrary(
		organizationId,
		useMemo(() => ({ ...EMPTY_FILTERS, q: debounced.trim() }), [debounced]),
		1,
		{ enabled: tab === 'photo' },
	)
	const photos = library.data?.photos ?? []
	const previews = useFileUrls(photos.map((photo) => photo.thumbnail_key))
	const thumbnailUrls = useMemo(() => {
		const urls: Record<string, string | undefined> = {}
		for (const preview of previews) urls[preview.key] = preview.url
		return urls
	}, [previews])
	const upload = useUploadKnowledgeFile(organizationId)

	const uploadImage = async (file: File) => {
		setUploadError(null)
		const mimeType = mimeTypeOf(file)
		const refusal = refusalOf(file)
		if (
			refusal !== null ||
			mimeType === null ||
			!mimeType.startsWith('image/')
		) {
			setUploadError(refusal ?? 'Choisissez une image JPEG, PNG ou WebP.')
			return
		}
		try {
			const article = await upload.mutateAsync({
				file: file.slice(0, file.size, mimeType),
				mimeType,
				title: titleFromFileName(file.name),
				folderId,
			})
			onPick('drive', article.id)
		} catch (error) {
			setUploadError(mutationErrorMessage(error))
		}
	}

	return (
		<ImagePickerDialog
			open
			tab={tab}
			query={query}
			driveImages={(drive.data ?? []).map((image) => ({
				id: image.id,
				label: image.title,
			}))}
			photos={photos}
			thumbnailUrls={thumbnailUrls}
			isLoading={tab === 'drive' ? drive.isLoading : library.isLoading}
			isUploading={upload.isPending}
			error={
				uploadError ??
				mutationErrorMessage(tab === 'drive' ? drive.error : library.error)
			}
			folderName={folderName}
			onTabChange={(next) => {
				setTab(next)
				setQuery('')
				setUploadError(null)
			}}
			onQueryChange={setQuery}
			onPickDrive={(id) => onPick('drive', id)}
			onPickPhoto={(photo: LibraryPhoto) => onPick('photo', photo.id)}
			onUpload={(file) => {
				void uploadImage(file)
			}}
			onOpenChange={(open) => {
				if (!open) onClose()
			}}
		/>
	)
}

interface FilePickerProps {
	organizationId: string
	onPick: (articleId: string) => void
	onClose: () => void
}

export function FilePickerFeature({
	organizationId,
	onPick,
	onClose,
}: FilePickerProps) {
	const [query, setQuery] = useState('')
	const debounced = useDebounced(query, SEARCH_DELAY_MS)
	const files = useDriveFiles(organizationId, debounced, true)
	return (
		<ChoiceDialog
			open
			title="Choisir un fichier"
			description="Un fichier du Drive à joindre au document."
			query={query}
			items={(files.data ?? []).map((file) => ({
				id: file.id,
				label: file.title,
			}))}
			isLoading={files.isLoading}
			emptyText="Aucun fichier dans le Drive."
			onQueryChange={setQuery}
			onPick={onPick}
			onOpenChange={(open) => {
				if (!open) onClose()
			}}
		/>
	)
}

interface TableViewPickerProps {
	organizationId: string
	onPick: (tableId: string, viewId: string) => void
	onClose: () => void
}

export function TableViewPickerFeature({
	organizationId,
	onPick,
	onClose,
}: TableViewPickerProps) {
	const [query, setQuery] = useState('')
	const [tableId, setTableId] = useState<string | null>(null)
	const tables = useKnowledgeTables(organizationId)
	const detail = useKnowledgeTable(organizationId, tableId ?? '')
	const needle = query.trim().toLowerCase()
	const items = (tables.data ?? [])
		.filter((table) => table.name.toLowerCase().includes(needle))
		.map((table) => ({ id: table.id, label: table.name }))

	if (tableId !== null) {
		const views = [...(detail.data?.views ?? [])].sort(
			(a, b) => a.position - b.position,
		)
		return (
			<ChoiceDialog
				open
				title="Choisir une vue"
				description={detail.data?.name ?? 'Vue de la table'}
				query={null}
				items={views.map((view) => ({ id: view.id, label: view.name }))}
				isLoading={detail.isLoading}
				emptyText="Cette table n’a aucune vue."
				onQueryChange={() => {}}
				onPick={(viewId) => onPick(tableId, viewId)}
				onOpenChange={(open) => {
					if (!open) setTableId(null)
				}}
			/>
		)
	}

	return (
		<ChoiceDialog
			open
			title="Choisir une table"
			description="La table dont le document affiche une vue, en lecture."
			query={query}
			items={items}
			isLoading={tables.isLoading}
			emptyText="Aucune table."
			onQueryChange={setQuery}
			onPick={setTableId}
			onOpenChange={(open) => {
				if (!open) onClose()
			}}
		/>
	)
}

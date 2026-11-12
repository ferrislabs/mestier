import { useEffect, useMemo, useRef, useState } from 'react'
import { useUploadFile } from '#/hooks/use-customers'
import { useFileUrls } from '#/hooks/use-file-url'
import {
	type LibraryPhoto,
	useCopyLibraryPhoto,
	usePhotoLibrary,
} from '#/hooks/use-photo-library'
import { mutationErrorMessage } from '#/lib/api-error'
import { useStylePreviewPages } from '#/pages/doe-style/feature/use-style-preview-pages'
import {
	blockRect,
	type CoverBlock,
	type CoverBlockKey,
	type CoverLayout,
	coverBlockErrors,
	defaultCoverLayout,
	setBlockRect,
	validateCoverLayout,
} from '#/pages/doe-style/lib/cover-layout'
import {
	addDecoration,
	appendDecoration,
	appliesToPage,
	createImageDecoration,
	type Decoration,
	type DecorationLayer,
	decorationErrors,
	duplicateDecoration,
	nudgeRect,
	pagesOf,
	type Rect,
	rectOf,
	removeDecoration,
	reorderDecoration,
	reorderedIndex,
	updateDecoration,
	validateDecorations,
} from '#/pages/doe-style/lib/decorations'
import {
	canRedo,
	canUndo,
	createHistory,
	push,
	redo,
	undo,
} from '#/pages/doe-style/lib/history'
import { shortcutFor } from '#/pages/doe-style/lib/shortcuts'
import type { CompleteDoeStyle } from '#/pages/doe-style/lib/style'
import { rotateBy, toggleFlip } from '#/pages/doe-style/lib/transform'
import { LayoutEditor } from '#/pages/doe-style/ui/layout-editor'
import { PhotoLibraryPicker } from '#/pages/doe-style/ui/photo-library-picker'
import { EMPTY_FILTERS } from '#/pages/photo-library/lib/filters'

const SEARCH_DELAY_MS = 300
const EDITABLE_TARGET =
	'input, textarea, select, [contenteditable="true"], [role="combobox"], [role="listbox"], [role="option"]'

interface LayoutEditorFeatureProps {
	organizationId: string
	style: CompleteDoeStyle
	accentColor: string
	onApply: (decorations: Decoration[], coverLayout: CoverLayout | null) => void
	onCancel: () => void
}

interface EditorState {
	decorations: Decoration[]
	coverLayout: CoverLayout | null
}

function useDebounced<T>(value: T, delay: number): T {
	const [debounced, setDebounced] = useState(value)
	useEffect(() => {
		const timer = setTimeout(() => setDebounced(value), delay)
		return () => clearTimeout(timer)
	}, [value, delay])
	return debounced
}

async function aspectRatioOf(file: Blob): Promise<number> {
	try {
		const bitmap = await createImageBitmap(file)
		const ratio = bitmap.width / bitmap.height
		bitmap.close()
		return ratio
	} catch {
		return Number.NaN
	}
}

export function LayoutEditorFeature({
	organizationId,
	style,
	accentColor,
	onApply,
	onCancel,
}: LayoutEditorFeatureProps) {
	const [history, setHistory] = useState(() =>
		createHistory<EditorState>({
			decorations: style.decorations,
			coverLayout: style.cover_layout,
		}),
	)
	const [selection, setSelection] = useState<number | null>(null)
	const [blockSelection, setBlockSelection] = useState<CoverBlockKey | null>(
		null,
	)
	const [activePage, setActivePage] = useState(0)
	const [picker, setPicker] = useState<'add' | 'replace' | null>(null)
	const [search, setSearch] = useState('')
	const decorations = history.present.decorations
	const coverLayout = history.present.coverLayout
	const latest = useRef(decorations)
	latest.current = decorations
	const latestLayout = useRef(coverLayout)
	latestLayout.current = coverLayout
	const latestBlock = useRef<CoverBlockKey | null>(null)
	const latestSelection = useRef<number | null>(null)

	const upload = useUploadFile()
	const copy = useCopyLibraryPhoto(organizationId)
	const debouncedSearch = useDebounced(search, SEARCH_DELAY_MS)
	const filters = useMemo(
		() => ({ ...EMPTY_FILTERS, q: debouncedSearch.trim() }),
		[debouncedSearch],
	)
	const library = usePhotoLibrary(organizationId, filters, 1, {
		enabled: picker !== null,
	})
	const photos = library.data?.photos ?? []
	const thumbnails = useFileUrls(photos.map((photo) => photo.thumbnail_key))
	const thumbnailUrls = useMemo(() => {
		const urls: Record<string, string | undefined> = {}
		for (const thumbnail of thumbnails) urls[thumbnail.key] = thumbnail.url
		return urls
	}, [thumbnails])

	const live = useStylePreviewPages(organizationId, {
		...style,
		decorations,
		cover_layout: coverLayout,
	})

	const selectedIndex =
		selection !== null && selection < decorations.length ? selection : null
	latestSelection.current = selectedIndex
	const selected =
		selectedIndex === null ? undefined : decorations[selectedIndex]
	const [selectedImage] = useFileUrls(
		selected?.kind === 'image' && selected.key ? [selected.key] : [],
	)
	const selectedBlock =
		activePage === 0 && coverLayout !== null ? blockSelection : null
	latestBlock.current = selectedBlock
	const validation =
		validateDecorations(decorations) ?? validateCoverLayout(coverLayout)

	function commit(next: Decoration[]) {
		setHistory((current) =>
			next === current.present.decorations
				? current
				: push(current, { ...current.present, decorations: next }),
		)
	}

	function commitLayout(next: CoverLayout | null) {
		setHistory((current) =>
			next === current.present.coverLayout
				? current
				: push(current, { ...current.present, coverLayout: next }),
		)
	}

	function selectBlock(key: CoverBlockKey) {
		setBlockSelection(key)
		setSelection(null)
		setActivePage(0)
	}

	function selectPage(index: number) {
		setActivePage(index)
		if (index !== 0) setBlockSelection(null)
	}

	function customizeCover() {
		if (latestLayout.current === null) commitLayout(defaultCoverLayout())
	}

	function resetCover() {
		setBlockSelection(null)
		commitLayout(null)
	}

	function updateBlock(key: CoverBlockKey, next: CoverBlock) {
		const layout = latestLayout.current
		if (layout) commitLayout({ ...layout, [key]: next })
	}

	function commitBlockRect(key: CoverBlockKey, rect: Rect) {
		const layout = latestLayout.current
		if (layout) commitLayout(setBlockRect(layout, key, rect))
	}

	function select(index: number | null) {
		setSelection(index)
		if (index !== null) setBlockSelection(null)
		const target = index === null ? undefined : latest.current[index]
		if (target && !appliesToPage(target, activePage)) {
			setActivePage(pagesOf(target) === 'inner' ? 1 : 0)
		}
	}

	function appendAndSelect(decoration: Decoration) {
		const base = latest.current
		const next = appendDecoration(base, decoration)
		if (next === base) return
		commit(next)
		setBlockSelection(null)
		setSelection(base.length)
	}

	function updateAt(
		index: number,
		change: (current: Decoration) => Decoration,
	) {
		const current = latest.current[index]
		if (!current) return
		commit(updateDecoration(latest.current, index, change(current)))
	}

	function updateSelected(change: (current: Decoration) => Decoration) {
		const index = latestSelection.current
		if (index !== null) updateAt(index, change)
	}

	function commitRect(index: number, rect: Rect) {
		updateAt(index, (current) => ({ ...current, ...rect }))
	}

	function replaceKey(index: number, key: string) {
		updateAt(index, (current) =>
			current.kind === 'image' ? { ...current, key } : current,
		)
	}

	function move(direction: 'forward' | 'backward') {
		const index = latestSelection.current
		if (index === null) return
		commit(reorderDecoration(latest.current, index, direction))
		setSelection(reorderedIndex(latest.current, index, direction))
	}

	function duplicate() {
		const index = latestSelection.current
		if (index === null) return
		const next = duplicateDecoration(latest.current, index)
		if (next === latest.current) return
		commit(next)
		setSelection(index + 1)
	}

	function remove() {
		const index = latestSelection.current
		if (index === null) return
		commit(removeDecoration(latest.current, index))
		setSelection(null)
	}

	function addImageFile(file: File) {
		void Promise.all([aspectRatioOf(file), upload.mutateAsync(file)])
			.then(([ratio, response]) =>
				appendAndSelect(createImageDecoration(response.data.key, ratio)),
			)
			.catch(() => {})
	}

	function replaceImageFile(file: File) {
		const index = latestSelection.current
		if (index === null) return
		void upload
			.mutateAsync(file)
			.then((response) => replaceKey(index, response.data.key))
			.catch(() => {})
	}

	function pickPhoto(photo: LibraryPhoto) {
		const mode = picker
		const index = latestSelection.current
		void copy
			.mutateAsync(photo.id)
			.then(({ key }) => {
				if (mode === 'replace' && index !== null) replaceKey(index, key)
				else {
					appendAndSelect(
						createImageDecoration(key, photo.width / photo.height),
					)
				}
				setPicker(null)
			})
			.catch(() => {})
	}

	function handleKeyDown(event: React.KeyboardEvent<HTMLElement>) {
		if (event.defaultPrevented) return
		const target = event.target as HTMLElement
		const action = shortcutFor({
			key: event.key,
			ctrlKey: event.ctrlKey,
			metaKey: event.metaKey,
			shiftKey: event.shiftKey,
			editable: target.matches(EDITABLE_TARGET),
		})
		if (!action) return
		event.preventDefault()
		const index = latestSelection.current
		switch (action.type) {
			case 'undo':
				setHistory(undo)
				return
			case 'redo':
				setHistory(redo)
				return
			case 'duplicate':
				duplicate()
				return
			case 'remove':
				remove()
				return
			case 'nudge': {
				const layout = latestLayout.current
				const key = latestBlock.current
				if (key && layout) {
					const frame = blockRect(layout[key])
					const moved = nudgeRect(frame, action.key, action.shift)
					if (moved.x !== frame.x || moved.y !== frame.y) {
						commitLayout(setBlockRect(layout, key, moved))
					}
					return
				}
				const current = index === null ? undefined : latest.current[index]
				if (index === null || !current) return
				const rect = rectOf(current)
				const next = nudgeRect(rect, action.key, action.shift)
				if (next.x !== rect.x || next.y !== rect.y) commitRect(index, next)
			}
		}
	}

	const message =
		mutationErrorMessage(upload.error) ??
		mutationErrorMessage(copy.error) ??
		live.error ??
		validation ??
		null

	return (
		<>
			<LayoutEditor
				decorations={decorations}
				errors={decorationErrors(decorations)}
				selectedIndex={selectedIndex}
				coverLayout={coverLayout}
				selectedBlock={selectedBlock}
				blockErrors={coverLayout ? coverBlockErrors(coverLayout) : {}}
				pages={live.pages}
				activePage={activePage}
				previewLoading={live.loading}
				message={message}
				imageUrl={selectedImage?.url}
				canUndo={canUndo(history)}
				canRedo={canRedo(history)}
				isUploading={upload.isPending}
				invalid={validation !== undefined}
				onSelect={select}
				onSelectBlock={selectBlock}
				onCommitBlock={commitBlockRect}
				onUpdateBlock={updateBlock}
				onCustomizeCover={customizeCover}
				onResetCover={resetCover}
				onSelectPage={selectPage}
				onAddShape={(kind) => {
					const base = latest.current
					const next = addDecoration(base, kind, accentColor)
					if (next === base) return
					commit(next)
					setSelection(base.length)
				}}
				onAddImageFile={addImageFile}
				onOpenLibrary={(mode) => {
					copy.reset()
					setPicker(mode)
				}}
				onReplaceImageFile={replaceImageFile}
				onUpdateSelected={(next) => updateSelected(() => next)}
				onCommitRect={commitRect}
				onRotateTo={(index, rotation) =>
					updateAt(index, (current) => ({ ...current, rotation }))
				}
				onRotateBy={(delta) =>
					updateSelected((current) => rotateBy(current, delta))
				}
				onToggleFlip={(axis) =>
					updateSelected((current) => toggleFlip(current, axis))
				}
				onSetLayer={(layer: DecorationLayer) =>
					updateSelected((current) => ({ ...current, layer }))
				}
				onMove={move}
				onDuplicate={duplicate}
				onRemove={remove}
				onUndo={() => setHistory(undo)}
				onRedo={() => setHistory(redo)}
				onApply={() => onApply(decorations, coverLayout)}
				onCancel={onCancel}
				onKeyDown={handleKeyDown}
			/>
			<PhotoLibraryPicker
				open={picker !== null}
				photos={photos}
				thumbnailUrls={thumbnailUrls}
				search={search}
				loading={library.isLoading}
				error={
					mutationErrorMessage(copy.error) ??
					mutationErrorMessage(library.error)
				}
				pendingId={copy.isPending ? (copy.variables ?? null) : null}
				onSearchChange={setSearch}
				onPick={pickPhoto}
				onClose={() => setPicker(null)}
			/>
		</>
	)
}

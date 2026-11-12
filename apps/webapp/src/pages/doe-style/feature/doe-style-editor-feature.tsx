import { useRef, useState } from 'react'
import { useUploadFile } from '#/hooks/use-customers'
import { usePreviewDoeStyle } from '#/hooks/use-doe-models'
import { useFileUrls } from '#/hooks/use-file-url'
import { mutationErrorMessage } from '#/lib/api-error'
import { LayoutEditorFeature } from '#/pages/doe-style/feature/layout-editor-feature'
import { useStylePreviewPages } from '#/pages/doe-style/feature/use-style-preview-pages'
import { defaultCoverLayout } from '#/pages/doe-style/lib/cover-layout'
import {
	decorationErrors,
	type Rect,
	updateDecoration,
} from '#/pages/doe-style/lib/decorations'
import {
	type CompleteDoeStyle,
	isValidHex,
	normalizeStyle,
	validateStyle,
} from '#/pages/doe-style/lib/style'
import { activeThemeId, applyTheme } from '#/pages/doe-style/lib/themes'
import { DoeStyleForm } from '#/pages/doe-style/ui/doe-style-form'

interface DoeStyleEditorFeatureProps {
	organizationId: string
	value: CompleteDoeStyle
	onChange: (patch: Partial<CompleteDoeStyle>) => void
}

export function DoeStyleEditorFeature({
	organizationId,
	value,
	onChange,
}: DoeStyleEditorFeatureProps) {
	const upload = useUploadFile()
	const preview = usePreviewDoeStyle(organizationId)
	const [logoPreview] = useFileUrls(value.logo_key ? [value.logo_key] : [])
	const live = useStylePreviewPages(organizationId, value)
	const [activePage, setActivePage] = useState(0)
	const [selection, setSelection] = useState<number | null>(null)
	const [layoutEditorOpen, setLayoutEditorOpen] = useState(false)
	const latest = useRef(value)
	latest.current = value
	const selectedDecoration =
		selection !== null && selection < value.decorations.length
			? selection
			: null
	const selectedItem =
		selectedDecoration === null
			? undefined
			: value.decorations[selectedDecoration]
	const [decorationImage] = useFileUrls(
		selectedItem?.kind === 'image' && selectedItem.key
			? [selectedItem.key]
			: [],
	)
	const errors = validateStyle(value)
	const error =
		live.error ??
		mutationErrorMessage(upload.error) ??
		mutationErrorMessage(preview.error)

	return (
		<div className="flex flex-col gap-3">
			<DoeStyleForm
				value={value}
				errors={errors}
				pages={live.pages}
				activePage={activePage}
				previewLoading={live.loading}
				previewError={error}
				activeThemeId={activeThemeId(value)}
				selectedDecoration={selectedDecoration}
				decorationErrors={decorationErrors(value.decorations)}
				decorationImageUrl={decorationImage?.url}
				isUploadingDecorationImage={upload.isPending}
				onSelectDecoration={setSelection}
				onCommitDecoration={(index, rect: Rect) => {
					const current = latest.current.decorations[index]
					if (!current) return
					onChange({
						decorations: updateDecoration(latest.current.decorations, index, {
							...current,
							...rect,
						}),
					})
				}}
				onRotateDecoration={(index, rotation) => {
					const current = latest.current.decorations[index]
					if (!current) return
					onChange({
						decorations: updateDecoration(latest.current.decorations, index, {
							...current,
							rotation,
						}),
					})
				}}
				onOpenLayoutEditor={() => setLayoutEditorOpen(true)}
				onDecorationImageUpload={(index, file) => {
					void upload
						.mutateAsync(file)
						.then((response) => {
							const current = latest.current.decorations[index]
							if (current?.kind !== 'image') return
							onChange({
								decorations: updateDecoration(
									latest.current.decorations,
									index,
									{ ...current, key: response.data.key },
								),
							})
						})
						.catch(() => {})
				}}
				onSelectPage={setActivePage}
				onApplyTheme={(theme) => onChange(applyTheme(value, theme))}
				onCustomizeCover={() =>
					onChange({ cover_layout: defaultCoverLayout() })
				}
				onResetCover={() => onChange({ cover_layout: null })}
				logoUrl={logoPreview?.url}
				isUploadingLogo={upload.isPending}
				previewing={preview.isPending}
				previewDisabled={Object.keys(errors).length > 0}
				onChange={onChange}
				onPreview={() => {
					preview.mutate(normalizeStyle(value))
				}}
				onLogoUpload={(file) => {
					void upload
						.mutateAsync(file)
						.then((response) => onChange({ logo_key: response.data.key }))
						.catch(() => {})
				}}
				onLogoRemove={() => onChange({ logo_key: null })}
			/>
			{layoutEditorOpen ? (
				<LayoutEditorFeature
					organizationId={organizationId}
					style={value}
					accentColor={
						isValidHex(value.primary)
							? value.primary.trim().toLowerCase()
							: '#0f3d36'
					}
					onApply={(decorations, cover_layout) => {
						onChange({ decorations, cover_layout })
						setLayoutEditorOpen(false)
					}}
					onCancel={() => setLayoutEditorOpen(false)}
				/>
			) : null}
		</div>
	)
}

import { useState } from 'react'
import { useUploadFile } from '#/hooks/use-customers'
import { usePreviewDoeStyle } from '#/hooks/use-doe-models'
import { useFileUrls } from '#/hooks/use-file-url'
import { mutationErrorMessage } from '#/lib/api-error'
import { useStylePreviewPages } from '#/pages/doe-style/feature/use-style-preview-pages'
import {
	type CompleteDoeStyle,
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
				onSelectPage={setActivePage}
				onApplyTheme={(theme) => onChange(applyTheme(value, theme))}
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
		</div>
	)
}

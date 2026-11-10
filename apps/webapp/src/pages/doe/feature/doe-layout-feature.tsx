import {
	useProjectDoeStyle,
	useSaveProjectDoeStyle,
} from '#/hooks/use-doe-models'
import { LayoutEditorFeature } from '#/pages/doe-style/feature/layout-editor-feature'
import {
	completeStyle,
	isValidHex,
	normalizeStyle,
} from '#/pages/doe-style/lib/style'

interface DoeLayoutFeatureProps {
	organizationId: string
	projectId: string
	onSaved: () => void
	onFailed: (message: string) => void
	onClose: () => void
}

/**
 * The layout editor opened from a DOE. It edits the project's own style,
 * starting from the inherited one, and saves on Valider.
 */
export function DoeLayoutFeature({
	organizationId,
	projectId,
	onSaved,
	onFailed,
	onClose,
}: DoeLayoutFeatureProps) {
	const current = useProjectDoeStyle(organizationId, projectId)
	const save = useSaveProjectDoeStyle()
	const data = current.data?.data
	if (!data) return null
	const style = completeStyle(data.style ?? data.inherited)

	return (
		<LayoutEditorFeature
			organizationId={organizationId}
			style={style}
			accentColor={
				isValidHex(style.primary)
					? style.primary.trim().toLowerCase()
					: '#0f3d36'
			}
			onApply={(decorations) => {
				onClose()
				save
					.mutateAsync({
						path: { organization_id: organizationId, project_id: projectId },
						body: { style: normalizeStyle({ ...style, decorations }) },
					})
					.then(onSaved)
					.catch(() => onFailed('La mise en page n’a pas pu être enregistrée.'))
			}}
			onCancel={onClose}
		/>
	)
}

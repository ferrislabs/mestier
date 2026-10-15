import { useEffect, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type ProjectTemplate,
	useArchiveProjectTemplate,
	useCreateProjectTemplate,
	usePatchProjectTemplate,
	useProjectTemplate,
	useProjectTemplates,
	useReplaceProjectTemplateTasks,
	useRestoreProjectTemplate,
} from '#/hooks/use-project-templates'
import {
	buildTemplateTaskShapeRequest,
	emptyTemplateTaskDraft,
	type TemplateTaskDraft,
	templateTaskToDraft,
} from '#/pages/project-templates/lib/template-task-form'
import {
	EMPTY_PROJECT_TEMPLATE_FORM,
	type ProjectTemplateFormValues,
} from '#/pages/project-templates/types'
import { ProjectTemplateEditorDialog } from '#/pages/project-templates/ui/project-template-editor-dialog'
import { ProjectTemplatesList } from '#/pages/project-templates/ui/project-templates-list'

export interface ProjectTemplatesFeatureProps {
	includeArchived: boolean
	onIncludeArchivedChange: (includeArchived: boolean) => void
}

export function ProjectTemplatesFeature(props: ProjectTemplatesFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<ProjectTemplatesWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			{...props}
		/>
	)
}

type Draft = {
	id: string | null
	values: ProjectTemplateFormValues
	tasks: TemplateTaskDraft[]
}

function ProjectTemplatesWorkspace({
	organizationId,
	includeArchived,
	onIncludeArchivedChange,
}: ProjectTemplatesFeatureProps & {
	organizationId: string
}) {
	const [draft, setDraft] = useState<Draft | null>(null)

	const templates = useProjectTemplates(organizationId, { includeArchived })
	const editingTemplate = useProjectTemplate(
		organizationId,
		draft?.id ?? '',
		draft?.id !== null && draft?.id !== undefined,
	)

	const createTemplate = useCreateProjectTemplate()
	const patchTemplate = usePatchProjectTemplate()
	const replaceTasks = useReplaceProjectTemplateTasks()
	const archiveTemplate = useArchiveProjectTemplate()
	const restoreTemplate = useRestoreProjectTemplate()

	const rows: ProjectTemplate[] = templates.data?.data ?? []

	// Seeds the draft's task list once the full template (with its shapes)
	// loads — the list response never carries them (see
	// `ProjectTemplateResponse.tasks`'s own doc comment). Keyed on the fetch
	// result only: `draft`/`setDraft` are read and written inside, and
	// including them would re-run this on every keystroke in the dialog.
	// biome-ignore lint/correctness/useExhaustiveDependencies: intentionally scoped to the fetch result, see comment above
	useEffect(() => {
		if (!draft || draft.id === null) return
		if (draft.tasks.length > 0) return
		const tasks = editingTemplate.data?.data.tasks
		if (!tasks) return
		setDraft((current) =>
			current ? { ...current, tasks: tasks.map(templateTaskToDraft) } : current,
		)
	}, [editingTemplate.data])

	const isPending =
		createTemplate.isPending ||
		patchTemplate.isPending ||
		replaceTasks.isPending ||
		archiveTemplate.isPending ||
		restoreTemplate.isPending

	const error =
		templates.error ??
		createTemplate.error ??
		patchTemplate.error ??
		replaceTasks.error ??
		archiveTemplate.error ??
		restoreTemplate.error

	const submitDraft = async () => {
		if (!draft) return

		const taskShapes = draft.tasks.map(buildTemplateTaskShapeRequest)

		if (draft.id === null) {
			await createTemplate.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					name: draft.values.name.trim(),
					description: draft.values.description.trim() || null,
					tasks: taskShapes,
				},
			})
		} else {
			await patchTemplate.mutateAsync({
				path: {
					organization_id: organizationId,
					project_template_id: draft.id,
				},
				body: {
					name: draft.values.name.trim(),
					description: draft.values.description.trim() || null,
				},
			})
			await replaceTasks.mutateAsync({
				path: {
					organization_id: organizationId,
					project_template_id: draft.id,
				},
				body: { tasks: taskShapes },
			})
		}

		setDraft(null)
	}

	return (
		<>
			<ProjectTemplatesList
				templates={rows}
				includeArchived={includeArchived}
				isLoading={templates.isLoading}
				error={error?.message ?? null}
				onIncludeArchivedChange={onIncludeArchivedChange}
				onCreate={() =>
					setDraft({
						id: null,
						values: EMPTY_PROJECT_TEMPLATE_FORM,
						tasks: [emptyTemplateTaskDraft()],
					})
				}
				onEdit={(template) =>
					setDraft({
						id: template.id,
						values: {
							name: template.name,
							description: template.description ?? '',
						},
						tasks: [],
					})
				}
				onArchive={(template) => {
					void archiveTemplate.mutateAsync({
						path: {
							organization_id: organizationId,
							project_template_id: template.id,
						},
					})
				}}
				onRestore={(template) => {
					void restoreTemplate.mutateAsync({
						path: {
							organization_id: organizationId,
							project_template_id: template.id,
						},
					})
				}}
			/>

			<ProjectTemplateEditorDialog
				open={draft !== null}
				editingName={
					draft?.id
						? (rows.find((template) => template.id === draft.id)?.name ?? null)
						: null
				}
				values={draft?.values ?? EMPTY_PROJECT_TEMPLATE_FORM}
				tasks={draft?.tasks ?? []}
				isPending={isPending}
				error={error?.message ?? null}
				onOpenChange={(open) => {
					if (!open) setDraft(null)
				}}
				onValuesChange={(patch) =>
					setDraft((current) =>
						current
							? { ...current, values: { ...current.values, ...patch } }
							: current,
					)
				}
				onTasksChange={(tasks) =>
					setDraft((current) => (current ? { ...current, tasks } : current))
				}
				onSubmit={() => {
					void submitDraft()
				}}
			/>
		</>
	)
}

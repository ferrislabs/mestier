import { useCallback, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useDoeSettings } from '#/hooks/use-doe-models'
import {
	useArchiveProject,
	usePatchProject,
	useProject,
	useRestoreProject,
} from '#/hooks/use-projects'
import { DoeStyleEditorFeature } from '#/pages/doe-style/feature/doe-style-editor-feature'
import { completeStyle, sameStyle } from '#/pages/doe-style/lib/style'
import {
	saveProjectExtras,
	useProjectExtras,
} from '#/pages/projects/feature/mocks/project-extras'
import {
	saveProjectStyle,
	useProjectStyleOverrides,
} from '#/pages/projects/feature/mocks/project-styles'
import { useProjectFormOptions } from '#/pages/projects/feature/use-project-form-options'
import {
	extrasOfForm,
	fullFormOf,
	sameFullForm,
} from '#/pages/projects/lib/project-form'
import { statusOf } from '#/pages/projects/lib/project-list'
import { useSyncedDraft } from '#/pages/projects/lib/use-synced-draft'
import { optionalId } from '#/pages/projects/types'
import { Page, PageHeader } from '#/pages/projects/ui/page'
import { PageLoading } from '#/pages/projects/ui/page-loading'
import { ProjectCard } from '#/pages/projects/ui/project-card'
import { ProjectNotFound } from '#/pages/projects/ui/project-not-found'
import { StyleCard } from '#/pages/projects/ui/style-card'
import { UnsavedGuard } from '#/pages/projects/ui/unsaved-guard'

export function ProjectSettingsFeature({ projectId }: { projectId: string }) {
	const { activeOrganization } = useActiveOrganization()
	const project = useProject(activeOrganization.id, projectId)

	if (project.isLoading) return <PageLoading label="Chargement des réglages…" />
	if (!project.data?.data)
		return (
			<Page>
				<PageHeader title="Réglages du projet" />
				<div role="alert">
					<ProjectNotFound organizationSlug={activeOrganization.slug} />
				</div>
			</Page>
		)

	return (
		<ProjectSettingsWorkspace
			key={project.data.data.id}
			project={project.data.data}
			organizationId={activeOrganization.id}
		/>
	)
}

function ProjectSettingsWorkspace({
	project,
	organizationId,
}: {
	project: NonNullable<ReturnType<typeof useProject>['data']>['data']
	organizationId: string
}) {
	const extras = useProjectExtras(project.id).data
	const stored = fullFormOf(project, extras)
	const [form, setForm] = useSyncedDraft(stored, sameFullForm)
	const options = useProjectFormOptions(organizationId, form.customerId)
	const patchProject = usePatchProject()
	const archiveProject = useArchiveProject()
	const restoreProject = useRestoreProject()
	const settings = useDoeSettings(organizationId)
	const overrides = useProjectStyleOverrides()
	const defaults = completeStyle(settings.data?.data.style)
	const storedStyle = overrides[project.id] ?? defaults
	const [style, setStyle] = useSyncedDraft(storedStyle, sameStyle)
	const [projectDirty, setProjectDirty] = useState(false)
	const [styleDirty, setStyleDirty] = useState(false)
	const onProjectDirty = useCallback(
		(dirty: boolean) => setProjectDirty(dirty),
		[],
	)
	const onStyleDirty = useCallback((dirty: boolean) => setStyleDirty(dirty), [])
	const archived = statusOf(project) === 'archived'

	return (
		<Page>
			<PageHeader title="Réglages du projet" description={project.name} />
			<div className="grid gap-6">
				<ProjectCard
					projectName={project.name}
					status={statusOf(project)}
					form={form}
					stored={stored}
					customers={options.customers}
					quotes={options.quotes}
					customerContexts={options.customerContexts}
					isCustomerContextsLoading={options.isCustomerContextsLoading}
					people={options.people}
					isSaving={patchProject.isPending}
					error={
						patchProject.error?.message ??
						archiveProject.error?.message ??
						restoreProject.error?.message ??
						null
					}
					onChange={(patch) => setForm((current) => ({ ...current, ...patch }))}
					onSubmit={async () => {
						await patchProject.mutateAsync({
							path: {
								organization_id: organizationId,
								project_id: project.id,
							},
							body: {
								name: form.name.trim(),
								customer_id: optionalId(form.customerId),
								customer_context_id: optionalId(form.customerContextId),
								quote_id: optionalId(form.quoteId),
							},
						})
						await saveProjectExtras(project.id, extrasOfForm(form))
					}}
					onToggleArchive={() => {
						const path = {
							organization_id: organizationId,
							project_id: project.id,
						}
						if (archived) void restoreProject.mutateAsync({ path })
						else void archiveProject.mutateAsync({ path })
					}}
					onDirtyChange={onProjectDirty}
				/>
				<StyleCard
					draft={style}
					stored={storedStyle}
					defaults={defaults}
					editor={
						<DoeStyleEditorFeature
							organizationId={organizationId}
							value={style}
							onChange={(patch) =>
								setStyle((current) => ({ ...current, ...patch }))
							}
						/>
					}
					isSaving={false}
					error={null}
					onReplace={setStyle}
					onSubmit={() => saveProjectStyle(project.id, style)}
					onDirtyChange={onStyleDirty}
				/>
			</div>
			<UnsavedGuard dirty={projectDirty || styleDirty} />
		</Page>
	)
}

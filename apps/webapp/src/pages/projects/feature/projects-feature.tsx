import { useQueryClient } from '@tanstack/react-query'
import { useMemo, useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers } from '#/hooks/use-customers'
import { usePlanning } from '#/hooks/use-planning'
import {
	useCreateProjectTemplate,
	useInstantiateProjectTemplate,
	useProjectTemplate,
	useProjectTemplates,
} from '#/hooks/use-project-templates'
import type { Project } from '#/hooks/use-projects'
import {
	useArchiveProject,
	useCreateProject,
	usePatchProject,
	useProjects,
	useRestoreProject,
} from '#/hooks/use-projects'
import { useQuotes } from '#/hooks/use-quotes'
import { computeWindow } from '#/pages/planning/lib/window'
import { todayIsoDate } from '#/pages/planning/types'
import { buildTemplateTaskShapeRequest } from '#/pages/project-templates/lib/template-task-form'
import { EMPTY_PROJECT_TEMPLATE_FORM } from '#/pages/project-templates/types'
import { ProjectTemplateEditorDialog } from '#/pages/project-templates/ui/project-template-editor-dialog'
import { projectTasksToTemplateDrafts } from '#/pages/projects/lib/save-as-template'
import {
	EMPTY_PROJECT_FORM,
	emptyInstantiateTemplateForm,
	type InstantiateTemplateFormValues,
	optionalId,
	type ProjectFormValues,
} from '#/pages/projects/types'
import { InstantiateTemplateDialog } from '#/pages/projects/ui/instantiate-template-dialog'
import { ProjectFormDialog } from '#/pages/projects/ui/project-form-dialog'
import { ProjectsList } from '#/pages/projects/ui/projects-list'
import { quoteReferenceLabel } from '#/pages/quotes/types'

const TASKS_PATH = '/api/v1/organizations/{organization_id}/tasks'

export interface ProjectsFeatureProps {
	/** From `?projectId=`, so the profitability screen can link at one row. */
	highlightedProjectId?: string
	includeArchived: boolean
	onIncludeArchivedChange: (includeArchived: boolean) => void
}

export function ProjectsFeature(props: ProjectsFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<ProjectsWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			{...props}
		/>
	)
}

type Draft = { id: string | null; values: ProjectFormValues }
type TemplateDraft = {
	projectId: string
	values: { name: string; description: string }
	tasks: ReturnType<typeof projectTasksToTemplateDrafts>
}

function ProjectsWorkspace({
	organizationId,
	organizationName,
	highlightedProjectId,
	includeArchived,
	onIncludeArchivedChange,
}: ProjectsFeatureProps & {
	organizationId: string
	organizationName: string
}) {
	const queryClient = useQueryClient()
	const [search, setSearch] = useState('')
	const [draft, setDraft] = useState<Draft | null>(null)
	const [instantiateDraft, setInstantiateDraft] =
		useState<InstantiateTemplateFormValues | null>(null)
	const [templateDraft, setTemplateDraft] = useState<TemplateDraft | null>(null)
	const [saveAsTemplateError, setSaveAsTemplateError] = useState<string | null>(
		null,
	)
	const [isBuildingTemplateDraft, setIsBuildingTemplateDraft] = useState(false)

	const projects = useProjects(organizationId, { includeArchived })
	const customers = useCustomers(organizationId)
	const quotes = useQuotes(organizationId)
	const templates = useProjectTemplates(organizationId, {
		includeArchived: false,
	})
	const previewTemplate = useProjectTemplate(
		organizationId,
		instantiateDraft?.templateId ?? '',
		Boolean(instantiateDraft?.templateId),
	)
	// Fetched solely for `timezone` — offsets (template instantiation and
	// "save as template") resolve against the organization's own zone, not
	// the browser's. Mirrors `task-list-feature.tsx`'s own use of a
	// range-independent `GET /planning` call for the same reason.
	const planningQuery = usePlanning(
		organizationId,
		computeWindow('day', todayIsoDate()),
	)
	const timeZone = planningQuery.data?.data.timezone ?? 'UTC'

	const createProject = useCreateProject()
	const patchProject = usePatchProject()
	const archiveProject = useArchiveProject()
	const restoreProject = useRestoreProject()
	const instantiateTemplate = useInstantiateProjectTemplate()
	const createTemplate = useCreateProjectTemplate()

	const rows: Project[] = projects.data?.data ?? []
	const customerRows = customers.data?.data ?? []
	const quoteRows = quotes.data?.data ?? []
	const templateRows = templates.data?.data ?? []

	const nameByCustomer = useMemo(() => {
		const index = new Map<string, string>()
		for (const customer of customerRows) index.set(customer.id, customer.name)

		return index
	}, [customerRows])

	const normalizedSearch = search.trim().toLowerCase()
	const visible = useMemo(
		() =>
			rows.filter((project) =>
				project.name.toLowerCase().includes(normalizedSearch),
			),
		[rows, normalizedSearch],
	)

	// Narrowed here rather than in the dialog: which quotes belong to a customer
	// is a data question, and the dialog is presentational.
	const quotesForDraft = useMemo(() => {
		if (!draft || draft.values.customerId === '') return []

		return quoteRows
			.filter((quote) => quote.customer_id === draft.values.customerId)
			.map((quote) => ({
				id: quote.id,
				label: `${quoteReferenceLabel(quote.reference)} — ${quote.title}`,
			}))
	}, [draft, quoteRows])

	const quotesForInstantiate = useMemo(() => {
		if (!instantiateDraft || instantiateDraft.customerId === '') return []

		return quoteRows
			.filter((quote) => quote.customer_id === instantiateDraft.customerId)
			.map((quote) => ({
				id: quote.id,
				label: `${quoteReferenceLabel(quote.reference)} — ${quote.title}`,
			}))
	}, [instantiateDraft, quoteRows])

	const isPending =
		createProject.isPending ||
		patchProject.isPending ||
		archiveProject.isPending ||
		restoreProject.isPending

	const error =
		projects.error ??
		createProject.error ??
		patchProject.error ??
		archiveProject.error ??
		restoreProject.error

	const submitDraft = async () => {
		if (!draft) return

		const body = {
			name: draft.values.name.trim(),
			customer_id: optionalId(draft.values.customerId),
			customer_context_id: null,
			quote_id: optionalId(draft.values.quoteId),
		}

		if (draft.id === null) {
			await createProject.mutateAsync({
				path: { organization_id: organizationId },
				body,
			})
		} else {
			await patchProject.mutateAsync({
				path: { organization_id: organizationId, project_id: draft.id },
				body,
			})
		}

		setDraft(null)
	}

	const submitInstantiate = async () => {
		if (!instantiateDraft || instantiateDraft.templateId === '') return

		await instantiateTemplate.mutateAsync({
			path: {
				organization_id: organizationId,
				project_template_id: instantiateDraft.templateId,
			},
			body: {
				name: instantiateDraft.name.trim(),
				start_date: instantiateDraft.startDate,
				customer_id: optionalId(instantiateDraft.customerId),
				customer_context_id: null,
				quote_id: optionalId(instantiateDraft.quoteId),
			},
		})

		setInstantiateDraft(null)
	}

	/**
	 * Loads a project's whole task set (roots, then every root's subtasks —
	 * `GET /tasks` has no `project_id` filter, so this narrows client-side,
	 * same as the picker capped at 100 elsewhere in this module) and opens
	 * the template builder pre-filled with the offsets it computes.
	 */
	const startSaveAsTemplate = async (project: Project) => {
		setSaveAsTemplateError(null)
		setIsBuildingTemplateDraft(true)
		try {
			const rootPage = await queryClient.fetchQuery(
				window.tanstackApi.get(TASKS_PATH, {
					path: { organization_id: organizationId },
					query: { page: 1, per_page: 100 },
				}).queryOptions,
			)
			const projectRoots = rootPage.data.filter(
				(task) => task.project_id === project.id,
			)
			const childPages = await Promise.all(
				projectRoots
					.filter((root) => (root.child_count ?? 0) > 0)
					.map((root) =>
						queryClient.fetchQuery(
							window.tanstackApi.get(TASKS_PATH, {
								path: { organization_id: organizationId },
								query: { parent_task_id: root.id, page: 1, per_page: 100 },
							}).queryOptions,
						),
					),
			)
			const allTasks: Schemas.TaskResponse[] = [
				...projectRoots,
				...childPages.flatMap((page) => page.data),
			]

			const tasks = projectTasksToTemplateDrafts(allTasks, timeZone)
			if (tasks.length === 0) {
				setSaveAsTemplateError('Ce projet ne porte aucune tâche à enregistrer.')
				return
			}

			setTemplateDraft({
				projectId: project.id,
				values: { name: project.name, description: '' },
				tasks,
			})
		} finally {
			setIsBuildingTemplateDraft(false)
		}
	}

	const submitTemplateDraft = async () => {
		if (!templateDraft) return

		await createTemplate.mutateAsync({
			path: { organization_id: organizationId },
			body: {
				name: templateDraft.values.name.trim(),
				description: templateDraft.values.description.trim() || null,
				tasks: templateDraft.tasks.map(buildTemplateTaskShapeRequest),
			},
		})

		setTemplateDraft(null)
	}

	return (
		<>
			<ProjectsList
				organizationName={organizationName}
				projects={visible}
				customerName={(customerId) => nameByCustomer.get(customerId) ?? null}
				search={search}
				includeArchived={includeArchived}
				highlightedProjectId={highlightedProjectId}
				isLoading={projects.isLoading}
				error={error?.message ?? saveAsTemplateError}
				editingId={draft?.id ?? null}
				isSaving={isPending}
				isBuildingTemplate={isBuildingTemplateDraft}
				onSearchChange={setSearch}
				onIncludeArchivedChange={onIncludeArchivedChange}
				onCreate={() => setDraft({ id: null, values: EMPTY_PROJECT_FORM })}
				onStartFromTemplate={() =>
					setInstantiateDraft(emptyInstantiateTemplateForm(todayIsoDate()))
				}
				onEdit={(project) =>
					setDraft({
						id: project.id,
						values: {
							name: project.name,
							customerId: project.customer_id ?? '',
							quoteId: project.quote_id ?? '',
						},
					})
				}
				onCancelEdit={() => setDraft(null)}
				onSaveEdit={() => {
					void submitDraft()
				}}
				onArchive={(project) => {
					void archiveProject.mutateAsync({
						path: {
							organization_id: organizationId,
							project_id: project.id,
						},
					})
				}}
				onRestore={(project) => {
					void restoreProject.mutateAsync({
						path: {
							organization_id: organizationId,
							project_id: project.id,
						},
					})
				}}
				onSaveAsTemplate={(project) => {
					void startSaveAsTemplate(project)
				}}
			/>

			<ProjectFormDialog
				open={draft !== null}
				editingName={
					draft?.id
						? (rows.find((project) => project.id === draft.id)?.name ?? null)
						: null
				}
				values={draft?.values ?? EMPTY_PROJECT_FORM}
				customers={customerRows.map((customer) => ({
					id: customer.id,
					label: customer.name,
				}))}
				quotes={quotesForDraft}
				isPending={isPending}
				error={error?.message ?? null}
				onOpenChange={(open) => {
					if (!open) setDraft(null)
				}}
				onChange={(patch) =>
					setDraft((current) =>
						current
							? { ...current, values: { ...current.values, ...patch } }
							: current,
					)
				}
				onSubmit={() => {
					void submitDraft()
				}}
			/>

			<InstantiateTemplateDialog
				open={instantiateDraft !== null}
				templates={templateRows.map((template) => ({
					id: template.id,
					label: template.name,
				}))}
				values={
					instantiateDraft ?? emptyInstantiateTemplateForm(todayIsoDate())
				}
				previewTasks={previewTemplate.data?.data.tasks ?? null}
				customers={customerRows.map((customer) => ({
					id: customer.id,
					label: customer.name,
				}))}
				quotes={quotesForInstantiate}
				isPending={instantiateTemplate.isPending}
				error={instantiateTemplate.error?.message ?? null}
				onOpenChange={(open) => {
					if (!open) setInstantiateDraft(null)
				}}
				onChange={(patch) =>
					setInstantiateDraft((current) =>
						current ? { ...current, ...patch } : current,
					)
				}
				onSubmit={() => {
					void submitInstantiate()
				}}
			/>

			<ProjectTemplateEditorDialog
				open={templateDraft !== null}
				editingName={null}
				values={templateDraft?.values ?? EMPTY_PROJECT_TEMPLATE_FORM}
				tasks={templateDraft?.tasks ?? []}
				isPending={createTemplate.isPending}
				error={createTemplate.error?.message ?? null}
				onOpenChange={(open) => {
					if (!open) setTemplateDraft(null)
				}}
				onValuesChange={(patch) =>
					setTemplateDraft((current) =>
						current
							? { ...current, values: { ...current.values, ...patch } }
							: current,
					)
				}
				onTasksChange={(tasks) =>
					setTemplateDraft((current) =>
						current ? { ...current, tasks } : current,
					)
				}
				onSubmit={() => {
					void submitTemplateDraft()
				}}
			/>
		</>
	)
}

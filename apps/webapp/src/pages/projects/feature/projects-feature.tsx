import { useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers } from '#/hooks/use-customers'
import type { Project } from '#/hooks/use-projects'
import {
	useArchiveProject,
	useCreateProject,
	usePatchProject,
	useProjects,
	useRestoreProject,
} from '#/hooks/use-projects'
import { useQuotes } from '#/hooks/use-quotes'
import {
	EMPTY_PROJECT_FORM,
	optionalId,
	type ProjectFormValues,
} from '#/pages/projects/types'
import { ProjectFormDialog } from '#/pages/projects/ui/project-form-dialog'
import { ProjectsList } from '#/pages/projects/ui/projects-list'

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
	const [search, setSearch] = useState('')
	const [draft, setDraft] = useState<Draft | null>(null)

	const projects = useProjects(organizationId, { includeArchived })
	const customers = useCustomers(organizationId)
	const quotes = useQuotes(organizationId)

	const createProject = useCreateProject()
	const patchProject = usePatchProject()
	const archiveProject = useArchiveProject()
	const restoreProject = useRestoreProject()

	const rows: Project[] = projects.data?.data ?? []
	const customerRows = customers.data?.data ?? []
	const quoteRows = quotes.data?.data ?? []

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
				label: `${quote.reference} — ${quote.title}`,
			}))
	}, [draft, quoteRows])

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
				error={error?.message ?? null}
				editingId={draft?.id ?? null}
				isSaving={isPending}
				onSearchChange={setSearch}
				onIncludeArchivedChange={onIncludeArchivedChange}
				onCreate={() => setDraft({ id: null, values: EMPTY_PROJECT_FORM })}
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
		</>
	)
}

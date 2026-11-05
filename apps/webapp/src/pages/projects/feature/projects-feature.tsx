import { useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers } from '#/hooks/use-customers'
import { useDoes } from '#/hooks/use-doe'
import {
	useInstantiateProjectTemplate,
	useProjectTemplate,
	useProjectTemplates,
} from '#/hooks/use-project-templates'
import { useProjects } from '#/hooks/use-projects'
import { useQuotes } from '#/hooks/use-quotes'
import { todayIsoDate } from '#/pages/planning/types'
import { useAllMissions } from '#/pages/projects/feature/mocks/missions'
import { useProjectExtrasMap } from '#/pages/projects/feature/mocks/project-extras'
import { usePeople } from '#/pages/projects/feature/use-people'
import { missionStats } from '#/pages/projects/lib/missions'
import { EMPTY_EXTRAS } from '#/pages/projects/lib/model'
import {
	filterRows,
	listEmpty,
	type ProjectFilters,
	type ProjectRow,
	projectsSummary,
	projectTiles,
	resolveClient,
	statusOf,
} from '#/pages/projects/lib/project-list'
import {
	emptyInstantiateTemplateForm,
	type InstantiateTemplateFormValues,
	optionalId,
} from '#/pages/projects/types'
import { InstantiateTemplateDialog } from '#/pages/projects/ui/instantiate-template-dialog'
import { ProjectsList } from '#/pages/projects/ui/projects-list'
import { quoteReferenceLabel } from '#/pages/quotes/types'

export interface ProjectsFeatureProps {
	filters: ProjectFilters
	highlightedProjectId?: string
	onFiltersChange: (filters: ProjectFilters) => void
}

export function ProjectsFeature(props: ProjectsFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<ProjectsWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			{...props}
		/>
	)
}

function ProjectsWorkspace({
	organizationId,
	organizationSlug,
	filters,
	highlightedProjectId,
	onFiltersChange,
}: ProjectsFeatureProps & {
	organizationId: string
	organizationSlug: string
}) {
	const [instantiateDraft, setInstantiateDraft] =
		useState<InstantiateTemplateFormValues | null>(null)

	const projects = useProjects(organizationId, {
		includeArchived: filters.status !== 'actifs',
	})
	const does = useDoes(organizationId)
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
	const instantiateTemplate = useInstantiateProjectTemplate()
	const people = usePeople(organizationId)
	const extras = useProjectExtrasMap()
	const missions = useAllMissions()

	const customerRows = customers.data?.data ?? []
	const quoteRows = quotes.data?.data ?? []
	const templateRows = templates.data?.data ?? []

	const customerOptions = useMemo(
		() =>
			customerRows.map((customer) => ({
				id: customer.id,
				label: customer.name,
			})),
		[customerRows],
	)

	const rows = useMemo<ProjectRow[]>(() => {
		const nameByCustomer = new Map(
			customerRows.map((customer) => [customer.id, customer.name]),
		)
		const revisionByProject = new Map(
			(does.data?.data ?? []).map((doe) => [doe.project_id, doe.revision]),
		)
		return (projects.data?.data ?? []).map((project) => ({
			project,
			status: statusOf(project),
			customerName: project.customer_id
				? (nameByCustomer.get(project.customer_id) ?? 'Client')
				: null,
			extras: extras[project.id] ?? EMPTY_EXTRAS,
			missions: missionStats(
				missions.filter((mission) => mission.projectId === project.id),
			),
			doeRevision: revisionByProject.get(project.id) ?? null,
			owner: people.data.find(
				(person) => person.id === extras[project.id]?.ownerId,
			),
		}))
	}, [projects.data, customerRows, does.data, extras, missions, people.data])

	const active: ProjectFilters = {
		...filters,
		client: resolveClient(filters.client, customerRows),
	}
	const listed = filterRows(rows, active)

	const quotesForInstantiate = useMemo(() => {
		if (!instantiateDraft || instantiateDraft.customerId === '') return []

		return quoteRows
			.filter((quote) => quote.customer_id === instantiateDraft.customerId)
			.map((quote) => ({
				id: quote.id,
				label: `${quoteReferenceLabel(quote.reference)} — ${quote.title}`,
			}))
	}, [instantiateDraft, quoteRows])

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

	return (
		<>
			<ProjectsList
				organizationSlug={organizationSlug}
				summary={projectsSummary(rows)}
				tiles={projectTiles(rows)}
				filters={active}
				customers={customerOptions}
				listed={listed}
				empty={listEmpty(rows.length, active)}
				newProjectCustomerId={
					customerRows.some((customer) => customer.id === active.client)
						? active.client
						: undefined
				}
				highlightedProjectId={highlightedProjectId}
				isLoading={projects.isLoading}
				error={projects.error?.message ?? null}
				onFiltersChange={onFiltersChange}
				onStartFromTemplate={() =>
					setInstantiateDraft(emptyInstantiateTemplateForm(todayIsoDate()))
				}
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
				customers={customerOptions}
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
		</>
	)
}

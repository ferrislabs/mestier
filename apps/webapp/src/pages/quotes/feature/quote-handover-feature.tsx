import { Link, useNavigate } from '@tanstack/react-router'
import { AlertCircle, ArrowLeft, Loader2 } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { Button } from '#/components/ui/button'
import { PageShell, SectionCard } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { usePlanning } from '#/hooks/use-planning'
import {
	type Quote,
	type TaskProposal,
	useCreateQuotePlan,
	useQuotePlanProposal,
} from '#/hooks/use-quotes'
import { buildOrgPath } from '#/modules/org-path'
import { computeWindow } from '#/pages/planning/lib/window'
import { todayIsoDate } from '#/pages/planning/types'
import {
	buildPlannedTaskRequest,
	emptyHandoverTaskDraft,
	type HandoverTaskDraft,
	validateHandoverHierarchy,
} from '#/pages/quotes/lib/handover-task-form'
import { QuoteHandoverUI } from '#/pages/quotes/ui/quote-handover-ui'

export interface QuoteHandoverFeatureProps {
	quoteId: string
}

export function QuoteHandoverFeature({ quoteId }: QuoteHandoverFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const proposalQuery = useQuotePlanProposal(quoteId)
	const backTo = buildOrgPath(activeOrganization.slug, `/crm/quotes/${quoteId}`)

	if (proposalQuery.isLoading) {
		return (
			<PageShell>
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement du devis…
				</SectionCard>
			</PageShell>
		)
	}

	if (proposalQuery.isError || !proposalQuery.data?.data) {
		return (
			<PageShell>
				<SectionCard className="flex min-h-72 flex-col items-center justify-center gap-3 p-8 text-center">
					<AlertCircle className="size-6 text-destructive" />
					<div>
						<p className="font-semibold">Impossible de préparer la remise</p>
						<p className="mt-1 text-sm text-muted-foreground">
							{proposalQuery.error?.message ??
								'Seul un devis accepté peut devenir un projet.'}
						</p>
					</div>
					<Button asChild variant="outline">
						<Link to={backTo}>
							<ArrowLeft />
							Retour au devis
						</Link>
					</Button>
				</SectionCard>
			</PageShell>
		)
	}

	return (
		<QuoteHandoverWorkspace
			quoteId={quoteId}
			backTo={backTo}
			quote={proposalQuery.data.data.quote}
			proposal={proposalQuery.data.data.tasks}
		/>
	)
}

function QuoteHandoverWorkspace({
	quoteId,
	backTo,
	quote,
	proposal,
}: {
	quoteId: string
	backTo: string
	quote: Quote
	proposal: TaskProposal[]
}) {
	const { activeOrganization } = useActiveOrganization()
	const navigate = useNavigate()
	const createPlan = useCreateQuotePlan()

	const [projectName, setProjectName] = useState(quote.title)
	const [tasks, setTasks] = useState<HandoverTaskDraft[]>([])
	const seededTitle = useRef(false)
	useEffect(() => {
		if (seededTitle.current) return
		seededTitle.current = true
		setProjectName(quote.title)
	}, [quote.title])

	// Fetched solely for `timezone` — a task's start/end date and time
	// resolve against the organization's own zone, not the browser's.
	// Mirrors `projects-feature.tsx`'s own use of a range-independent `GET
	// /planning` call for the same reason.
	const planningQuery = usePlanning(
		activeOrganization.id,
		computeWindow('day', todayIsoDate()),
	)
	const timeZone = planningQuery.data?.data.timezone ?? 'UTC'

	const addTaskFromLine = (line: TaskProposal) => {
		setTasks((current) => [
			...current,
			emptyHandoverTaskDraft({
				today: todayIsoDate(),
				title: line.title,
				quoteLineIds: [line.quote_line_id],
				suggestedMinutes: line.suggested_minutes ?? null,
			}),
		])
	}

	const addBlankTask = () => {
		setTasks((current) => [
			...current,
			emptyHandoverTaskDraft({ today: todayIsoDate() }),
		])
	}

	const updateTask = (index: number, patch: Partial<HandoverTaskDraft>) => {
		setTasks((current) =>
			current.map((task, taskIndex) =>
				taskIndex === index ? { ...task, ...patch } : task,
			),
		)
	}

	const removeTask = (index: number) => {
		setTasks((current) => {
			const next = current
				.filter((_task, taskIndex) => taskIndex !== index)
				.map((task) => {
					if (task.parentIndex === null) return task
					if (task.parentIndex === index) return { ...task, parentIndex: null }
					return {
						...task,
						parentIndex:
							task.parentIndex > index
								? task.parentIndex - 1
								: task.parentIndex,
					}
				})
			return next
		})
	}

	const hierarchyErrors = validateHandoverHierarchy(tasks)
	const error = hierarchyErrors[0] ?? createPlan.error?.message ?? null

	const submit = async () => {
		if (hierarchyErrors.length > 0) return

		const plannedTasks = tasks.map((task) =>
			buildPlannedTaskRequest(task, timeZone),
		)
		if (plannedTasks.some((task) => task === null)) return

		const result = await createPlan.mutateAsync({
			path: { quote_id: quoteId },
			body: {
				name: projectName.trim(),
				force_new: false,
				tasks: plannedTasks as NonNullable<(typeof plannedTasks)[number]>[],
			},
		})

		await navigate({
			to: buildOrgPath(activeOrganization.slug, '/planning/projects'),
			search: { projectId: result.data.project.id, archived: false },
		})
	}

	return (
		<QuoteHandoverUI
			backTo={backTo}
			quote={quote}
			proposal={proposal}
			projectName={projectName}
			tasks={tasks}
			isPending={createPlan.isPending}
			error={error}
			onProjectNameChange={setProjectName}
			onAddTaskFromLine={addTaskFromLine}
			onAddBlankTask={addBlankTask}
			onTaskChange={updateTask}
			onRemoveTask={removeTask}
			onSubmit={() => {
				void submit()
			}}
		/>
	)
}

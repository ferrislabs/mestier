import { useState } from 'react'
import {
	type AssignmentReport,
	useAssignmentReports,
	useResolveAssignmentReport,
} from '#/hooks/use-assignment-reports'
import { useCustomerContexts, useCustomers } from '#/hooks/use-customers'
import type { PlanningResource } from '#/hooks/use-planning'
import { useProjects } from '#/hooks/use-projects'
import { useQuotes } from '#/hooks/use-quotes'
import {
	useCreateTaskComment,
	useDeleteTaskComment,
	useTaskComments,
	useUpdateTaskComment,
} from '#/hooks/use-task-comments'
import { useCreateTaskLabel, useTaskLabels } from '#/hooks/use-task-labels'
import {
	useCreateTaskRecurrence,
	useDeleteTaskRecurrence,
} from '#/hooks/use-task-recurrences'
import {
	useCreateTask,
	useDeleteTask,
	usePatchTask,
	useSubtasks,
	useTask,
} from '#/hooks/use-tasks'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	canLoadMoreComments,
	canLoadOlderComments,
	nextPageAfterCreate,
} from '#/pages/planning/lib/comments'
import { nextLabelColor } from '#/pages/planning/lib/labels'
import {
	appliedWindowFields,
	minutesLabel,
	plannedMinutes,
	reportsForTask,
} from '#/pages/planning/lib/pending-reports'
import {
	canAddSubtask,
	formatWindowPlaceholder,
	resolveDisplayWindow,
} from '#/pages/planning/lib/subtasks'
import {
	buildCreateTaskPayload,
	buildCreateTaskRecurrencePayload,
	buildFollowUpPatchPayload,
	buildPatchTaskPayload,
	emptyTaskDraft,
	needsFollowUpPatch,
	type TaskFormValues,
	taskToDraft,
	validateTaskDraft,
} from '#/pages/planning/lib/task-form'
import { todayIsoDate } from '#/pages/planning/types'
import { TaskSheet } from '#/pages/planning/ui/task-sheet'
import { formatCents, quoteReferenceLabel } from '#/pages/quotes/types'

export type TaskSheetTarget =
	| { mode: 'create'; parentTaskId: string | null }
	| { mode: 'edit'; taskId: string }

export interface TaskSheetFeatureProps {
	organizationId: string
	timeZone: string
	resources: PlanningResource[]
	open: boolean
	target: TaskSheetTarget
	onOpenChange: (open: boolean) => void
	/** Opens a different task in this same sheet — a subtask row, or "add subtask" from a root's own tab. */
	onNavigate: (target: TaskSheetTarget) => void
}

const COMMENTS_PER_PAGE = 20

/**
 * Owns every mutation the task sheet drives: create/patch/delete the task
 * itself, its labels, and its comment thread. Mount with a `key` derived
 * from `target` (see `feature/planning-team-feature.tsx`) so switching
 * which task is targeted — including navigating from a subtask row —
 * remounts this component with fresh local state, rather than trying to
 * reconcile a create draft into an edit draft in place.
 */
export function TaskSheetFeature({
	organizationId,
	timeZone,
	resources,
	open,
	target,
	onOpenChange,
	onNavigate,
}: TaskSheetFeatureProps) {
	const taskId = target.mode === 'edit' ? target.taskId : null
	const taskQuery = useTask(
		organizationId,
		taskId ?? '',
		open && taskId !== null,
	)
	const task = taskQuery.data?.data ?? null

	const parentTaskId =
		target.mode === 'create'
			? target.parentTaskId
			: (task?.parent_task_id ?? null)
	const isSubtask = parentTaskId !== null

	// A parent fetch is only needed to render the inherited-window
	// placeholder — when the current task (create or already-loaded edit
	// draft) has no dates of its own.
	const needsParentWindow =
		isSubtask &&
		(target.mode === 'create' || (task !== null && !task.starts_at))
	const parentQuery = useTask(
		organizationId,
		parentTaskId ?? '',
		open && needsParentWindow && parentTaskId !== null,
	)
	const parent = parentQuery.data?.data ?? null

	// -- Correction loop (assignment reports) ----------------------------------

	// Org-wide, then narrowed to this task's own assignments below
	// (`reportsForTask`): the backend has no "reports for this task" filter
	// (a report carries `task_assignment_id`, not `task_id`), and a
	// manager's pending queue is small enough that one page covers it — see
	// `use-assignment-reports.ts`'s own doc.
	const pendingReportsQuery = useAssignmentReports(
		organizationId,
		'PENDING',
		1,
		100,
		open && target.mode === 'edit',
	)
	const pendingReportsForTask = reportsForTask(
		pendingReportsQuery.data?.data ?? [],
		task?.assignments ?? [],
	)
	const resolveReport = useResolveAssignmentReport(organizationId)
	const [applyingReportId, setApplyingReportId] = useState<string | null>(null)
	const [dismissingReportId, setDismissingReportId] = useState<string | null>(
		null,
	)
	const [dismissNote, setDismissNote] = useState('')

	function memberName(memberId: string): string {
		return (
			resources.find((resource) => resource.member_id === memberId)
				?.display_name ?? 'Membre inconnu'
		)
	}

	function handleApplyReport(report: AssignmentReport) {
		if (!task) return
		const fields = appliedWindowFields(
			task.starts_at ?? null,
			report.reported_minutes,
			timeZone,
		)
		if (!fields) return
		setValues((current) => ({ ...current, ...fields }))
		setApplyingReportId(report.id)
		setDismissingReportId(null)
	}

	function handleStartDismissReport(report: AssignmentReport) {
		setDismissingReportId(report.id)
		setDismissNote('')
		setApplyingReportId(null)
	}

	async function handleConfirmDismissReport(report: AssignmentReport) {
		try {
			await resolveReport.mutateAsync({
				path: { assignment_report_id: report.id },
				body: {
					resolution: 'DISMISSED',
					resolution_note: dismissNote.trim() || null,
				},
			})
			setDismissingReportId(null)
			setDismissNote('')
		} catch {
			// Surfaced via `resolveReport.error` in the panel.
		}
	}

	const [values, setValues] = useState<TaskFormValues>(() =>
		target.mode === 'create'
			? emptyTaskDraft({
					parentTaskId: target.parentTaskId,
					today: todayIsoDate(),
				})
			: emptyTaskDraft({ parentTaskId: null, today: todayIsoDate() }),
	)
	const [didSeedEdit, setDidSeedEdit] = useState(false)
	if (target.mode === 'edit' && task && !didSeedEdit) {
		setValues(taskToDraft(task, timeZone))
		setDidSeedEdit(true)
	}

	const [saveError, setSaveError] = useState<string | null>(null)

	// A single unpaginated fetch of the organization's first 100 customers —
	// feeds both the create form's picker and, in edit mode, the client-side
	// name lookup below (`editCustomerName`). Beyond 100 customers this
	// silently misses some: the create picker just won't list them, and an
	// edit-mode task whose client falls past the cut shows "Aucun client"
	// even though `task.customer_id` is set. No pagination or search exists
	// on this fetch today — worth knowing about before this ships to an
	// organization with a large customer base.
	const customersQuery = useCustomers(organizationId, { page: 1, perPage: 100 })
	const customerContextsQuery = useCustomerContexts(
		values.customerId,
		Boolean(values.customerId),
	)

	// The organization's quotes, narrowed to the chosen customer's accepted ones
	// below. No per-customer endpoint exists, so this reads the same page the
	// quote list does and filters client-side; fine at this size, and it shares
	// the cache with the quotes screen rather than adding a request.
	const quotesQuery = useQuotes(organizationId, { page: 1, perPage: 100 })
	// Live projects only: attaching a task to an archived one would resurrect a
	// subject somebody deliberately retired.
	const projectsQuery = useProjects(organizationId, { includeArchived: false })

	const labelsQuery = useTaskLabels(organizationId)
	const createLabel = useCreateTaskLabel(organizationId)
	const labels = labelsQuery.data?.data ?? []

	async function handleCreateLabel(name: string) {
		try {
			const created = await createLabel.mutateAsync({
				path: { organization_id: organizationId },
				body: { name, color: nextLabelColor(labels) },
			})
			// Auto-selects the label just created — the user typed a name
			// intending to apply it, not merely to enrich the catalog.
			setValues((current) => ({
				...current,
				labelIds: [...current.labelIds, created.data.id],
			}))
		} catch {
			// Surfaced via createLabel.error inside the picker's own render.
		}
	}

	const createTask = useCreateTask(organizationId)
	const patchTask = usePatchTask()
	const deleteTask = useDeleteTask()
	const createTaskRecurrence = useCreateTaskRecurrence(organizationId)
	const deleteTaskRecurrence = useDeleteTaskRecurrence()

	const windowPlaceholder = formatWindowPlaceholder(
		parent
			? {
					startsAt: parent.starts_at as string,
					endsAt: parent.ends_at as string,
				}
			: null,
		timeZone,
	)

	async function handleSubmit() {
		setSaveError(null)

		if (target.mode === 'create' && values.recurrence.enabled) {
			const recurrencePayload = buildCreateTaskRecurrencePayload(values, {
				timeZone,
			})
			if (!recurrencePayload) return

			try {
				// One request: `CreateTaskRecurrenceCommand` carries the
				// template's assignees directly, unlike a bare task's `POST`
				// — see `buildCreateTaskRecurrencePayload`'s own doc for why
				// there is no follow-up `PATCH` here.
				await createTaskRecurrence.mutateAsync({
					path: { organization_id: organizationId },
					body: recurrencePayload,
				})
				onOpenChange(false)
			} catch (error) {
				setSaveError(errorMessage(error))
			}
			return
		}

		if (target.mode === 'create') {
			const createPayload = buildCreateTaskPayload(values, {
				parentTaskId: target.parentTaskId,
				timeZone,
			})
			if (!createPayload) return

			try {
				// Two network calls for one user action: `POST /tasks` never
				// accepts `assignees`/`label_ids` (a task is always created
				// bare — see `lib/task-form.ts`'s `buildCreateTaskPayload`
				// doc), so a create that also picked assignees or labels on
				// the form has to follow up with an immediate `PATCH` using
				// the id the `POST` just returned. Invisible to the user —
				// one "Créer" click, one save state — but genuinely two
				// requests; a failure in the second leaves the task created
				// but unassigned/unlabeled rather than rolling back the
				// first (there is no cross-request transaction to roll back
				// into), surfaced the same as any other save error.
				const created = await createTask.mutateAsync({
					path: { organization_id: organizationId },
					body: createPayload,
				})

				if (
					needsFollowUpPatch({
						assignees: values.assignees,
						labelIds: values.labelIds,
					})
				) {
					await patchTask.mutateAsync({
						path: { organization_id: organizationId, task_id: created.data.id },
						body: buildFollowUpPatchPayload({
							assignees: values.assignees,
							labelIds: values.labelIds,
						}),
					})
				}

				onOpenChange(false)
			} catch (error) {
				setSaveError(errorMessage(error))
			}
			return
		}

		if (!taskId) return
		const patchPayload = buildPatchTaskPayload(values, { isSubtask, timeZone })
		if (!patchPayload) return

		try {
			await patchTask.mutateAsync({
				path: { organization_id: organizationId, task_id: taskId },
				body: patchPayload,
			})
		} catch (error) {
			setSaveError(errorMessage(error))
			return
		}

		// Two calls, in that order: the report is only marked applied once
		// the task edit above actually succeeded — see the issue's own
		// warning against marking a report applied against a task that
		// never moved. A failure here leaves the sheet open rather than
		// closing over an error the manager would otherwise never see: the
		// task already moved, and the report can be resolved again from
		// this same panel.
		if (applyingReportId) {
			try {
				await resolveReport.mutateAsync({
					path: { assignment_report_id: applyingReportId },
					body: { resolution: 'APPLIED', resolution_note: null },
				})
			} catch {
				return
			}
		}

		onOpenChange(false)
	}

	async function handleDelete() {
		if (!taskId) return
		try {
			await deleteTask.mutateAsync({
				path: { organization_id: organizationId, task_id: taskId },
			})
			onOpenChange(false)
		} catch (error) {
			setSaveError(errorMessage(error))
		}
	}

	/**
	 * The three scopes a `DELETE` on a recurring occurrence can take — see
	 * `ui/task-sheet.tsx`'s `deleteSeriesOptions` doc. `thisAndFollowing`
	 * hits the same `DELETE /tasks/{id}` endpoint as a plain delete, only
	 * with `?scope=` added; the whole-series choice hits
	 * `DELETE /task-recurrences/{id}` instead, a different aggregate
	 * entirely.
	 */
	async function handleDeleteOccurrence(
		scope: 'THIS_OCCURRENCE' | 'THIS_AND_FOLLOWING',
	) {
		if (!taskId) return
		try {
			await deleteTask.mutateAsync({
				path: { organization_id: organizationId, task_id: taskId },
				query: { scope },
			})
			onOpenChange(false)
		} catch (error) {
			setSaveError(errorMessage(error))
		}
	}

	async function handleDeleteWholeSeries() {
		if (!task?.recurrence_id) return
		try {
			await deleteTaskRecurrence.mutateAsync({
				path: { task_recurrence_id: task.recurrence_id },
			})
			onOpenChange(false)
		} catch (error) {
			setSaveError(errorMessage(error))
		}
	}

	// -- Subtasks -------------------------------------------------------------

	const subtasksQuery = useSubtasks(
		organizationId,
		taskId ?? '',
		open && target.mode === 'edit' && Boolean(taskId),
	)
	const subtaskItems = (subtasksQuery.data?.data ?? []).map((subtask) => {
		const resolved = task
			? resolveDisplayWindow(
					{
						startsAt: subtask.starts_at ?? null,
						endsAt: subtask.ends_at ?? null,
					},
					task.starts_at && task.ends_at
						? { startsAt: task.starts_at, endsAt: task.ends_at }
						: null,
				)
			: null
		return {
			id: subtask.id,
			title: subtask.title,
			status: subtask.status,
			assigneeCount: subtask.member_ids.length,
			inheritedWindow: resolved?.inherited ?? false,
		}
	})

	// -- Fil de commentaires ----------------------------------------------------

	const [commentsPage, setCommentsPage] = useState(1)
	const commentsQuery = useTaskComments(
		organizationId,
		taskId ?? '',
		commentsPage,
		COMMENTS_PER_PAGE,
		open && target.mode === 'edit' && Boolean(taskId),
	)
	const comments = commentsQuery.data?.data ?? []
	const commentsPagination = commentsQuery.data?.pagination ?? null

	const [draftBody, setDraftBody] = useState('')
	const createComment = useCreateTaskComment(organizationId, taskId ?? '')

	const [editingCommentId, setEditingCommentId] = useState<string | null>(null)
	const [editingBody, setEditingBody] = useState('')
	const updateComment = useUpdateTaskComment(organizationId, taskId ?? '')
	const deleteComment = useDeleteTaskComment(organizationId, taskId ?? '')
	const [deletingCommentId, setDeletingCommentId] = useState<string | null>(
		null,
	)

	async function handleSubmitComment() {
		if (!taskId || !draftBody.trim()) return
		try {
			await createComment.mutateAsync({
				path: { organization_id: organizationId, task_id: taskId },
				body: { body: draftBody.trim() },
			})
			setDraftBody('')
			setCommentsPage(
				nextPageAfterCreate(commentsPagination, COMMENTS_PER_PAGE),
			)
		} catch {
			// Surfaced via createComment.error — the draft is kept so nothing is lost.
		}
	}

	async function handleConfirmEditComment() {
		if (!taskId || !editingCommentId || !editingBody.trim()) return
		try {
			await updateComment.mutateAsync({
				path: {
					organization_id: organizationId,
					task_id: taskId,
					comment_id: editingCommentId,
				},
				body: { body: editingBody.trim() },
			})
			setEditingCommentId(null)
		} catch {
			// Surfaced via updateComment.error.
		}
	}

	async function handleDeleteComment(comment: { id: string }) {
		if (!taskId) return
		setDeletingCommentId(comment.id)
		try {
			await deleteComment.mutateAsync({
				path: {
					organization_id: organizationId,
					task_id: taskId,
					comment_id: comment.id,
				},
			})
		} finally {
			setDeletingCommentId(null)
		}
	}

	const errors = validateTaskDraft(values, { isSubtask })
	const assigneeOptions = resources.map((resource) => ({
		resourceId: resource.resource_id,
		displayName: resource.display_name,
	}))
	const customerContexts = (customerContextsQuery.data?.data ?? []).map(
		(context) => ({
			id: context.id,
			label: context.label,
		}),
	)
	// Only the accepted quotes of the chosen customer. A projet bills against
	// something the customer agreed to, and offering a draft would invite a
	// margin computed from a number nobody signed.
	const quotes = (quotesQuery.data?.data ?? [])
		.filter(
			(quote) =>
				quote.customer_id === values.customerId && quote.status === 'ACCEPTED',
		)
		.map((quote) => ({
			id: quote.id,
			label: `${quoteReferenceLabel(quote.reference)} · ${formatCents(quote.gross_cents)}`,
		}))
	// Internal projects are offered like any other — a meeting has to be
	// attachable, that is the point of them existing.
	const projects = (projectsQuery.data?.data ?? []).map((project) => ({
		id: project.id,
		label: project.name,
		isInternal: project.is_internal,
	}))
	const customerOptions = (customersQuery.data?.data ?? []).map((customer) => ({
		id: customer.id,
		displayName: customer.name.trim(),
	}))
	// Edit mode's static customer display: `TaskResponse` carries only
	// `customer_id`, no name — resolved against the same customer list the
	// create form's own selector uses, already loaded, rather than a second
	// request just to render one line of text.
	const editCustomerName = task?.customer_id
		? (customerOptions.find((customer) => customer.id === task.customer_id)
				?.displayName ?? null)
		: null
	const taskPlannedMinutes = task
		? plannedMinutes(task.starts_at ?? null, task.ends_at ?? null)
		: null
	const plannedLabel =
		taskPlannedMinutes === null
			? 'Durée non planifiée'
			: minutesLabel(taskPlannedMinutes)
	const isPartOfSeries = Boolean(task?.recurrence_id)

	return (
		<TaskSheet
			open={open}
			mode={target.mode}
			title={sheetTitle(target.mode, isSubtask)}
			isSaving={
				createTask.isPending ||
				patchTask.isPending ||
				createTaskRecurrence.isPending
			}
			isDeleting={deleteTask.isPending || deleteTaskRecurrence.isPending}
			saveError={saveError}
			onSubmit={() => void handleSubmit()}
			onDelete={
				target.mode === 'edit' && !isPartOfSeries
					? () => void handleDelete()
					: undefined
			}
			deleteSeriesOptions={
				target.mode === 'edit' && isPartOfSeries
					? {
							onThisOccurrence: () =>
								void handleDeleteOccurrence('THIS_OCCURRENCE'),
							onThisAndFollowing: () =>
								void handleDeleteOccurrence('THIS_AND_FOLLOWING'),
							onWholeSeries: () => void handleDeleteWholeSeries(),
						}
					: undefined
			}
			onOpenChange={onOpenChange}
			fields={{
				mode: target.mode,
				isSubtask,
				values,
				onChange: (patch) => setValues((current) => ({ ...current, ...patch })),
				errors,
				windowPlaceholder,
				isPartOfSeries,
				customerName: editCustomerName,
				customers: customerOptions,
				customerContexts,
				quotes,
				isQuotesLoading: quotesQuery.isLoading,
				projects,
				isProjectsLoading: projectsQuery.isLoading,
				isCustomerContextsLoading: customerContextsQuery.isLoading,
				labels: labels.map((label) => ({
					id: label.id,
					name: label.name,
					color: label.color,
				})),
				isCreatingLabel: createLabel.isPending,
				onCreateLabel: (name) => void handleCreateLabel(name),
				assigneeOptions,
			}}
			subtasksTab={
				target.mode === 'edit'
					? {
							subtasks: subtaskItems,
							isLoading: subtasksQuery.isLoading,
							error: subtasksQuery.error?.message ?? null,
							canAddSubtask: task
								? canAddSubtask({ parentTaskId: task.parent_task_id ?? null })
								: false,
							onAddSubtask: () =>
								taskId && onNavigate({ mode: 'create', parentTaskId: taskId }),
							onOpenSubtask: (id) => onNavigate({ mode: 'edit', taskId: id }),
						}
					: undefined
			}
			commentsTab={
				target.mode === 'edit'
					? {
							comments,
							isLoading: commentsQuery.isLoading,
							error: commentsQuery.error?.message ?? null,
							canLoadMore: canLoadMoreComments(commentsPagination),
							canLoadOlder: canLoadOlderComments(commentsPagination),
							draftBody,
							onDraftChange: setDraftBody,
							onSubmit: () => void handleSubmitComment(),
							isSubmitting: createComment.isPending,
							editingCommentId,
							editingBody,
							onStartEdit: (comment) => {
								setEditingCommentId(comment.id)
								setEditingBody(comment.body)
							},
							onEditBodyChange: setEditingBody,
							onConfirmEdit: () => void handleConfirmEditComment(),
							onCancelEdit: () => setEditingCommentId(null),
							onDelete: (comment) => void handleDeleteComment(comment),
							deletingCommentId,
							onLoadOlder: () =>
								setCommentsPage((page) => Math.max(1, page - 1)),
							onLoadMore: () => setCommentsPage((page) => page + 1),
						}
					: undefined
			}
			pendingReportPanel={
				target.mode === 'edit' && pendingReportsForTask.length > 0
					? {
							reports: pendingReportsForTask,
							memberName,
							plannedLabel,
							reportedLabel: minutesLabel,
							applyingReportId,
							onApply: handleApplyReport,
							onCancelApply: () => setApplyingReportId(null),
							isResolving: resolveReport.isPending,
							resolveError: resolveReport.error?.message ?? null,
							dismissingReportId,
							dismissNote,
							onStartDismiss: handleStartDismissReport,
							onCancelDismiss: () => setDismissingReportId(null),
							onDismissNoteChange: setDismissNote,
							onConfirmDismiss: (report) =>
								void handleConfirmDismissReport(report),
						}
					: undefined
			}
		/>
	)
}

function sheetTitle(mode: 'create' | 'edit', isSubtask: boolean): string {
	if (mode === 'edit') return 'Modifier la tâche'
	return isSubtask ? 'Nouvelle sous-tâche' : 'Nouvelle tâche'
}

function errorMessage(error: unknown): string {
	return mutationErrorMessage(error) ?? "L'enregistrement a échoué. Réessayez."
}

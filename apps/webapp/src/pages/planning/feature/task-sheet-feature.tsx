import { useState } from 'react'
import { useAuth } from 'react-oidc-context'
import { useCustomerContexts, useCustomers } from '#/hooks/use-customers'
import type { PlanningResource } from '#/hooks/use-planning'
import {
	useCreateTaskComment,
	useDeleteTaskComment,
	useTaskComments,
	useUpdateTaskComment,
} from '#/hooks/use-task-comments'
import { useCreateTaskLabel, useTaskLabels } from '#/hooks/use-task-labels'
import {
	useCreateTask,
	useDeleteTask,
	usePatchTask,
	useSubtasks,
	useTask,
} from '#/hooks/use-tasks'
import {
	canLoadMoreComments,
	canLoadOlderComments,
	nextPageAfterCreate,
} from '#/pages/planning/lib/comments'
import { nextLabelColor } from '#/pages/planning/lib/labels'
import {
	canAddSubtask,
	formatWindowPlaceholder,
	resolveDisplayWindow,
} from '#/pages/planning/lib/subtasks'
import {
	buildCreateTaskPayload,
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
	const auth = useAuth()
	const profile = auth.user?.profile
	// Mirrors `_app.tsx`'s own fallback chain for the signed-in user's
	// display name — see `lib/comments.ts`'s `isOwnComment` doc for why this
	// heuristic exists at all (no endpoint hands back the caller's internal
	// `UserId` to compare against `TaskCommentAuthorResponse.id` directly).
	const currentDisplayName: string | null =
		profile?.name || profile?.preferred_username || profile?.email || null

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

	const customersQuery = useCustomers(organizationId, { page: 1, perPage: 100 })
	const customerContextsQuery = useCustomerContexts(
		values.customerId,
		Boolean(values.customerId),
	)

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

		if (target.mode === 'create') {
			const createPayload = buildCreateTaskPayload(values, {
				parentTaskId: target.parentTaskId,
				timeZone,
			})
			if (!createPayload) return

			try {
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
			onOpenChange(false)
		} catch (error) {
			setSaveError(errorMessage(error))
		}
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

	// -- Sous-tâches ----------------------------------------------------------

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
			assigneeCount: subtask.employee_ids.length,
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
	const customerOptions = (customersQuery.data?.data ?? []).map((customer) => ({
		id: customer.id,
		displayName: `${customer.last_name} ${customer.first_name}`.trim(),
	}))
	// Edit mode's static customer display: `TaskResponse` carries only
	// `customer_id`, no name — resolved against the same customer list the
	// create form's own selector uses, already loaded, rather than a second
	// request just to render one line of text.
	const editCustomerName = task?.customer_id
		? (customerOptions.find((customer) => customer.id === task.customer_id)
				?.displayName ?? null)
		: null

	return (
		<TaskSheet
			open={open}
			mode={target.mode}
			title={sheetTitle(target.mode, isSubtask)}
			isSaving={createTask.isPending || patchTask.isPending}
			isDeleting={deleteTask.isPending}
			canDelete={target.mode === 'edit' && (task?.child_count ?? 0) === 0}
			saveError={saveError}
			onSubmit={() => void handleSubmit()}
			onDelete={target.mode === 'edit' ? () => void handleDelete() : undefined}
			onOpenChange={onOpenChange}
			fields={{
				mode: target.mode,
				isSubtask,
				values,
				onChange: (patch) => setValues((current) => ({ ...current, ...patch })),
				errors,
				windowPlaceholder,
				customerName: editCustomerName,
				customers: customerOptions,
				customerContexts,
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
							currentDisplayName,
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
		/>
	)
}

function sheetTitle(mode: 'create' | 'edit', isSubtask: boolean): string {
	if (mode === 'edit') return 'Modifier la tâche'
	return isSubtask ? 'Nouvelle sous-tâche' : 'Nouvelle tâche'
}

function errorMessage(error: unknown): string {
	if (error instanceof Error) return error.message
	return "L'enregistrement a échoué. Réessayez."
}

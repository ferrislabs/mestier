import { useState } from 'react'
import { useHasPermission } from '#/hooks/use-permissions'
import {
	useAddSupportEntry,
	useAssignSupportTicket,
	useDeleteSupportDraft,
	useEditSupportDraft,
	useOpenSupportAttachment,
	useSendSupportDraft,
	useSupportThread,
	useTransitionSupportTicket,
} from '#/hooks/use-support'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	clearDraftReviews,
	commentDraft,
	toggleDraftApproval,
	useDraftReviews,
} from '#/pages/support/feature/mocks/draft-reviews'
import { setTicketPriority } from '#/pages/support/feature/mocks/priorities'
import { usePropertiesPanel } from '#/pages/support/feature/use-properties-panel'
import {
	useAddSupportTicketBadge,
	useRemoveSupportTicketBadge,
} from '#/pages/support/feature/use-support-badges'
import { useSupportTicketVM } from '#/pages/support/feature/use-support-ticket'
import { SUPPORT_COLUMNS } from '#/pages/support/lib/columns'
import { memberHue } from '#/pages/support/lib/format'
import {
	canSubmitBody,
	sortEntriesByCreation,
	toEntryVM,
} from '#/pages/support/lib/thread'
import { UNKNOWN_MEMBER_LABEL } from '#/pages/support/lib/view-models'
import type {
	SupportComposerMode,
	SupportDetailVariant,
	SupportDraftReviewsVM,
	SupportStatus,
	SupportStatusOption,
	SupportTicketVM,
} from '#/pages/support/types'
import { TicketDetailUI } from '#/pages/support/ui/ticket-detail-ui'

const ATTACHMENT_ERROR_MESSAGE = 'Impossible d’ouvrir la pièce jointe.'
const DEFAULT_NEXT_STATUS: SupportStatus = 'waiting_customer'

const STATUS_OPTIONS: SupportStatusOption[] = SUPPORT_COLUMNS.map(
	({ status, label }) => ({ status, label }),
)

interface MutationProbe {
	isPending: boolean
	isError: boolean
	error: unknown
	submittedAt: number
	variables: unknown
}

function pendingEntryIdOf(mutation: MutationProbe): string | null {
	if (!mutation.isPending) return null

	const variables = mutation.variables as
		| { path?: { entry_id?: string } }
		| undefined
	return variables?.path?.entry_id ?? null
}

function latestFailureMessage(mutations: MutationProbe[]): string | null {
	let latest: MutationProbe | null = null

	for (const mutation of mutations) {
		if (!mutation.isError) continue
		if (latest === null || mutation.submittedAt > latest.submittedAt) {
			latest = mutation
		}
	}

	return mutationErrorMessage(latest?.error)
}

export interface SupportTicketDetailFeatureProps {
	organizationId: string
	ticketId: string
	variant: SupportDetailVariant
	siblings?: SupportTicketVM[]
	onOpenTicket?: (ticketId: string) => void
	onOpenFullPage?: (ticketId: string) => void
	onClose?: () => void
}

export function SupportTicketDetailFeature({
	organizationId,
	ticketId,
	variant,
	siblings,
	onOpenTicket,
	onOpenFullPage,
	onClose,
}: SupportTicketDetailFeatureProps) {
	const canManage = useHasPermission('MANAGE_SUPPORT')
	const {
		query: ticketQuery,
		vm: ticket,
		members,
	} = useSupportTicketVM(organizationId, ticketId)
	const threadQuery = useSupportThread(organizationId, ticketId)
	const addEntry = useAddSupportEntry(organizationId, ticketId)
	const editDraft = useEditSupportDraft(organizationId, ticketId)
	const sendDraft = useSendSupportDraft(organizationId, ticketId)
	const deleteDraft = useDeleteSupportDraft(organizationId, ticketId)
	const transition = useTransitionSupportTicket(organizationId, ticketId)
	const assign = useAssignSupportTicket(organizationId, ticketId)
	const addBadge = useAddSupportTicketBadge(organizationId)
	const removeBadge = useRemoveSupportTicketBadge(organizationId)
	const openAttachment = useOpenSupportAttachment(organizationId)
	const reviews = useDraftReviews()
	const [propertiesOpen, toggleProperties] = usePropertiesPanel()
	const [mode, setMode] = useState<SupportComposerMode>('reply')
	const [text, setText] = useState('')
	const [nextStatus, setNextStatus] =
		useState<SupportStatus>(DEFAULT_NEXT_STATUS)
	const [editingDraftId, setEditingDraftId] = useState<string | null>(null)
	const [attachmentError, setAttachmentError] = useState<string | null>(null)

	const reviewerId = members.members[0]?.id ?? ''
	const path = { organization_id: organizationId, ticket_id: ticketId }

	function resetComposer() {
		setText('')
		setEditingDraftId(null)
		setMode('reply')
	}

	function applyNextStatus(status: SupportStatus) {
		if (!ticket || status === ticket.status) return
		transition.mutate({ path, body: { status } })
	}

	function finishSend(status: SupportStatus) {
		resetComposer()
		applyNextStatus(status)
	}

	function sendExistingDraft(entryId: string, status: SupportStatus) {
		sendDraft.mutate(
			{ path: { organization_id: organizationId, entry_id: entryId } },
			{
				onSuccess: () => {
					clearDraftReviews(entryId)
					finishSend(status)
				},
			},
		)
	}

	function handleSend() {
		if (!canSubmitBody(text)) return

		if (editingDraftId) {
			const entryId = editingDraftId
			editDraft.mutate(
				{
					path: { organization_id: organizationId, entry_id: entryId },
					body: { body: text },
				},
				{ onSuccess: () => sendExistingDraft(entryId, nextStatus) },
			)
			return
		}

		addEntry.mutate(
			{ path, body: { kind: 'reply', body: text } },
			{ onSuccess: () => finishSend(nextStatus) },
		)
	}

	function handleSaveDraft() {
		if (!canSubmitBody(text)) return

		if (editingDraftId) {
			const entryId = editingDraftId
			editDraft.mutate(
				{
					path: { organization_id: organizationId, entry_id: entryId },
					body: { body: text },
				},
				{
					onSuccess: () => {
						clearDraftReviews(entryId)
						resetComposer()
					},
				},
			)
			return
		}

		addEntry.mutate(
			{ path, body: { kind: 'draft', body: text } },
			{ onSuccess: resetComposer },
		)
	}

	function handleAddNote() {
		if (!canSubmitBody(text)) return

		addEntry.mutate(
			{ path, body: { kind: 'note', body: text } },
			{ onSuccess: resetComposer },
		)
	}

	function handleDeleteDraft(entryId: string) {
		deleteDraft.mutate(
			{ path: { organization_id: organizationId, entry_id: entryId } },
			{
				onSuccess: () => {
					clearDraftReviews(entryId)
					if (editingDraftId === entryId) resetComposer()
				},
			},
		)
	}

	async function handleOpenAttachment(attachmentId: string) {
		try {
			await openAttachment(attachmentId)
			setAttachmentError(null)
		} catch {
			setAttachmentError(ATTACHMENT_ERROR_MESSAGE)
		}
	}

	const requesterName = ticket?.requester.name ?? ''
	const entries = sortEntriesByCreation(threadQuery.data?.data ?? []).map(
		(entry) => toEntryVM(entry, members.namesById, requesterName),
	)

	const reviewsByEntry = new Map<string, SupportDraftReviewsVM>()
	for (const entry of entries) {
		if (entry.kind !== 'draft') continue
		const stored = reviews.data[entry.id] ?? []
		const approvals = stored.filter((review) => review.kind === 'approval')
		reviewsByEntry.set(entry.id, {
			approvedBy: approvals.map(
				(review) => members.namesById[review.memberId] ?? UNKNOWN_MEMBER_LABEL,
			),
			approvedByMe: approvals.some((review) => review.memberId === reviewerId),
			comments: stored
				.filter((review) => review.kind === 'comment')
				.map((review) => ({
					id: review.id,
					memberName:
						members.namesById[review.memberId] ?? UNKNOWN_MEMBER_LABEL,
					memberHue: memberHue(review.memberId),
					kind: 'comment',
					body: review.body,
				})),
		})
	}

	const pendingEntryId =
		[editDraft, sendDraft, deleteDraft]
			.map(pendingEntryIdOf)
			.find((entryId) => entryId !== null) ?? null
	const threadError =
		threadQuery.error?.message ??
		attachmentError ??
		latestFailureMessage([editDraft, sendDraft, deleteDraft])

	const otherTickets = (siblings ?? []).filter(
		(other) =>
			ticket !== null &&
			other.id !== ticket.id &&
			other.requester.email !== null &&
			other.requester.email === ticket.requester.email,
	)
	const knownBadges = [
		...new Set(
			(siblings ?? []).flatMap((other) => other.badges.map((b) => b.label)),
		),
	]

	const conversation = ticket
		? {
				entries,
				isLoading: threadQuery.isLoading,
				error: threadError,
				onRetry: () => void threadQuery.refetch(),
				canManage,
				mode,
				onModeChange: setMode,
				text,
				onTextChange: setText,
				nextStatus,
				onNextStatusChange: setNextStatus,
				editingDraftId,
				pendingEntryId,
				isSubmitting:
					addEntry.isPending || editDraft.isPending || sendDraft.isPending,
				composerError: mutationErrorMessage(addEntry.error),
				reviewsByEntry,
				onEditDraft: (entryId: string) => {
					const draft = entries.find((entry) => entry.id === entryId)
					if (!draft) return
					setEditingDraftId(entryId)
					setMode('reply')
					setText(draft.body)
				},
				onCancelDraft: resetComposer,
				onSendDraft: (entryId: string) =>
					sendExistingDraft(entryId, DEFAULT_NEXT_STATUS),
				onDeleteDraft: handleDeleteDraft,
				onToggleApproval: (entryId: string) =>
					void toggleDraftApproval(entryId, reviewerId),
				onCommentDraft: (entryId: string, body: string) =>
					void commentDraft(entryId, reviewerId, body),
				onSend: handleSend,
				onSaveDraft: handleSaveDraft,
				onAddNote: handleAddNote,
				onOpenAttachment: (attachmentId: string) =>
					void handleOpenAttachment(attachmentId),
			}
		: null

	const properties = ticket
		? {
				ticket,
				canManage,
				statusOptions: STATUS_OPTIONS,
				members: members.members,
				knownBadges,
				otherTickets,
				error: latestFailureMessage([
					transition,
					assign,
					addBadge,
					removeBadge,
				]),
				onOpenOtherTicket: (otherId: string) => onOpenTicket?.(otherId),
				onStatusChange: (status: SupportStatus) =>
					transition.mutate({ path, body: { status } }),
				onPriorityChange: (priority: Parameters<typeof setTicketPriority>[1]) =>
					void setTicketPriority(ticketId, priority),
				onAssigneeChange: (memberId: string | null) =>
					assign.mutate({ path, body: { member_id: memberId } }),
				onAddBadge: (label: string) =>
					addBadge.mutate({ path, body: { label } }),
				onRemoveBadge: (label: string) =>
					removeBadge.mutate({ path: { ...path, label } }),
			}
		: null

	return (
		<TicketDetailUI
			variant={variant}
			ticket={ticket}
			isLoading={ticketQuery.isLoading}
			error={ticket ? null : (ticketQuery.error?.message ?? null)}
			onRetry={() => void ticketQuery.refetch()}
			conversation={conversation}
			properties={properties}
			propertiesOpen={propertiesOpen}
			onToggleProperties={toggleProperties}
			onOpenFullPage={() => onOpenFullPage?.(ticketId)}
			onClose={onClose}
		/>
	)
}

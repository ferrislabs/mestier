import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import {
	useAddSupportEntry,
	useAssignSupportTicket,
	useDeleteSupportDraft,
	useEditSupportDraft,
	useOpenSupportAttachment,
	useSendSupportDraft,
	useSupportThread,
	useSupportTicket,
	useTransitionSupportTicket,
} from '#/hooks/use-support'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	SupportOrganizationUnavailable,
	useSupportMembers,
} from '#/pages/support/feature/use-support-screen'
import { SUPPORT_COLUMNS } from '#/pages/support/lib/columns'
import {
	canSubmitBody,
	SUPPORT_COMPOSER_TABS,
	sortEntriesByCreation,
	toEntryVM,
	toTicketDetailVM,
} from '#/pages/support/lib/thread'
import { assigneeLabel } from '#/pages/support/lib/view-models'
import type {
	SupportComposerTab,
	SupportStatus,
	SupportStatusOption,
	SupportTicketScreenProps,
} from '#/pages/support/types'
import { SupportComposerUI } from '#/pages/support/ui/support-composer-ui'
import { SupportSidePanelUI } from '#/pages/support/ui/support-side-panel-ui'
import { SupportThreadUI } from '#/pages/support/ui/support-thread-ui'
import { SupportTicketPageUI } from '#/pages/support/ui/support-ticket-page-ui'

const ATTACHMENT_ERROR_MESSAGE = 'Impossible d’ouvrir la pièce jointe.'

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

export function SupportTicketFeature({
	ticketId,
	onBack,
}: SupportTicketScreenProps) {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) return <SupportOrganizationUnavailable />

	return (
		<SupportTicketContent
			key={`${activeOrganization.id}:${ticketId}`}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			ticketId={ticketId}
			onBack={onBack}
		/>
	)
}

interface SupportTicketContentProps extends SupportTicketScreenProps {
	organizationId: string
	organizationName: string
}

function SupportTicketContent({
	organizationId,
	organizationName,
	ticketId,
	onBack,
}: SupportTicketContentProps) {
	const canManage = useHasPermission('MANAGE_SUPPORT')
	const members = useSupportMembers(organizationId)
	const ticketQuery = useSupportTicket(organizationId, ticketId)
	const threadQuery = useSupportThread(organizationId, ticketId)
	const addEntry = useAddSupportEntry(organizationId, ticketId)
	const editDraft = useEditSupportDraft(organizationId, ticketId)
	const sendDraft = useSendSupportDraft(organizationId, ticketId)
	const deleteDraft = useDeleteSupportDraft(organizationId, ticketId)
	const transition = useTransitionSupportTicket(organizationId, ticketId)
	const assign = useAssignSupportTicket(organizationId, ticketId)
	const openAttachment = useOpenSupportAttachment(organizationId)
	const [activeTab, setActiveTab] = useState<SupportComposerTab>('reply')
	const [body, setBody] = useState('')
	const [editingEntryId, setEditingEntryId] = useState<string | null>(null)
	const [attachmentError, setAttachmentError] = useState<string | null>(null)

	function handleSubmit() {
		if (!canSubmitBody(body)) return

		addEntry.mutate(
			{
				path: { organization_id: organizationId, ticket_id: ticketId },
				body: { kind: activeTab, body },
			},
			{ onSuccess: () => setBody('') },
		)
	}

	function handleSaveEdit(entryId: string, nextBody: string) {
		if (!canSubmitBody(nextBody)) return

		editDraft.mutate(
			{
				path: { organization_id: organizationId, entry_id: entryId },
				body: { body: nextBody },
			},
			{ onSuccess: () => setEditingEntryId(null) },
		)
	}

	function handleSendDraft(entryId: string) {
		sendDraft.mutate({
			path: { organization_id: organizationId, entry_id: entryId },
		})
	}

	function handleDeleteDraft(entryId: string) {
		deleteDraft.mutate({
			path: { organization_id: organizationId, entry_id: entryId },
		})
	}

	function handleChangeStatus(status: SupportStatus) {
		transition.mutate({
			path: { organization_id: organizationId, ticket_id: ticketId },
			body: { status },
		})
	}

	function handleChangeAssignee(memberId: string | null) {
		assign.mutate({
			path: { organization_id: organizationId, ticket_id: ticketId },
			body: { member_id: memberId },
		})
	}

	async function handleOpenAttachment(attachmentId: string) {
		try {
			await openAttachment(attachmentId)
			setAttachmentError(null)
		} catch {
			setAttachmentError(ATTACHMENT_ERROR_MESSAGE)
		}
	}

	const ticket = ticketQuery.data?.data
	const detail = ticket ? toTicketDetailVM(ticket) : null
	const entries = sortEntriesByCreation(threadQuery.data?.data ?? []).map(
		(entry) => toEntryVM(entry, members.namesById),
	)
	const pendingEntryId =
		[editDraft, sendDraft, deleteDraft]
			.map(pendingEntryIdOf)
			.find((entryId) => entryId !== null) ?? null
	const threadError =
		threadQuery.error?.message ??
		attachmentError ??
		latestFailureMessage([editDraft, sendDraft, deleteDraft])

	const thread = (
		<SupportThreadUI
			entries={entries}
			canManage={canManage}
			isLoading={threadQuery.isLoading}
			error={threadError}
			onRetry={() => void threadQuery.refetch()}
			editingEntryId={editingEntryId}
			pendingEntryId={pendingEntryId}
			onStartEdit={setEditingEntryId}
			onCancelEdit={() => setEditingEntryId(null)}
			onSaveEdit={handleSaveEdit}
			onSendDraft={handleSendDraft}
			onDeleteDraft={handleDeleteDraft}
			onOpenAttachment={(attachmentId) =>
				void handleOpenAttachment(attachmentId)
			}
		/>
	)

	const composer = canManage ? (
		<SupportComposerUI
			tabs={SUPPORT_COMPOSER_TABS}
			activeTab={activeTab}
			body={body}
			isSubmitting={addEntry.isPending}
			error={mutationErrorMessage(addEntry.error)}
			onTabChange={setActiveTab}
			onBodyChange={setBody}
			onSubmit={handleSubmit}
		/>
	) : null

	const sidePanel =
		ticket && detail ? (
			<SupportSidePanelUI
				ticket={detail}
				canManage={canManage}
				statusOptions={STATUS_OPTIONS}
				assigneeLabel={assigneeLabel(
					ticket.assignee_member_id,
					members.namesById,
				)}
				assignees={members.assignees}
				isUpdating={transition.isPending || assign.isPending}
				error={latestFailureMessage([transition, assign])}
				onChangeStatus={handleChangeStatus}
				onChangeAssignee={handleChangeAssignee}
			/>
		) : null

	return (
		<SupportTicketPageUI
			organizationName={organizationName}
			ticket={detail}
			isLoading={ticketQuery.isLoading}
			error={ticketQuery.error?.message ?? null}
			onRetry={() => void ticketQuery.refetch()}
			onBack={onBack}
			thread={thread}
			composer={composer}
			sidePanel={sidePanel}
		/>
	)
}

import type { UIEvent } from 'react'
import { useLayoutEffect, useRef, useState } from 'react'
import { type MessageAttachment, useMessages } from '#/hooks/use-chat'
import { useUploadFile } from '#/hooks/use-customers'
import { useFileUrls } from '#/hooks/use-file-url'
import { useGatewayUserId } from '#/hooks/use-gateway'
import { useSendTyping, useTypingUsers } from '#/hooks/use-typing'
import type { PendingAttachment } from '#/pages/chat/ui/message-composer-ui'

interface PendingAttachmentState extends PendingAttachment {
	ref?: MessageAttachment
}

/** How close to the bottom (px) still counts as "was at the bottom" — the
 * threshold for auto-scrolling to a newly arrived message. */
const NEAR_BOTTOM_THRESHOLD_PX = 80
/** How close to the top (px) triggers loading the previous page. */
const NEAR_TOP_THRESHOLD_PX = 60

/**
 * Everything a message thread's UI needs — composing, sending, editing,
 * scrolling, attachments — shared between the channel pane
 * (`chat-channel-feature.tsx`) and the thread panel (`#326`). A thread is
 * just a channel with its own id (`response.rs`'s model), so both mount
 * this against their own `channelId` and get the same behavior for free.
 */
export function useMessageThreadUi(channelId: string) {
	const currentUserId = useGatewayUserId()
	const {
		messages,
		hasMoreOlder,
		isLoadingOlder,
		isLoadingInitial,
		initialError,
		loadOlder,
		sendMessage,
		retrySend,
		editMessage,
		deleteMessage,
		toggleReaction,
	} = useMessages(channelId, currentUserId)
	const typingUserIds = useTypingUsers(channelId)
	const notifyTyping = useSendTyping(channelId)
	const uploadFile = useUploadFile()

	const [draft, setDraft] = useState('')
	const [pendingAttachments, setPendingAttachments] = useState<
		PendingAttachmentState[]
	>([])
	const [editingMessageId, setEditingMessageId] = useState<string | null>(null)
	const [editDraft, setEditDraft] = useState('')

	const scrollContainerRef = useRef<HTMLDivElement>(null)
	const previousScrollHeightRef = useRef<number | null>(null)
	const previousMessageCountRef = useRef(0)
	const wasNearBottomRef = useRef(true)

	// Keeps the reader's view stable when older messages are prepended, and
	// auto-scrolls to a newly arrived message only when they were already at
	// the bottom — so reading old messages is never interrupted by a new one.
	useLayoutEffect(() => {
		const container = scrollContainerRef.current
		if (!container) return

		if (previousScrollHeightRef.current !== null) {
			container.scrollTop +=
				container.scrollHeight - previousScrollHeightRef.current
			previousScrollHeightRef.current = null
			previousMessageCountRef.current = messages.length
			return
		}

		const grew = messages.length > previousMessageCountRef.current
		previousMessageCountRef.current = messages.length
		if (grew && wasNearBottomRef.current) {
			container.scrollTop = container.scrollHeight
		}
	}, [messages.length])

	function handleScroll(event: UIEvent<HTMLDivElement>) {
		const container = event.currentTarget
		wasNearBottomRef.current =
			container.scrollHeight - container.scrollTop - container.clientHeight <
			NEAR_BOTTOM_THRESHOLD_PX

		if (
			container.scrollTop < NEAR_TOP_THRESHOLD_PX &&
			hasMoreOlder &&
			!isLoadingOlder
		) {
			previousScrollHeightRef.current = container.scrollHeight
			loadOlder()
		}
	}

	const attachmentKeys = Array.from(
		new Set(
			messages.flatMap((entry) =>
				entry.message.attachments.map((attachment) => attachment.storage_key),
			),
		),
	)
	const fileUrls = useFileUrls(attachmentKeys)
	function resolveAttachmentUrl(storageKey: string) {
		return fileUrls.find((preview) => preview.key === storageKey)?.url
	}

	function handleAttachFiles(files: FileList) {
		for (const file of Array.from(files)) {
			const id = crypto.randomUUID()
			setPendingAttachments((previous) => [
				...previous,
				{ id, filename: file.name, status: 'uploading' },
			])
			uploadFile.mutate(file, {
				onSuccess: (result) => {
					// `useUploadFile` returns the raw `DataEnvelope` too — see
					// `customer-edit-feature.tsx`'s `uploaded.data.key` for the
					// same unwrap.
					setPendingAttachments((previous) =>
						previous.map((attachment) =>
							attachment.id === id
								? {
										...attachment,
										status: 'ready',
										ref: {
											storage_key: result.data.key,
											filename: file.name,
											mime_type: result.data.mime_type,
											size_bytes: result.data.size_bytes,
										},
									}
								: attachment,
						),
					)
				},
				onError: () => {
					setPendingAttachments((previous) =>
						previous.map((attachment) =>
							attachment.id === id
								? { ...attachment, status: 'failed' }
								: attachment,
						),
					)
				},
			})
		}
	}

	function handleRemoveAttachment(id: string) {
		setPendingAttachments((previous) =>
			previous.filter((attachment) => attachment.id !== id),
		)
	}

	function handleSend() {
		const attachments = pendingAttachments
			.filter(
				(
					attachment,
				): attachment is PendingAttachmentState & { ref: MessageAttachment } =>
					attachment.status === 'ready' && attachment.ref !== undefined,
			)
			.map((attachment) => attachment.ref)
		sendMessage(draft.trim(), attachments)
		setDraft('')
		setPendingAttachments([])
	}

	function handleStartEdit(messageId: string, content: string) {
		setEditingMessageId(messageId)
		setEditDraft(content)
	}

	function handleConfirmEdit() {
		if (!editingMessageId) return
		void editMessage(editingMessageId, editDraft)
		setEditingMessageId(null)
		setEditDraft('')
	}

	function handleCancelEdit() {
		setEditingMessageId(null)
		setEditDraft('')
	}

	return {
		currentUserId,
		typingCount: typingUserIds.size,
		messageListProps: {
			messages,
			currentUserId,
			isLoadingInitial,
			initialError,
			isLoadingOlder,
			hasMoreOlder,
			editingMessageId,
			editDraft,
			scrollContainerRef,
			onScroll: handleScroll,
			onRetry: retrySend,
			onStartEdit: handleStartEdit,
			onChangeEditDraft: setEditDraft,
			onConfirmEdit: handleConfirmEdit,
			onCancelEdit: handleCancelEdit,
			onDelete: deleteMessage,
			onToggleReaction: toggleReaction,
			resolveAttachmentUrl,
		},
		composerProps: {
			value: draft,
			onChange: setDraft,
			onSend: handleSend,
			onTyping: notifyTyping,
			pendingAttachments,
			onAttachFiles: handleAttachFiles,
			onRemoveAttachment: handleRemoveAttachment,
		},
	}
}

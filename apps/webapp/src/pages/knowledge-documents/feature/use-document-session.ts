import { useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useRef, useState } from 'react'
import { KNOWLEDGE_KEY } from '#/hooks/use-knowledge'
import { documentSessionApi } from '#/hooks/use-knowledge-documents'
import type { DocumentContent } from '#/pages/knowledge-documents/lib/document-api-types'
import {
	createDocumentSession,
	type DocumentSession,
	type SessionState,
} from '#/pages/knowledge-documents/lib/document-session'

interface Options {
	organizationId: string
	articleId: string
	canEdit: boolean
	loadedRevision: number
}

const INITIAL: SessionState = {
	phase: 'acquiring',
	saveState: 'idle',
	holderId: null,
	conflict: null,
	saveError: null,
}

export function useDocumentSession({
	organizationId,
	articleId,
	canEdit,
	loadedRevision,
}: Options) {
	const queryClient = useQueryClient()
	const [state, setState] = useState<SessionState>(INITIAL)
	const sessionRef = useRef<DocumentSession | null>(null)

	useEffect(() => {
		const session = createDocumentSession({
			api: documentSessionApi(organizationId, articleId),
			canEdit,
			loadedRevision,
			onState: setState,
		})
		sessionRef.current = session
		session.start()
		const onPageHide = () => session.pageHide()
		window.addEventListener('pagehide', onPageHide)
		return () => {
			window.removeEventListener('pagehide', onPageHide)
			sessionRef.current = null
			void session.leave().finally(() => {
				void queryClient.invalidateQueries({ queryKey: [KNOWLEDGE_KEY] })
			})
		}
	}, [organizationId, articleId, canEdit, loadedRevision, queryClient])

	const change = useCallback((read: () => DocumentContent) => {
		sessionRef.current?.change(read)
	}, [])

	return { state, change }
}

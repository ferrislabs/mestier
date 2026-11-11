import { createContext, useContext } from 'react'
import type { MentionKind } from '#/pages/knowledge-documents/lib/document-api-types'

export interface DocumentEnvironment {
	organizationId: string
	folderId: string | null
	editable: boolean
	resourceHref: (kind: MentionKind, id: string) => string
	openResource: (kind: MentionKind, id: string) => void
	openTable: (tableId: string, viewId: string) => void
}

const DocumentEnvironmentContext = createContext<DocumentEnvironment | null>(
	null,
)

export const DocumentEnvironmentProvider = DocumentEnvironmentContext.Provider

export function useDocumentEnvironment(): DocumentEnvironment {
	const environment = useContext(DocumentEnvironmentContext)
	if (!environment) {
		throw new Error('Le document n’a pas d’environnement.')
	}
	return environment
}

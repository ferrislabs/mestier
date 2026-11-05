import { createContext, useContext } from 'react'
import type { NodeIssue } from '#/pages/automation/lib/node-issues'

const NO_ISSUES: ReadonlyMap<string, NodeIssue[]> = new Map()

export const NodeIssuesContext =
	createContext<ReadonlyMap<string, NodeIssue[]>>(NO_ISSUES)

export function useNodeIssues(nodeId: string): readonly NodeIssue[] {
	return useContext(NodeIssuesContext).get(nodeId) ?? []
}

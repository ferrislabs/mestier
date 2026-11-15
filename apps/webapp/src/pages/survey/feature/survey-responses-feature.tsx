import { getCoreRowModel, useReactTable } from '@tanstack/react-table'
import { useMemo, useState } from 'react'
import { toast } from 'sonner'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import {
	useDeleteSurveyResponse,
	useSurvey,
	useSurveyGraphVersion,
	useSurveyResponses,
} from '#/hooks/use-surveys'
import { answerText } from '#/pages/survey/lib/answer-format'
import { refusalOf } from '#/pages/survey/lib/reason-messages'
import {
	answeredPath,
	fromSchema,
	identifierQuestion,
} from '#/pages/survey/lib/survey-graph'
import {
	ResponseDetailSheet,
	type ResponseRow,
	responseColumns,
	SurveyResponsesList,
} from '#/pages/survey/ui/survey-responses'

const COLUMNS = responseColumns()

export function SurveyResponsesFeature({ surveyId }: { surveyId: string }) {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const canManage = useHasPermission('MANAGE_SURVEYS')
	const [page, setPage] = useState(1)
	const [openId, setOpenId] = useState<string | null>(null)
	const [confirmDelete, setConfirmDelete] = useState(false)

	const survey = useSurvey(organizationId, surveyId)
	const responses = useSurveyResponses(organizationId, surveyId, page)
	const { members } = useReferenceCatalog(organizationId, {
		members: true,
		employeeProfiles: false,
		equipment: false,
		serviceRates: false,
		products: false,
		customUnits: false,
	})
	const deleteResponse = useDeleteSurveyResponse()

	const graph = useMemo(
		() => fromSchema(survey.data?.data.graph),
		[survey.data],
	)
	const currentVersion = survey.data?.data.graph_version ?? null
	const rows = useMemo<ResponseRow[]>(() => {
		const identifier = identifierQuestion(graph)
		const names = new Map(
			(members.data?.data ?? []).map((member) => [
				member.id,
				member.display_name,
			]),
		)
		return (responses.data?.data ?? []).map((response) => {
			const identified = identifier
				? answerText(response.answers[identifier.id])
				: null
			const member = response.member_id
				? (names.get(response.member_id) ?? 'Membre de l’organisation')
				: null
			const version = response.graph_version ?? null
			return {
				response,
				respondent: identified ?? member ?? 'Répondant anonyme',
				olderVersion:
					version !== null && version !== currentVersion ? version : null,
			}
		})
	}, [graph, members.data, responses.data, currentVersion])

	const table = useReactTable({
		data: rows,
		columns: COLUMNS,
		getCoreRowModel: getCoreRowModel(),
		getRowId: (row) => row.response.id,
	})

	const pagination = responses.data?.pagination
	const opened = rows.find((row) => row.response.id === openId) ?? null
	const versionQuery = useSurveyGraphVersion(
		organizationId,
		surveyId,
		opened?.olderVersion ?? null,
	)
	const openedGraph = useMemo(() => {
		if (!opened?.olderVersion || versionQuery.isError) return graph
		const version = versionQuery.data?.data
		return version ? fromSchema(version.graph) : null
	}, [opened, graph, versionQuery.data, versionQuery.isError])

	const remove = async () => {
		if (!opened) return
		setConfirmDelete(false)
		try {
			await deleteResponse.mutateAsync({
				path: {
					organization_id: organizationId,
					response_id: opened.response.id,
				},
			})
			setOpenId(null)
			if (rows.length === 1 && page > 1) setPage(page - 1)
			toast.success('Réponse supprimée.')
		} catch (failure) {
			toast.error(refusalOf(failure).message, { duration: 6000 })
		}
	}

	return (
		<>
			<SurveyResponsesList
				title={survey.data?.data.title ?? 'Questionnaire'}
				table={table}
				isEmpty={rows.length === 0}
				isLoading={responses.isLoading}
				error={responses.error ? refusalOf(responses.error).message : null}
				page={page}
				pageCount={pagination?.last_page ?? null}
				total={pagination?.total ?? null}
				onPage={setPage}
				onOpen={(row) => setOpenId(row.response.id)}
			/>
			<ResponseDetailSheet
				row={opened}
				path={
					opened && openedGraph
						? answeredPath(openedGraph, opened.response.answers)
						: []
				}
				pathLoading={opened?.olderVersion != null && versionQuery.isLoading}
				missingVersion={
					opened?.olderVersion != null && versionQuery.isError
						? opened.olderVersion
						: null
				}
				currentVersion={currentVersion}
				canManage={canManage}
				onClose={() => setOpenId(null)}
				onDelete={() => setConfirmDelete(true)}
			/>
			<ConfirmDialog
				open={confirmDelete}
				onOpenChange={setConfirmDelete}
				title="Supprimer cette réponse ?"
				description="La suppression est définitive. Les événements déjà émis pour cette réponse ne sont pas retirés."
				confirmLabel="Supprimer"
				destructive
				onConfirm={() => void remove()}
			/>
		</>
	)
}

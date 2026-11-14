import { useNavigate } from '@tanstack/react-router'
import { getCoreRowModel, useReactTable } from '@tanstack/react-table'
import { useMemo, useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import {
	useDeleteSurveyResponse,
	useSurvey,
	useSurveyResponses,
} from '#/hooks/use-surveys'
import { buildOrgPath } from '#/modules/org-path'
import { useNotice } from '#/pages/customers/feature/use-notice'
import { answerText } from '#/pages/survey/lib/answer-format'
import { refusalOf } from '#/pages/survey/lib/reason-messages'
import { surveyPath } from '#/pages/survey/lib/survey-format'
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
	const navigate = useNavigate()
	const canManage = useHasPermission('MANAGE_SURVEYS')
	const notice = useNotice()
	const [page, setPage] = useState(1)
	const [openId, setOpenId] = useState<string | null>(null)
	const [confirmDelete, setConfirmDelete] = useState(false)
	const [actionError, setActionError] = useState<string | null>(null)

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
			return {
				response,
				respondent: identified ?? member ?? 'Répondant anonyme',
			}
		})
	}, [graph, members.data, responses.data])

	const table = useReactTable({
		data: rows,
		columns: COLUMNS,
		getCoreRowModel: getCoreRowModel(),
		getRowId: (row) => row.response.id,
	})

	const pagination = responses.data?.pagination
	const opened = rows.find((row) => row.response.id === openId) ?? null

	const remove = async () => {
		if (!opened) return
		setConfirmDelete(false)
		setActionError(null)
		try {
			await deleteResponse.mutateAsync({
				path: {
					organization_id: organizationId,
					response_id: opened.response.id,
				},
			})
			setOpenId(null)
			notice.show('Réponse supprimée.')
		} catch (failure) {
			setActionError(refusalOf(failure).message)
		}
	}

	return (
		<>
			<SurveyResponsesList
				title={survey.data?.data.title ?? 'Questionnaire'}
				table={table}
				isEmpty={rows.length === 0}
				isLoading={responses.isLoading}
				error={
					actionError ??
					(responses.error ? refusalOf(responses.error).message : null)
				}
				notice={notice.message}
				page={page}
				pageCount={pagination?.last_page ?? null}
				total={pagination?.total ?? null}
				onDismissNotice={notice.clear}
				onBack={() =>
					void navigate({
						to: buildOrgPath(activeOrganization.slug, surveyPath(surveyId)),
					})
				}
				onPage={setPage}
				onOpen={(row) => setOpenId(row.response.id)}
			/>
			<ResponseDetailSheet
				row={opened}
				path={opened ? answeredPath(graph, opened.response.answers) : []}
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

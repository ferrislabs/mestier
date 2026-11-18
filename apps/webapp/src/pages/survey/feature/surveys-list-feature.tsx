import { useNavigate } from '@tanstack/react-router'
import { getCoreRowModel, useReactTable } from '@tanstack/react-table'
import { useMemo, useRef, useState } from 'react'
import { toast } from 'sonner'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import {
	useCampaigns,
	useCloseSurvey,
	useCreateSurvey,
	useDeleteSurvey,
	useDuplicateSurvey,
	useOpenSurvey,
	useSurveys,
} from '#/hooks/use-surveys'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import {
	SurveyCreateDialogFeature,
	type SurveyCreateValues,
} from '#/pages/survey/feature/survey-create-dialog-feature'
import {
	type LifecycleAction,
	lifecycleCopy,
} from '#/pages/survey/lib/lifecycle'
import { refusalOf } from '#/pages/survey/lib/reason-messages'
import {
	publicSurveyUrl,
	surveyPath,
	surveyResponsesPath,
} from '#/pages/survey/lib/survey-format'
import {
	type SurveyAction,
	type SurveyRow,
	surveyColumns,
} from '#/pages/survey/ui/survey-columns'
import { SurveysList } from '#/pages/survey/ui/surveys-list'

interface Pending {
	action: LifecycleAction
	row: SurveyRow
}

export function SurveysListFeature() {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const navigate = useNavigate()
	const canManage = useHasPermission('MANAGE_SURVEYS')
	const [creating, setCreating] = useState(false)
	const [pending, setPending] = useState<Pending | null>(null)

	const surveys = useSurveys(organizationId)
	const campaigns = useCampaigns(organizationId)
	const createSurvey = useCreateSurvey()
	const duplicateSurvey = useDuplicateSurvey()
	const openSurvey = useOpenSurvey()
	const closeSurvey = useCloseSurvey()
	const deleteSurvey = useDeleteSurvey()

	const rows = useMemo<SurveyRow[]>(() => {
		const names = new Map(
			(campaigns.data?.data ?? []).map((campaign) => [
				campaign.id,
				campaign.name,
			]),
		)
		return (surveys.data?.data ?? []).map((survey) => ({
			survey,
			campaignName: survey.campaign_id
				? (names.get(survey.campaign_id) ?? null)
				: null,
		}))
	}, [surveys.data, campaigns.data])

	const goTo = (path: string) =>
		navigate({ to: buildOrgPath(activeOrganization.slug, path) })

	const run = async (work: () => Promise<unknown>, done: string) => {
		try {
			await work()
			toast.success(done)
		} catch (error) {
			toast.error(refusalOf(error).message, { duration: 6000 })
		}
	}

	const onAction = (action: SurveyAction, row: SurveyRow) => {
		const path = { organization_id: organizationId, survey_id: row.survey.id }
		switch (action) {
			case 'edit':
				void goTo(surveyPath(row.survey.id))
				return
			case 'responses':
				void goTo(surveyResponsesPath(row.survey.id))
				return
			case 'copy-link': {
				const slug = row.survey.public_slug
				if (!slug) return
				void run(
					() =>
						navigator.clipboard.writeText(
							publicSurveyUrl(window.location.origin, slug),
						),
					'Lien public copié.',
				)
				return
			}
			case 'duplicate':
				void run(
					() => duplicateSurvey.mutateAsync({ path }),
					'Questionnaire dupliqué en brouillon.',
				)
				return
			case 'open':
			case 'close':
			case 'delete':
				setPending({ action, row })
				return
		}
	}

	const onActionRef = useRef(onAction)
	onActionRef.current = onAction
	const columns = useMemo(
		() =>
			surveyColumns({
				canManage,
				duplicating: duplicateSurvey.isPending,
				onAction: (action, row) => onActionRef.current(action, row),
			}),
		[canManage, duplicateSurvey.isPending],
	)

	const table = useReactTable({
		data: rows,
		columns,
		getCoreRowModel: getCoreRowModel(),
		getRowId: (row) => row.survey.id,
	})

	const confirm = async () => {
		if (!pending) return
		const { action, row } = pending
		const path = { organization_id: organizationId, survey_id: row.survey.id }
		const copy = lifecycleCopy(action, row.survey.title)
		setPending(null)
		await run(() => {
			if (action === 'open') return openSurvey.mutateAsync({ path })
			if (action === 'close') return closeSurvey.mutateAsync({ path })
			return deleteSurvey.mutateAsync({ path })
		}, copy.done)
	}

	const create = async (values: SurveyCreateValues) => {
		const created = await createSurvey.mutateAsync({
			path: { organization_id: organizationId },
			body: values,
		})
		setCreating(false)
		await goTo(surveyPath(created.data.id))
	}

	const copy = pending
		? lifecycleCopy(pending.action, pending.row.survey.title)
		: null

	return (
		<>
			<SurveysList
				table={table}
				isEmpty={rows.length === 0}
				isLoading={surveys.isLoading}
				error={surveys.error ? mutationErrorMessage(surveys.error) : null}
				canManage={canManage}
				onCreate={() => {
					createSurvey.reset()
					setCreating(true)
				}}
			/>
			<SurveyCreateDialogFeature
				open={creating}
				isPending={createSurvey.isPending}
				error={
					createSurvey.error ? refusalOf(createSurvey.error).message : null
				}
				onOpenChange={setCreating}
				onSubmit={create}
			/>
			{copy ? (
				<ConfirmDialog
					open
					onOpenChange={(open) => {
						if (!open) setPending(null)
					}}
					title={copy.title}
					description={copy.description}
					confirmLabel={copy.confirmLabel}
					destructive={copy.destructive}
					onConfirm={() => void confirm()}
				/>
			) : null}
		</>
	)
}

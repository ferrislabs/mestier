import { useNavigate } from '@tanstack/react-router'
import { getCoreRowModel, useReactTable } from '@tanstack/react-table'
import { useMemo, useRef, useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import {
	useCampaigns,
	useCreateCampaign,
	useDeleteCampaign,
	useSurveys,
} from '#/hooks/use-surveys'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import { useNotice } from '#/pages/customers/feature/use-notice'
import { CampaignFormFeature } from '#/pages/survey/feature/campaign-form-feature'
import {
	type CampaignFormValues,
	campaignRequest,
	campaignValuesOf,
} from '#/pages/survey/lib/campaign-form'
import { refusalOf } from '#/pages/survey/lib/reason-messages'
import { campaignPath } from '#/pages/survey/lib/survey-format'
import {
	type CampaignRow,
	CampaignsList,
	campaignColumns,
} from '#/pages/survey/ui/campaigns-list'

export function CampaignsListFeature() {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const navigate = useNavigate()
	const canManage = useHasPermission('MANAGE_SURVEYS')
	const notice = useNotice()
	const [creating, setCreating] = useState(false)
	const [deleting, setDeleting] = useState<CampaignRow | null>(null)
	const [actionError, setActionError] = useState<string | null>(null)

	const campaigns = useCampaigns(organizationId)
	const surveys = useSurveys(organizationId)
	const createCampaign = useCreateCampaign()
	const deleteCampaign = useDeleteCampaign()

	const rows = useMemo<CampaignRow[]>(() => {
		const counts = new Map<string, number>()
		for (const survey of surveys.data?.data ?? []) {
			if (survey.campaign_id) {
				counts.set(
					survey.campaign_id,
					(counts.get(survey.campaign_id) ?? 0) + 1,
				)
			}
		}
		return (campaigns.data?.data ?? []).map((campaign) => ({
			campaign,
			surveyCount: counts.get(campaign.id) ?? 0,
		}))
	}, [campaigns.data, surveys.data])

	const handlers = {
		onEdit: (row: CampaignRow) =>
			void navigate({
				to: buildOrgPath(
					activeOrganization.slug,
					campaignPath(row.campaign.id),
				),
			}),
		onDelete: (row: CampaignRow) => setDeleting(row),
	}
	const handlersRef = useRef(handlers)
	handlersRef.current = handlers
	const columns = useMemo(
		() =>
			campaignColumns({
				canManage,
				onEdit: (row) => handlersRef.current.onEdit(row),
				onDelete: (row) => handlersRef.current.onDelete(row),
			}),
		[canManage],
	)
	const table = useReactTable({
		data: rows,
		columns,
		getCoreRowModel: getCoreRowModel(),
		getRowId: (row) => row.campaign.id,
	})

	const create = async (values: CampaignFormValues) => {
		await createCampaign.mutateAsync({
			path: { organization_id: organizationId },
			body: campaignRequest(values),
		})
		setCreating(false)
		notice.show('Campagne créée.')
	}

	const remove = async () => {
		if (!deleting) return
		const row = deleting
		setDeleting(null)
		setActionError(null)
		try {
			await deleteCampaign.mutateAsync({
				path: { organization_id: organizationId, campaign_id: row.campaign.id },
			})
			notice.show('Campagne supprimée.')
		} catch (failure) {
			setActionError(refusalOf(failure).message)
		}
	}

	return (
		<>
			<CampaignsList
				table={table}
				isEmpty={rows.length === 0}
				isLoading={campaigns.isLoading}
				error={
					actionError ??
					(campaigns.error ? mutationErrorMessage(campaigns.error) : null)
				}
				canManage={canManage}
				notice={notice.message}
				onDismissNotice={notice.clear}
				onCreate={() => {
					createCampaign.reset()
					setCreating(true)
				}}
			/>
			<Dialog open={creating} onOpenChange={setCreating}>
				<DialogContent className="sm:max-w-xl">
					<DialogHeader>
						<DialogTitle>Nouvelle campagne</DialogTitle>
						<DialogDescription>
							Rattachez-lui ensuite des questionnaires depuis leurs réglages.
						</DialogDescription>
					</DialogHeader>
					{creating ? (
						<CampaignFormFeature
							initialValues={campaignValuesOf(null)}
							submitLabel="Créer la campagne"
							readOnly={false}
							isPending={createCampaign.isPending}
							error={
								createCampaign.error
									? refusalOf(createCampaign.error).message
									: null
							}
							onSubmit={create}
						/>
					) : null}
				</DialogContent>
			</Dialog>
			{deleting ? (
				<ConfirmDialog
					open
					onOpenChange={(open) => {
						if (!open) setDeleting(null)
					}}
					title={`Supprimer « ${deleting.campaign.name} » ?`}
					description={
						deleting.surveyCount > 0
							? 'Des questionnaires y sont rattachés : la suppression sera refusée tant qu’ils le restent.'
							: 'La suppression est définitive.'
					}
					confirmLabel="Supprimer"
					destructive
					onConfirm={() => void remove()}
				/>
			) : null}
		</>
	)
}

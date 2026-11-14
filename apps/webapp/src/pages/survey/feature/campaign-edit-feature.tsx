import { useNavigate } from '@tanstack/react-router'
import { ArrowLeft, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell, SectionCard } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import {
	useCampaign,
	useDeleteCampaign,
	useUpdateCampaign,
} from '#/hooks/use-surveys'
import { buildOrgPath } from '#/modules/org-path'
import { useNotice } from '#/pages/customers/feature/use-notice'
import { Notice } from '#/pages/customers/ui/notice'
import { CampaignFormFeature } from '#/pages/survey/feature/campaign-form-feature'
import {
	type CampaignFormValues,
	campaignRequest,
	campaignValuesOf,
} from '#/pages/survey/lib/campaign-form'
import { refusalOf } from '#/pages/survey/lib/reason-messages'
import { campaignsPath } from '#/pages/survey/lib/survey-format'
import { CampaignStatusBadge } from '#/pages/survey/ui/survey-badges'
import { ErrorBanner, LoadingCard } from '#/pages/survey/ui/surveys-list'

export function CampaignEditFeature({ campaignId }: { campaignId: string }) {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const navigate = useNavigate()
	const canManage = useHasPermission('MANAGE_SURVEYS')
	const notice = useNotice()
	const campaign = useCampaign(organizationId, campaignId)
	const updateCampaign = useUpdateCampaign()
	const deleteCampaign = useDeleteCampaign()
	const [confirmDelete, setConfirmDelete] = useState(false)
	const [deleteError, setDeleteError] = useState<string | null>(null)

	const back = () =>
		navigate({ to: buildOrgPath(activeOrganization.slug, campaignsPath()) })

	const save = async (values: CampaignFormValues) => {
		await updateCampaign.mutateAsync({
			path: { organization_id: organizationId, campaign_id: campaignId },
			body: campaignRequest(values),
		})
		notice.show('Campagne enregistrée.')
	}

	const remove = async () => {
		setConfirmDelete(false)
		setDeleteError(null)
		try {
			await deleteCampaign.mutateAsync({
				path: { organization_id: organizationId, campaign_id: campaignId },
			})
			await back()
		} catch (failure) {
			setDeleteError(refusalOf(failure).message)
		}
	}

	const data = campaign.data?.data

	return (
		<PageShell>
			<PageHeader
				leading={
					<Button
						variant="ghost"
						size="icon-sm"
						aria-label="Retour aux campagnes"
						onClick={() => void back()}
					>
						<ArrowLeft aria-hidden />
					</Button>
				}
				title={data?.name ?? 'Campagne'}
				actions={data ? <CampaignStatusBadge status={data.status} /> : null}
			/>
			<Notice message={notice.message} onDismiss={notice.clear} />
			<ErrorBanner message={deleteError} />
			{campaign.isLoading ? (
				<LoadingCard label="Chargement de la campagne…" />
			) : !data ? (
				<ErrorBanner
					message={
						campaign.error
							? refusalOf(campaign.error).message
							: 'Cette campagne est introuvable.'
					}
				/>
			) : (
				<SectionCard className="p-5">
					<CampaignFormFeature
						key={`${data.id}:${data.name}:${data.starts_at}:${data.ends_at}`}
						initialValues={campaignValuesOf(data)}
						submitLabel="Enregistrer"
						readOnly={!canManage}
						isPending={updateCampaign.isPending}
						error={
							updateCampaign.error
								? refusalOf(updateCampaign.error).message
								: null
						}
						onSubmit={save}
						secondaryAction={
							canManage ? (
								<Button
									type="button"
									variant="ghost"
									onClick={() => setConfirmDelete(true)}
								>
									<Trash2 aria-hidden />
									Supprimer
								</Button>
							) : null
						}
					/>
				</SectionCard>
			)}
			<ConfirmDialog
				open={confirmDelete}
				onOpenChange={setConfirmDelete}
				title={`Supprimer « ${data?.name ?? 'la campagne'} » ?`}
				description="La suppression est refusée tant que des questionnaires y sont rattachés."
				confirmLabel="Supprimer"
				destructive
				onConfirm={() => void remove()}
			/>
		</PageShell>
	)
}

import { Plus } from 'lucide-react'
import { useState } from 'react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { PageShell, SectionCard } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	type PromotionDialogState,
	PromotionDialogsFeature,
} from '#/pages/cursus/promotions/feature/promotion-dialogs-feature'
import {
	useArchivePromotion,
	usePromotion,
} from '#/pages/cursus/promotions/feature/use-promotions'
import { PromotionHeader } from '#/pages/cursus/promotions/ui/promotion-header'
import { useTrainings } from '#/pages/cursus/shared/feature/cursus-queries'
import { TriadCreateFeature } from '#/pages/cursus/triads/feature/triad-create-feature'
import { useTriads } from '#/pages/cursus/triads/feature/use-triads'
import { TriadsTable } from '#/pages/cursus/triads/ui/triads-table'

export interface PromotionDetailFeatureProps {
	promotionId: string
	onBack: () => void
	onOpenTriad: (triadId: string) => void
	onDeclareSchoolTutor?: () => void
}

export function PromotionDetailFeature({
	promotionId,
	onBack,
	onOpenTriad,
	onDeclareSchoolTutor,
}: PromotionDetailFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const canManageCatalog = useHasPermission('MANAGE_CURSUS_CATALOG')
	const canManageTriads = useHasPermission('MANAGE_CURSUS_TRIADS')
	const [dialog, setDialog] = useState<PromotionDialogState>({ kind: 'none' })
	const [creatingTriad, setCreatingTriad] = useState(false)
	const promotionQuery = usePromotion(activeOrganizationId, promotionId)
	const triads = useTriads(activeOrganizationId, {
		promotionId,
		closed: undefined,
	})
	const trainings = useTrainings(activeOrganizationId)
	const archive = useArchivePromotion()
	const promotion = promotionQuery.data?.data

	if (promotionQuery.isError) {
		return (
			<PageShell>
				<p role="alert" className="text-state-alert">
					Promotion introuvable.
				</p>
			</PageShell>
		)
	}
	if (!promotion) {
		return (
			<PageShell>
				<p className="text-muted-foreground">Chargement…</p>
			</PageShell>
		)
	}

	const trainingRows = trainings.data?.data ?? []
	const trainingName =
		trainingRows.find((training) => training.id === promotion.training_id)
			?.name ?? '—'
	const rows = triads.data?.data ?? []
	const archived = Boolean(promotion.archived_at)

	return (
		<PageShell>
			<PromotionHeader
				promotion={promotion}
				trainingName={trainingName}
				canManage={canManageCatalog}
				onBack={onBack}
				onEdit={() => setDialog({ kind: 'edit', promotion })}
				onArchive={() =>
					archive.mutate({
						path: {
							organization_id: activeOrganizationId,
							promotion_id: promotion.id,
						},
					})
				}
			/>
			{archive.error ? (
				<p role="alert" className="text-[13px] text-state-alert">
					{mutationErrorMessage(archive.error)}
				</p>
			) : null}
			<SectionCard>
				<div className="flex items-center justify-between gap-3 border-b px-5 py-4">
					<h2 className="font-semibold">Trinômes ({rows.length})</h2>
					{canManageTriads && !archived ? (
						<Button
							type="button"
							size="sm"
							onClick={() => setCreatingTriad(true)}
						>
							<Plus />
							Nouveau trinôme
						</Button>
					) : null}
				</div>
				<div className="p-5">
					{rows.length === 0 && !triads.isLoading ? (
						<EmptyState title="Aucun trinôme dans cette promotion" />
					) : (
						<TriadsTable
							triads={rows}
							showPromotion={false}
							onOpen={onOpenTriad}
						/>
					)}
				</div>
			</SectionCard>
			{canManageCatalog ? (
				<PromotionDialogsFeature
					state={dialog}
					trainings={trainingRows
						.filter(
							(training) =>
								!training.archived_at || training.id === promotion.training_id,
						)
						.map((training) => ({ value: training.id, label: training.name }))}
					onClose={() => setDialog({ kind: 'none' })}
				/>
			) : null}
			{canManageTriads ? (
				<TriadCreateFeature
					open={creatingTriad}
					fixedPromotionId={promotion.id}
					onOpenChange={setCreatingTriad}
					onCreated={onOpenTriad}
					onDeclareSchoolTutor={onDeclareSchoolTutor}
				/>
			) : null}
		</PageShell>
	)
}

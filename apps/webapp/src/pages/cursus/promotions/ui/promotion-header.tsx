import { Archive, ArrowLeft, Pencil } from 'lucide-react'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { PageHeader } from '#/components/ui/surface'
import type { Promotion } from '#/pages/cursus/promotions/feature/use-promotions'

interface PromotionHeaderProps {
	promotion: Promotion
	trainingName: string
	canManage: boolean
	onBack: () => void
	onEdit: () => void
	onArchive: () => void
}

export function PromotionHeader({
	promotion,
	trainingName,
	canManage,
	onBack,
	onEdit,
	onArchive,
}: PromotionHeaderProps) {
	return (
		<PageHeader
			leading={
				<Button
					type="button"
					variant="ghost"
					size="icon"
					aria-label="Retour aux promotions"
					onClick={onBack}
				>
					<ArrowLeft />
				</Button>
			}
			title={promotion.label}
			description={`${trainingName} · ${promotion.start_year}–${promotion.end_year}`}
			actions={
				<>
					{promotion.archived_at ? (
						<Badge variant="secondary">Archivée</Badge>
					) : null}
					{canManage ? (
						<>
							<Button type="button" variant="outline" onClick={onEdit}>
								<Pencil />
								Modifier
							</Button>
							{promotion.archived_at ? null : (
								<Button type="button" variant="outline" onClick={onArchive}>
									<Archive />
									Archiver
								</Button>
							)}
						</>
					) : null}
				</>
			}
		/>
	)
}

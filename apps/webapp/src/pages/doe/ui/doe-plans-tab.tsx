import { ArrowDown, ArrowUp, FileText, X } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import type { DoeFile } from '#/pages/doe/types'
import { FileDropZone } from '#/pages/doe/ui/file-drop-zone'

interface DoePlansTabProps {
	sectionNumber: number
	title: string
	plans: DoeFile[]
	isUploading: boolean
	onUpload: (files: File[]) => void
	onCaptionChange: (index: number, caption: string) => void
	onMove: (index: number, delta: -1 | 1) => void
	onRemove: (index: number) => void
}

export function DoePlansTab({
	sectionNumber,
	title,
	plans,
	isUploading,
	onUpload,
	onCaptionChange,
	onMove,
	onRemove,
}: DoePlansTabProps) {
	return (
		<SectionCard>
			<SectionHeader title={`${sectionNumber}. ${title}`} />
			<div className="space-y-4 p-5">
				<RequirePermission permission="MANAGE_PLANNING">
					<FileDropZone
						accept="application/pdf,image/*"
						label="Déposez des plans ici"
						hint="Ou cliquez pour choisir des PDF ou des images"
						isUploading={isUploading}
						onFiles={onUpload}
					/>
				</RequirePermission>
				{plans.length === 0 ? (
					<p className="text-sm text-muted-foreground">Aucun plan.</p>
				) : (
					<ul className="divide-y rounded-md border">
						{plans.map((plan, index) => (
							<li key={plan.key} className="flex items-center gap-3 px-3 py-2">
								<FileText className="size-4 shrink-0 text-muted-foreground" />
								<input
									className="min-w-0 flex-1 rounded-md border bg-card px-2 py-1 text-sm"
									value={plan.caption ?? ''}
									placeholder="Désignation du plan"
									aria-label={`Désignation du plan ${index + 1}`}
									onChange={(event) =>
										onCaptionChange(index, event.target.value)
									}
								/>
								<RequirePermission permission="MANAGE_PLANNING">
									<Button
										type="button"
										variant="ghost"
										size="icon-sm"
										disabled={index === 0}
										onClick={() => onMove(index, -1)}
									>
										<ArrowUp />
										<span className="sr-only">Monter le plan {index + 1}</span>
									</Button>
									<Button
										type="button"
										variant="ghost"
										size="icon-sm"
										disabled={index === plans.length - 1}
										onClick={() => onMove(index, 1)}
									>
										<ArrowDown />
										<span className="sr-only">
											Descendre le plan {index + 1}
										</span>
									</Button>
									<Button
										type="button"
										variant="ghost"
										size="icon-sm"
										onClick={() => onRemove(index)}
									>
										<X />
										<span className="sr-only">Retirer le plan {index + 1}</span>
									</Button>
								</RequirePermission>
							</li>
						))}
					</ul>
				)}
			</div>
		</SectionCard>
	)
}

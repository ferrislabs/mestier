import { ArrowDown, ArrowUp, FileText, X } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Input } from '#/components/ui/input'
import type { DoeFile } from '#/pages/doe/types'
import { FileDropZone } from '#/pages/doe/ui/file-drop-zone'
import { IconButton } from '#/pages/doe/ui/icon-button'
import { SectionFrame } from '#/pages/doe/ui/section-frame'

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
		<SectionFrame number={sectionNumber} title={title}>
			<div className="grid gap-4 p-5">
				<RequirePermission permission="MANAGE_PLANNING">
					<FileDropZone
						accept="application/pdf,image/*"
						label="Dépose des plans ici"
						hint="Ou clique pour choisir des PDF ou des images"
						isUploading={isUploading}
						onFiles={onUpload}
					/>
				</RequirePermission>
				{plans.length === 0 ? (
					<p className="py-4 text-center text-muted-foreground text-sm">
						Aucun plan dans ce thème.
					</p>
				) : (
					<ul className="divide-y rounded-md border">
						{plans.map((plan, index) => {
							const position = index + 1
							return (
								<li
									key={plan.key}
									className="flex flex-wrap items-center gap-3 px-3 py-2.5"
								>
									<span className="w-5 text-right text-muted-foreground text-sm tabular-nums">
										{position}
									</span>
									<span
										aria-hidden
										className="flex h-10 w-14 shrink-0 items-center justify-center rounded border bg-muted text-muted-foreground"
									>
										<FileText className="size-5" />
									</span>
									<Input
										value={plan.caption ?? ''}
										placeholder="Désignation du plan"
										aria-label={`Désignation du plan ${position}`}
										autoComplete="off"
										className="h-8 min-w-0 flex-1 basis-40"
										onChange={(event) =>
											onCaptionChange(index, event.target.value)
										}
									/>
									<RequirePermission permission="MANAGE_PLANNING">
										<div className="flex items-center gap-0.5">
											<IconButton
												label={`Monter le plan ${position}`}
												icon={ArrowUp}
												disabled={index === 0}
												onClick={() => onMove(index, -1)}
											/>
											<IconButton
												label={`Descendre le plan ${position}`}
												icon={ArrowDown}
												disabled={index === plans.length - 1}
												onClick={() => onMove(index, 1)}
											/>
											<IconButton
												label={`Retirer le plan ${position}`}
												icon={X}
												onClick={() => onRemove(index)}
											/>
										</div>
									</RequirePermission>
								</li>
							)
						})}
					</ul>
				)}
			</div>
		</SectionFrame>
	)
}

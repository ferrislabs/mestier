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
	listed: string[]
	onListedChange: (names: string[]) => void
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
	listed,
	onListedChange,
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
				<div className="grid gap-2">
					<span className="font-medium text-sm">Plans fournis à part</span>
					<p className="text-muted-foreground text-xs">
						Entrée ajoute une ligne. Ils sont listés dans le DOE, sans pièce
						jointe.
					</p>
					<ul className="grid gap-1.5" data-listed-plans>
						{(listed.length === 0 ? [''] : listed).map((name, index) => {
							const position = plans.length + index + 1
							return (
								// biome-ignore lint/suspicious/noArrayIndexKey: names are free text and may repeat
								<li key={index} className="flex items-center gap-3">
									<span className="w-5 text-right text-muted-foreground text-sm tabular-nums">
										{position}
									</span>
									<Input
										value={name}
										placeholder="Nom du plan, par exemple Détail terrasse A12"
										aria-label={`Nom du plan ${position}`}
										autoComplete="off"
										className="h-8 min-w-0 flex-1"
										onChange={(event) => {
											const next = listed.length === 0 ? [''] : [...listed]
											next[index] = event.target.value
											onListedChange(next)
										}}
										onKeyDown={(event) => {
											const list = event.currentTarget.closest('ul')
											if (event.key === 'Enter') {
												event.preventDefault()
												if (name.trim() === '') return
												const next = [...listed]
												next.splice(index + 1, 0, '')
												onListedChange(next)
												requestAnimationFrame(() =>
													list?.querySelectorAll('input')[index + 1]?.focus(),
												)
											} else if (
												event.key === 'Backspace' &&
												name === '' &&
												listed.length > 0
											) {
												event.preventDefault()
												onListedChange(listed.filter((_, at) => at !== index))
												requestAnimationFrame(() =>
													list
														?.querySelectorAll('input')
														[Math.max(index - 1, 0)]?.focus(),
												)
											}
										}}
									/>
									<IconButton
										label={`Retirer le plan ${position}`}
										icon={X}
										onClick={() =>
											onListedChange(listed.filter((_, at) => at !== index))
										}
									/>
								</li>
							)
						})}
					</ul>
				</div>
			</div>
		</SectionFrame>
	)
}

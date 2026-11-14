import { Camera, ImageIcon, ImagePlus, Loader2, X } from 'lucide-react'
import type { ChangeEvent } from 'react'
import { cn } from '#/lib/utils'
import type { PhotoItem } from '#/pages/survey-respond/lib/respond-types'

const STATUS_LABELS: Record<PhotoItem['status'], string> = {
	converting: 'Conversion…',
	uploading: 'Envoi…',
	done: 'Envoyée',
	error: 'Refusée',
}

const PICKER_CLASS =
	'flex min-h-12 flex-1 cursor-pointer items-center justify-center gap-2 rounded-xl border border-dashed bg-card px-4 text-base font-medium transition-colors hover:border-primary/60 hover:bg-primary/5 has-[:focus-visible]:ring-[3px] has-[:focus-visible]:ring-ring/50'

function filesOf(event: ChangeEvent<HTMLInputElement>): File[] {
	const files = Array.from(event.target.files ?? [])
	event.target.value = ''
	return files
}

export function PhotoAnswer({
	id,
	items,
	limit,
	notice,
	describedBy,
	onAdd,
	onRemove,
}: {
	id: string
	items: readonly PhotoItem[]
	limit: number
	notice: string | null
	describedBy: string | undefined
	onAdd: (files: File[]) => void
	onRemove: (itemId: string) => void
}) {
	const kept = items.filter((item) => item.status !== 'error').length
	const full = kept >= limit
	const limitId = `${id}-limit`
	const inputDescription = [limitId, describedBy].filter(Boolean).join(' ')
	return (
		<div className="flex flex-col gap-4">
			<p id={limitId} className="text-sm text-muted-foreground">
				{kept} sur {limit} photo{limit > 1 ? 's' : ''} au plus.
			</p>
			{full ? null : (
				<div className="flex flex-col gap-3 sm:flex-row">
					<label className={PICKER_CLASS}>
						<Camera className="size-5" aria-hidden />
						Prendre une photo
						<input
							type="file"
							accept="image/*"
							capture="environment"
							className="sr-only"
							aria-describedby={inputDescription}
							onChange={(event) => onAdd(filesOf(event))}
						/>
					</label>
					<label className={PICKER_CLASS}>
						<ImagePlus className="size-5" aria-hidden />
						Choisir des photos
						<input
							id={id}
							type="file"
							accept="image/*,.heic,.heif"
							multiple
							className="sr-only"
							aria-describedby={inputDescription}
							onChange={(event) => onAdd(filesOf(event))}
						/>
					</label>
				</div>
			)}
			{notice ? (
				<p role="alert" className="text-sm text-destructive">
					{notice}
				</p>
			) : null}
			{items.length > 0 ? (
				<ul
					aria-label="Photos jointes"
					className="grid grid-cols-2 gap-3 sm:grid-cols-3"
				>
					{items.map((item) => (
						<li
							key={item.id}
							className={cn(
								'relative flex flex-col overflow-hidden rounded-xl border bg-card',
								item.status === 'error' && 'border-destructive/60',
							)}
						>
							<div className="relative flex aspect-square items-center justify-center bg-muted">
								{item.previewUrl ? (
									<img
										src={item.previewUrl}
										alt={`Aperçu de ${item.name}`}
										className="size-full object-cover"
									/>
								) : (
									<ImageIcon
										className="size-8 text-muted-foreground"
										aria-hidden
									/>
								)}
								{item.status === 'converting' || item.status === 'uploading' ? (
									<span className="absolute inset-0 flex items-center justify-center bg-background/60">
										<Loader2
											className="size-6 motion-safe:animate-spin"
											aria-hidden
										/>
									</span>
								) : null}
							</div>
							<div className="flex items-start gap-1 p-2">
								<div className="min-w-0 flex-1">
									<p className="truncate text-sm">{item.name}</p>
									<p
										aria-live="polite"
										className={cn(
											'text-xs',
											item.status === 'error'
												? 'text-destructive'
												: 'text-muted-foreground',
										)}
									>
										{item.message ?? STATUS_LABELS[item.status]}
									</p>
								</div>
								<button
									type="button"
									aria-label={`Retirer ${item.name}`}
									className="-m-1 flex size-11 shrink-0 items-center justify-center rounded-lg text-muted-foreground outline-none hover:bg-accent hover:text-foreground focus-visible:ring-[3px] focus-visible:ring-ring/50"
									onClick={() => onRemove(item.id)}
								>
									<X className="size-5" aria-hidden />
								</button>
							</div>
						</li>
					))}
				</ul>
			) : null}
		</div>
	)
}

import { ArrowDown, ArrowUp, Trash2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	KIND_LABELS,
	MAKER_LABEL_TEXT,
	type MakerLabel,
	type PhotosPerPage,
} from '#/pages/doe/types'
import {
	MAX_BLANK_PAGES,
	MIN_BLANK_PAGES,
} from '#/pages/doe-models/lib/model-form'
import type { ModelSectionDraft } from '#/pages/doe-models/types'

const PER_PAGE_LABELS: Record<PhotosPerPage, string> = {
	one: '1 par page',
	two: '2 par page',
	four: '4 par page',
}

function Caption({ htmlFor, children }: { htmlFor: string; children: string }) {
	return (
		<label htmlFor={htmlFor} className="text-[12px] text-muted-foreground">
			{children}
		</label>
	)
}

function KindSettings({
	baseId,
	section,
	onChange,
}: {
	baseId: string
	section: ModelSectionDraft
	onChange: (patch: Partial<ModelSectionDraft>) => void
}) {
	switch (section.kind) {
		case 'supplies':
			return (
				<div className="flex items-center gap-2">
					<Caption htmlFor={`${baseId}-maker`}>Libellé du fabricant</Caption>
					<Select
						value={section.makerLabel}
						onValueChange={(makerLabel) =>
							onChange({ makerLabel: makerLabel as MakerLabel })
						}
					>
						<SelectTrigger id={`${baseId}-maker`} size="sm" className="w-32">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{(Object.keys(MAKER_LABEL_TEXT) as MakerLabel[]).map((label) => (
								<SelectItem key={label} value={label}>
									{MAKER_LABEL_TEXT[label]}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</div>
			)
		case 'photos':
			return (
				<div className="flex items-center gap-2">
					<Caption htmlFor={`${baseId}-per-page`}>Photos par page</Caption>
					<Select
						value={section.perPage}
						onValueChange={(perPage) =>
							onChange({ perPage: perPage as PhotosPerPage })
						}
					>
						<SelectTrigger id={`${baseId}-per-page`} size="sm" className="w-32">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{(Object.keys(PER_PAGE_LABELS) as PhotosPerPage[]).map(
								(perPage) => (
									<SelectItem key={perPage} value={perPage}>
										{PER_PAGE_LABELS[perPage]}
									</SelectItem>
								),
							)}
						</SelectContent>
					</Select>
				</div>
			)
		case 'blank_pages':
			return (
				<div className="flex items-center gap-2">
					<Caption htmlFor={`${baseId}-count`}>Pages libres</Caption>
					<Input
						id={`${baseId}-count`}
						type="number"
						inputMode="numeric"
						min={MIN_BLANK_PAGES}
						max={MAX_BLANK_PAGES}
						value={section.count}
						onChange={(event) =>
							onChange({ count: Number(event.target.value) || MIN_BLANK_PAGES })
						}
						className="h-8 w-20"
					/>
					<span className="text-[12px] text-muted-foreground">
						de {MIN_BLANK_PAGES} à {MAX_BLANK_PAGES}
					</span>
				</div>
			)
		case 'plans':
			return null
	}
}

export function SectionRow({
	index,
	count,
	section,
	invalid,
	onChange,
	onMove,
	onRemove,
}: {
	index: number
	count: number
	section: ModelSectionDraft
	invalid: boolean
	onChange: (patch: Partial<ModelSectionDraft>) => void
	onMove: (delta: -1 | 1) => void
	onRemove: () => void
}) {
	const baseId = `section-${section.key}`
	const number = index + 1
	return (
		<li className="grid gap-2 rounded-md border bg-card p-3">
			<div className="flex items-center gap-2">
				<span
					aria-hidden
					className="w-5 shrink-0 text-center text-[12px] text-muted-foreground tabular-nums"
				>
					{number}
				</span>
				<Input
					value={section.title}
					onChange={(event) => onChange({ title: event.target.value })}
					aria-label={`Titre du thème ${number}`}
					aria-invalid={invalid ? true : undefined}
					autoComplete="off"
				/>
				<div className="flex shrink-0 items-center">
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						disabled={index === 0}
						onClick={() => onMove(-1)}
					>
						<ArrowUp aria-hidden />
						<span className="sr-only">Monter le thème {number}</span>
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						disabled={index === count - 1}
						onClick={() => onMove(1)}
					>
						<ArrowDown aria-hidden />
						<span className="sr-only">Descendre le thème {number}</span>
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						onClick={onRemove}
					>
						<Trash2 aria-hidden />
						<span className="sr-only">Supprimer le thème {number}</span>
					</Button>
				</div>
			</div>
			<div className="flex flex-wrap items-center gap-x-4 gap-y-2 pl-7">
				<span className="rounded-full bg-muted px-2 py-0.5 text-[12px] text-muted-foreground">
					{KIND_LABELS[section.kind]}
				</span>
				<KindSettings baseId={baseId} section={section} onChange={onChange} />
				<label className="ml-auto flex items-center gap-2 text-[13px]">
					<input
						type="checkbox"
						checked={section.enabled}
						onChange={(event) => onChange({ enabled: event.target.checked })}
						className="size-4 accent-primary"
					/>
					Activé par défaut
				</label>
			</div>
		</li>
	)
}

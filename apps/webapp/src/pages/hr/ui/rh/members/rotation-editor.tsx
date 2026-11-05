import { ChevronDown, ChevronUp, Copy, Plus, Trash2 } from 'lucide-react'
import { useRef, useState } from 'react'
import { FormField } from '#/components/form-field'
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
	describeCycle,
	MAX_BLOCKS,
	validateSchedule,
} from '#/pages/hr/lib/rh/contract-schedule'
import { formatDate } from '#/pages/hr/lib/rh/format'
import {
	appendBlock,
	blockLengthError,
	cycleLabel,
	duplicateBlock,
	groupScheduleErrors,
	moveBlock,
	patchBlock,
	previewLines,
	type RotationSchedule,
	removeBlock,
	scheduleVolume,
	weekSummary,
} from '#/pages/hr/lib/rh/schedule-mode'
import type { IsoDate, ScheduleBlock } from '#/pages/hr/lib/rh/types'
import { ScheduleEditor } from '#/pages/hr/ui/rh/members/schedule-editor'

const PREVIEW_COUNT = 8

function useBlockKeys(length: number) {
	const counter = useRef(0)
	const keys = useRef<string[]>([])
	const fresh = () => {
		counter.current += 1
		return `block-${counter.current}`
	}
	while (keys.current.length < length) keys.current.push(fresh())
	keys.current.length = length
	return {
		keys: keys.current,
		add: () => keys.current.push(fresh()),
		insertAfter: (index: number) => keys.current.splice(index + 1, 0, fresh()),
		swap: (index: number, target: number) => {
			const moved = keys.current[index]
			keys.current[index] = keys.current[target]
			keys.current[target] = moved
		},
		remove: (index: number) => keys.current.splice(index, 1),
	}
}

function BlockRow({
	idPrefix,
	index,
	count,
	block,
	open,
	onToggle,
	onChange,
	onDuplicate,
	onMove,
	onRemove,
}: {
	idPrefix: string
	index: number
	count: number
	block: ScheduleBlock
	open: boolean
	onToggle: () => void
	onChange: (patch: Partial<ScheduleBlock>) => void
	onDuplicate: () => void
	onMove: (delta: -1 | 1) => void
	onRemove: () => void
}) {
	const number = index + 1
	const lengthError = blockLengthError(block)
	return (
		<li className="rounded-lg border bg-card">
			<div className="flex flex-wrap items-center gap-2 p-3">
				<span
					aria-hidden
					className="flex size-6 shrink-0 items-center justify-center rounded-full bg-muted font-medium text-[11px] text-muted-foreground"
				>
					{number}
				</span>
				<Input
					type="number"
					min={1}
					inputMode="numeric"
					value={block.length}
					onChange={(event) =>
						onChange({ length: Math.trunc(Number(event.target.value)) || 0 })
					}
					aria-label={`Durée de la séquence ${number}`}
					aria-invalid={lengthError ? true : undefined}
					className="h-8 w-16 bg-card"
				/>
				<Select
					value={block.unit}
					onValueChange={(unit) =>
						onChange({ unit: unit as ScheduleBlock['unit'] })
					}
				>
					<SelectTrigger
						size="sm"
						className="w-28 bg-card"
						aria-label={`Unité de la séquence ${number}`}
					>
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="weeks">semaines</SelectItem>
						<SelectItem value="months">mois</SelectItem>
					</SelectContent>
				</Select>
				<Input
					value={block.label}
					onChange={(event) => onChange({ label: event.target.value })}
					placeholder="Entreprise, CFA, Semaine A…"
					aria-label={`Libellé de la séquence ${number}`}
					autoComplete="off"
					className="h-8 min-w-40 flex-1 bg-card"
				/>
				<span className="ml-auto flex items-center gap-0.5">
					<Button
						type="button"
						variant="ghost"
						size="icon-xs"
						aria-label={`Dupliquer la séquence ${number}`}
						onClick={onDuplicate}
					>
						<Copy aria-hidden />
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="icon-xs"
						aria-label={`Monter la séquence ${number}`}
						disabled={index === 0}
						onClick={() => onMove(-1)}
					>
						<ChevronUp aria-hidden />
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="icon-xs"
						aria-label={`Descendre la séquence ${number}`}
						disabled={index === count - 1}
						onClick={() => onMove(1)}
					>
						<ChevronDown aria-hidden />
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="icon-xs"
						aria-label={`Supprimer la séquence ${number}`}
						disabled={count === 1}
						onClick={onRemove}
					>
						<Trash2 aria-hidden />
					</Button>
				</span>
			</div>
			{lengthError && (
				<p className="px-3 pb-2 text-[12px] text-state-alert">{lengthError}</p>
			)}
			<div className="flex items-center justify-between gap-3 border-t px-3 py-1.5 text-[13px]">
				<span className="text-muted-foreground">{weekSummary(block.week)}</span>
				<Button
					type="button"
					variant="ghost"
					size="xs"
					aria-expanded={open}
					onClick={onToggle}
				>
					{open ? 'Replier la semaine' : 'Modifier la semaine'}
				</Button>
			</div>
			{open && (
				<div className="border-t p-3">
					<ScheduleEditor
						idPrefix={`${idPrefix}-week`}
						value={block.week}
						onChange={(week) => onChange({ week })}
					/>
				</div>
			)}
		</li>
	)
}

export function RotationEditor({
	idPrefix,
	value,
	onChange,
	from,
	today,
}: {
	idPrefix: string
	value: RotationSchedule
	onChange: (schedule: RotationSchedule) => void
	from: IsoDate
	today: IsoDate
}) {
	const errors = validateSchedule(value)
	const grouped = groupScheduleErrors(errors)
	const keys = useBlockKeys(value.blocks.length)
	const [openIndex, setOpenIndex] = useState<number | null>(null)
	const blocks = value.blocks
	const setBlocks = (next: readonly ScheduleBlock[]) =>
		onChange({ ...value, blocks: next })
	const valid = errors.length === 0

	return (
		<div className="grid gap-4">
			<FormField
				id={`${idPrefix}-anchor`}
				label="Début du cycle"
				required
				error={grouped.anchor ?? undefined}
				hint="Pour des semaines, choisis un lundi. Pour des mois, un jour avant le 29."
				className="sm:max-w-xs"
			>
				<Input
					id={`${idPrefix}-anchor`}
					type="date"
					value={value.anchor}
					onChange={(event) =>
						onChange({ ...value, anchor: event.target.value })
					}
					aria-invalid={grouped.anchor ? true : undefined}
					aria-describedby={
						grouped.anchor ? `${idPrefix}-anchor-error` : undefined
					}
				/>
			</FormField>

			<section aria-label="Séquences du cycle" className="grid gap-2">
				<h4 className="font-medium text-sm">Séquences</h4>
				<ol className="grid gap-2">
					{blocks.map((block, index) => (
						<BlockRow
							key={keys.keys[index]}
							idPrefix={`${idPrefix}-${keys.keys[index]}`}
							index={index}
							count={blocks.length}
							block={block}
							open={openIndex === index}
							onToggle={() => setOpenIndex(openIndex === index ? null : index)}
							onChange={(patch) => setBlocks(patchBlock(blocks, index, patch))}
							onDuplicate={() => {
								keys.insertAfter(index)
								setBlocks(duplicateBlock(blocks, index))
								setOpenIndex(index + 1)
							}}
							onMove={(delta) => {
								keys.swap(index, index + delta)
								setBlocks(moveBlock(blocks, index, delta))
								setOpenIndex(openIndex === index ? index + delta : openIndex)
							}}
							onRemove={() => {
								keys.remove(index)
								setBlocks(removeBlock(blocks, index))
								setOpenIndex(null)
							}}
						/>
					))}
				</ol>
				{grouped.blocks && (
					<p className="text-[12px] text-state-alert">{grouped.blocks}</p>
				)}
				<div>
					<Button
						type="button"
						variant="outline"
						size="sm"
						disabled={blocks.length >= MAX_BLOCKS}
						onClick={() => {
							keys.add()
							setBlocks(appendBlock(blocks))
							setOpenIndex(blocks.length)
						}}
					>
						<Plus aria-hidden /> Ajouter une séquence
					</Button>
				</div>
			</section>

			{valid && (
				<section
					aria-label="Récapitulatif du cycle"
					className="grid gap-1 rounded-md bg-muted/60 px-3 py-2.5 text-[13px]"
				>
					<p className="font-medium">{describeCycle(blocks)}</p>
					<p className="text-muted-foreground">{scheduleVolume(value, true)}</p>
				</section>
			)}

			{valid && (
				<section aria-label="Prochaines séquences" className="grid gap-2">
					<div>
						<h4 className="font-medium text-sm">Prochaines séquences</h4>
						<p className="text-[12px] text-muted-foreground">
							Vérifie le rythme sur les {cycleLabel(blocks)} à venir.
						</p>
					</div>
					<ol className="grid divide-y rounded-lg border bg-card">
						{previewLines(value, from, PREVIEW_COUNT).map((line, position) => (
							<li
								key={line.key}
								className="flex flex-wrap items-center gap-x-3 gap-y-0.5 px-3 py-2 text-[13px]"
							>
								<span className="min-w-24 font-medium">{line.label}</span>
								<span className="tabular text-muted-foreground">
									{formatDate(line.start)} au {formatDate(line.end)}
								</span>
								<span className="ml-auto text-muted-foreground">
									{line.summary}
								</span>
								{position === 0 && from === today && (
									<span className="inline-flex h-5 items-center rounded-sm bg-state-progress/12 px-1.5 font-medium text-[11px] text-state-progress">
										En cours
									</span>
								)}
							</li>
						))}
					</ol>
				</section>
			)}
		</div>
	)
}

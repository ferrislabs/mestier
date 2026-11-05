import {
	ArrowDown,
	ArrowUp,
	FileText,
	ImageIcon,
	Library,
	Pencil,
	Plus,
	Trash2,
} from 'lucide-react'
import { useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { supplyBadgeCounts, supplyNumber } from '#/pages/doe/lib/sections'
import {
	type DoeSupply,
	MAKER_LABEL_TEXT,
	type MakerLabel,
} from '#/pages/doe/types'
import { IconButton } from '#/pages/doe/ui/icon-button'
import { SectionFrame } from '#/pages/doe/ui/section-frame'
import { StatusBadge } from '#/pages/projects/ui/badges'

interface DoeSuppliesTabProps {
	sectionNumber: number
	title: string
	makerLabel: MakerLabel
	supplies: DoeSupply[]
	onMakerLabelChange: (label: MakerLabel) => void
	onPickFromCatalog: () => void
	onCreate: () => void
	onEdit: (supply: DoeSupply) => void
	onMove: (index: number, delta: -1 | 1) => void
	onRemove: (supply: DoeSupply) => void
}

function countLabel(count: number, singular: string, plural: string): string {
	return `${count} ${count > 1 ? plural : singular}`
}

export function DoeSuppliesTab({
	sectionNumber,
	title,
	makerLabel,
	supplies,
	onMakerLabelChange,
	onPickFromCatalog,
	onCreate,
	onEdit,
	onMove,
	onRemove,
}: DoeSuppliesTabProps) {
	const [removing, setRemoving] = useState<DoeSupply | null>(null)

	return (
		<SectionFrame
			number={sectionNumber}
			title={title}
			actions={
				<RequirePermission permission="MANAGE_PLANNING">
					<Select
						value={makerLabel}
						onValueChange={(value) => onMakerLabelChange(value as MakerLabel)}
					>
						<SelectTrigger
							aria-label="Libellé du fabricant"
							size="sm"
							className="w-36"
						>
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="manufacturer">
								{MAKER_LABEL_TEXT.manufacturer}
							</SelectItem>
							<SelectItem value="nursery">
								{MAKER_LABEL_TEXT.nursery}
							</SelectItem>
						</SelectContent>
					</Select>
					<Button
						type="button"
						variant="outline"
						size="sm"
						onClick={onPickFromCatalog}
					>
						<Library aria-hidden />
						Piocher dans le catalogue
					</Button>
					<Button type="button" size="sm" onClick={onCreate}>
						<Plus aria-hidden />
						Nouvelle fourniture
					</Button>
				</RequirePermission>
			}
		>
			{supplies.length === 0 ? (
				<p className="px-5 py-10 text-center text-muted-foreground text-sm">
					Aucune fourniture dans ce thème.
				</p>
			) : (
				<ul className="divide-y">
					{supplies.map((supply, index) => (
						<SupplyRow
							key={supply.id}
							numbering={supplyNumber(sectionNumber, index)}
							supply={supply}
							makerLabel={makerLabel}
							isFirst={index === 0}
							isLast={index === supplies.length - 1}
							onEdit={() => onEdit(supply)}
							onUp={() => onMove(index, -1)}
							onDown={() => onMove(index, 1)}
							onRemove={() => setRemoving(supply)}
						/>
					))}
				</ul>
			)}
			<ConfirmDialog
				open={removing !== null}
				onOpenChange={(open) => {
					if (!open) setRemoving(null)
				}}
				title="Supprimer cette fourniture ?"
				description={`« ${removing ? supplyName(removing) : ''} » sera retirée de ce thème. Le catalogue n’est pas modifié.`}
				confirmLabel="Supprimer"
				destructive
				onConfirm={() => {
					if (removing) onRemove(removing)
					setRemoving(null)
				}}
			/>
		</SectionFrame>
	)
}

function supplyName(supply: DoeSupply): string {
	return supply.title?.trim() || supply.designation
}

interface SupplyRowProps {
	numbering: string
	supply: DoeSupply
	makerLabel: MakerLabel
	isFirst: boolean
	isLast: boolean
	onEdit: () => void
	onUp: () => void
	onDown: () => void
	onRemove: () => void
}

function SupplyRow({
	numbering,
	supply,
	makerLabel,
	isFirst,
	isLast,
	onEdit,
	onUp,
	onDown,
	onRemove,
}: SupplyRowProps) {
	const counts = supplyBadgeCounts(supply)
	const name = supplyName(supply)
	const showDesignation = name !== supply.designation

	return (
		<li className="flex flex-col gap-3 px-5 py-4 md:flex-row md:items-start md:justify-between">
			<div className="grid min-w-0 gap-2">
				<div>
					<p className="font-medium">
						<span className="mr-2 text-muted-foreground tabular-nums">
							{numbering}
						</span>
						{name}
					</p>
					{showDesignation && (
						<p className="text-[13px] text-muted-foreground">
							{supply.designation}
						</p>
					)}
				</div>
				<dl className="grid gap-x-6 gap-y-1 text-sm sm:grid-cols-2">
					<Field label="Fournisseur" value={supply.supplier?.name} />
					<Field label={MAKER_LABEL_TEXT[makerLabel]} value={supply.maker} />
					<Field label="DPGF" value={supply.dpgf_ref} />
					<Field label="Destination" value={supply.destination} />
				</dl>
				<div className="flex flex-wrap gap-1.5">
					{counts.photos > 0 && (
						<StatusBadge tone="neutral">
							<ImageIcon aria-hidden className="size-3" />
							{countLabel(counts.photos, 'photo', 'photos')}
						</StatusBadge>
					)}
					{counts.datasheets > 0 && (
						<StatusBadge tone="neutral">
							<FileText aria-hidden className="size-3" />
							{countLabel(
								counts.datasheets,
								'fiche technique',
								'fiches techniques',
							)}
						</StatusBadge>
					)}
					{counts.isEmpty && (
						<StatusBadge tone="warn">Fiche incomplète</StatusBadge>
					)}
				</div>
			</div>
			<RequirePermission permission="MANAGE_PLANNING">
				<div className="flex shrink-0 items-center gap-0.5">
					<Button type="button" variant="outline" size="sm" onClick={onEdit}>
						<Pencil aria-hidden />
						Modifier
						<span className="sr-only"> {name}</span>
					</Button>
					<IconButton
						label={`Monter ${name}`}
						icon={ArrowUp}
						disabled={isFirst}
						onClick={onUp}
					/>
					<IconButton
						label={`Descendre ${name}`}
						icon={ArrowDown}
						disabled={isLast}
						onClick={onDown}
					/>
					<IconButton
						label={`Supprimer ${name}`}
						icon={Trash2}
						onClick={onRemove}
					/>
				</div>
			</RequirePermission>
		</li>
	)
}

function Field({ label, value }: { label: string; value?: string | null }) {
	return (
		<div className="flex gap-2">
			<dt className="text-muted-foreground">{label}</dt>
			<dd className="truncate">{value?.trim() ? value : '—'}</dd>
		</div>
	)
}

import {
	AlertTriangle,
	ArrowDown,
	ArrowUp,
	FileText,
	ImageIcon,
	Library,
	Pencil,
	Plus,
	X,
} from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import { supplyBadgeCounts, supplyNumber } from '#/pages/doe/lib/sections'
import {
	type DoeSupply,
	MAKER_LABEL_TEXT,
	type MakerLabel,
} from '#/pages/doe/types'

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
	return (
		<SectionCard>
			<SectionHeader
				title={`${sectionNumber}. ${title}`}
				actions={
					<RequirePermission permission="MANAGE_PLANNING">
						<div className="flex flex-wrap items-center gap-2">
							<Select
								value={makerLabel}
								onValueChange={(value) =>
									onMakerLabelChange(value as MakerLabel)
								}
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
								<Library />
								Piocher dans le catalogue
							</Button>
							<Button type="button" size="sm" onClick={onCreate}>
								<Plus />
								Nouvelle fourniture
							</Button>
						</div>
					</RequirePermission>
				}
			/>
			{supplies.length === 0 ? (
				<p className="p-8 text-center text-sm text-muted-foreground">
					Aucune fourniture dans ce thème.
				</p>
			) : (
				<ul className="divide-y">
					{supplies.map((supply, index) => (
						<SupplyCard
							key={supply.id}
							number={supplyNumber(sectionNumber, index)}
							supply={supply}
							makerLabel={makerLabel}
							isFirst={index === 0}
							isLast={index === supplies.length - 1}
							onEdit={() => onEdit(supply)}
							onUp={() => onMove(index, -1)}
							onDown={() => onMove(index, 1)}
							onRemove={() => onRemove(supply)}
						/>
					))}
				</ul>
			)}
		</SectionCard>
	)
}

interface SupplyCardProps {
	number: string
	supply: DoeSupply
	makerLabel: MakerLabel
	isFirst: boolean
	isLast: boolean
	onEdit: () => void
	onUp: () => void
	onDown: () => void
	onRemove: () => void
}

function SupplyCard({
	number,
	supply,
	makerLabel,
	isFirst,
	isLast,
	onEdit,
	onUp,
	onDown,
	onRemove,
}: SupplyCardProps) {
	const counts = supplyBadgeCounts(supply)
	const name = supply.title?.trim() || supply.designation

	return (
		<li className="flex flex-col gap-3 px-5 py-4 md:flex-row md:items-start md:justify-between">
			<div className="min-w-0 space-y-2">
				<p className="font-medium">
					<span className="mr-2 tabular-nums text-muted-foreground">
						{number}
					</span>
					{name}
				</p>
				<dl className="grid gap-x-6 gap-y-1 text-sm sm:grid-cols-2">
					<Field label="Fournisseur" value={supply.supplier?.name} />
					<Field label={MAKER_LABEL_TEXT[makerLabel]} value={supply.maker} />
					<Field label="DPGF" value={supply.dpgf_ref} />
					<Field label="Destination" value={supply.destination} />
				</dl>
				<div className="flex flex-wrap gap-2">
					{counts.photos > 0 ? (
						<Badge variant="secondary">
							<ImageIcon />
							{counts.photos} {counts.photos > 1 ? 'photos' : 'photo'}
						</Badge>
					) : null}
					{counts.datasheets > 0 ? (
						<Badge variant="secondary">
							<FileText />
							{counts.datasheets} {counts.datasheets > 1 ? 'pages' : 'page'} de
							fiche technique
						</Badge>
					) : null}
					{counts.isEmpty ? (
						<Badge variant="outline" className="text-warning">
							<AlertTriangle />
							Ni photo ni fiche technique
						</Badge>
					) : null}
				</div>
			</div>
			<RequirePermission permission="MANAGE_PLANNING">
				<div className="flex shrink-0 items-center gap-1">
					<Button type="button" variant="outline" size="sm" onClick={onEdit}>
						<Pencil />
						Modifier
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						disabled={isFirst}
						onClick={onUp}
					>
						<ArrowUp />
						<span className="sr-only">Monter {name}</span>
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						disabled={isLast}
						onClick={onDown}
					>
						<ArrowDown />
						<span className="sr-only">Descendre {name}</span>
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						onClick={onRemove}
					>
						<X />
						<span className="sr-only">Retirer {name}</span>
					</Button>
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

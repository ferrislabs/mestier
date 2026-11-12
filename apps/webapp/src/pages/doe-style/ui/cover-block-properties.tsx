import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Switch } from '#/components/ui/switch'
import type { CoverBlock } from '#/pages/doe-style/lib/cover-layout'

interface CoverBlockPropertiesProps {
	block: CoverBlock
	onChange: (next: CoverBlock) => void
}

export function CoverBlockProperties({
	block,
	onChange,
}: CoverBlockPropertiesProps) {
	return (
		<div className="grid gap-4">
			<div className="grid grid-cols-2 gap-4">
				<MmField
					label="X (mm)"
					value={block.x}
					onChange={(x) => onChange({ ...block, x })}
				/>
				<MmField
					label="Y (mm)"
					value={block.y}
					onChange={(y) => onChange({ ...block, y })}
				/>
				<MmField
					label="Largeur (mm)"
					value={block.width}
					onChange={(width) => onChange({ ...block, width })}
				/>
				<MmField
					label="Hauteur (mm)"
					value={block.height}
					onChange={(height) => onChange({ ...block, height })}
				/>
			</div>
			<div className="flex items-center gap-3">
				<Switch
					aria-label="Visible"
					checked={block.visible}
					onCheckedChange={(visible) => onChange({ ...block, visible })}
				/>
				<span className="font-medium text-sm">Visible</span>
			</div>
			<p className="text-muted-foreground text-xs">
				Un texte qui dépasse du cadre est coupé à l'impression.
			</p>
		</div>
	)
}

function MmField({
	label,
	value,
	onChange,
}: {
	label: string
	value: number
	onChange: (value: number) => void
}) {
	return (
		<div className="flex flex-col gap-2">
			<Label>{label}</Label>
			<Input
				type="number"
				inputMode="decimal"
				step={0.5}
				min={0}
				aria-label={label}
				value={Number.isFinite(value) ? value : ''}
				onChange={(event) => {
					if (event.target.value !== '') onChange(Number(event.target.value))
				}}
			/>
		</div>
	)
}

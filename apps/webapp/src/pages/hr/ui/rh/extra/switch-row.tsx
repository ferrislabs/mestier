import { useId } from 'react'
import { Label } from '#/components/ui/label'
import { Switch } from '#/components/ui/switch'

export function SwitchRow({
	label,
	help,
	checked,
	onChange,
}: {
	label: string
	help: string
	checked: boolean
	onChange: (checked: boolean) => void
}) {
	const id = useId()
	return (
		<div className="flex items-center justify-between gap-4">
			<div className="grid gap-1">
				<Label htmlFor={id}>{label}</Label>
				<p className="text-muted-foreground text-xs">{help}</p>
			</div>
			<Switch id={id} checked={checked} onCheckedChange={onChange} />
		</div>
	)
}

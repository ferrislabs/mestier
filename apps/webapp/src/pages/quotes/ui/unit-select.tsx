import {
	Select,
	SelectContent,
	SelectGroup,
	SelectItem,
	SelectLabel,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { ServiceRateUnit } from '#/hooks/use-reference-catalog'
import { formatUnit, UNIT_GROUPS } from '../types'

interface UnitSelectProps {
	value: ServiceRateUnit
	onChange: (unit: ServiceRateUnit) => void
}

/**
 * The unit picker, shared by the create and edit forms so the two can never
 * offer different sets. Grouped because a flat list of ten is a wall.
 */
export function UnitSelect({ value, onChange }: UnitSelectProps) {
	return (
		<Select
			value={value}
			onValueChange={(unit) => onChange(unit as ServiceRateUnit)}
		>
			<SelectTrigger className="w-full">
				<SelectValue />
			</SelectTrigger>
			<SelectContent>
				{UNIT_GROUPS.map((group) => (
					<SelectGroup key={group.label}>
						<SelectLabel>{group.label}</SelectLabel>
						{group.units.map((unit) => (
							<SelectItem key={unit} value={unit}>
								{formatUnit(unit)}
							</SelectItem>
						))}
					</SelectGroup>
				))}
			</SelectContent>
		</Select>
	)
}

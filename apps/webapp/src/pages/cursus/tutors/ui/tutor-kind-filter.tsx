import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import type { TutorTab } from '#/pages/cursus/tutors/lib/tutors'

export interface TutorKindFilterProps {
	tab: TutorTab
	companyCount: number | undefined
	schoolCount: number | undefined
	onTabChange: (tab: TutorTab) => void
}

function label(name: string, count: number | undefined): string {
	return count === undefined ? name : `${name} · ${count}`
}

export function TutorKindFilter({
	tab,
	companyCount,
	schoolCount,
	onTabChange,
}: TutorKindFilterProps) {
	return (
		<ToggleGroup
			type="single"
			variant="outline"
			size="sm"
			value={tab}
			onValueChange={(next) => {
				if (next === 'entreprise' || next === 'ecole') onTabChange(next)
			}}
			aria-label="Type de tuteur"
		>
			<ToggleGroupItem value="entreprise">
				{label('Entreprise', companyCount)}
			</ToggleGroupItem>
			<ToggleGroupItem value="ecole">
				{label('École', schoolCount)}
			</ToggleGroupItem>
		</ToggleGroup>
	)
}

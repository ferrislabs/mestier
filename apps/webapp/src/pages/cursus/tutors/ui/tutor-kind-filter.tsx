import { ChevronDown } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuRadioGroup,
	DropdownMenuRadioItem,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
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
	const company = label('Entreprise', companyCount)
	const school = label('École', schoolCount)

	return (
		<DropdownMenu>
			<DropdownMenuTrigger asChild>
				<Button
					type="button"
					variant="outline"
					size="sm"
					className="bg-card"
					aria-label="Type de tuteur"
				>
					{tab === 'ecole' ? school : company}
					<ChevronDown aria-hidden />
				</Button>
			</DropdownMenuTrigger>
			<DropdownMenuContent align="start">
				<DropdownMenuRadioGroup
					value={tab}
					onValueChange={(next) => {
						if (next === 'entreprise' || next === 'ecole') onTabChange(next)
					}}
				>
					<DropdownMenuRadioItem value="entreprise">
						{company}
					</DropdownMenuRadioItem>
					<DropdownMenuRadioItem value="ecole">{school}</DropdownMenuRadioItem>
				</DropdownMenuRadioGroup>
			</DropdownMenuContent>
		</DropdownMenu>
	)
}

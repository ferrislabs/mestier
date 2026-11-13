import { Tabs, TabsList, TabsTrigger } from '#/components/ui/tabs'
import type { TutorTab } from '#/pages/cursus/tutors/lib/tutors'

export interface TutorsTabsProps {
	tab: TutorTab
	companyCount: number | undefined
	schoolCount: number | undefined
	onTabChange: (tab: TutorTab) => void
}

function label(name: string, count: number | undefined): string {
	return count === undefined ? name : `${name} · ${count}`
}

export function TutorsTabs({
	tab,
	companyCount,
	schoolCount,
	onTabChange,
}: TutorsTabsProps) {
	return (
		<Tabs
			value={tab}
			onValueChange={(next) =>
				onTabChange(next === 'ecole' ? 'ecole' : 'entreprise')
			}
			className="mb-4"
		>
			<TabsList variant="line" aria-label="Type de tuteur">
				<TabsTrigger value="entreprise">
					{label('Entreprise', companyCount)}
				</TabsTrigger>
				<TabsTrigger value="ecole">{label('École', schoolCount)}</TabsTrigger>
			</TabsList>
		</Tabs>
	)
}

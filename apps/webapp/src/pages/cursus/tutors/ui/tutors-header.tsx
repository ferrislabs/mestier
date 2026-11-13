import { Plus } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageHeader } from '#/components/ui/surface'
import {
	TutorsTabs,
	type TutorsTabsProps,
} from '#/pages/cursus/tutors/ui/tutors-tabs'

export interface TutorsHeaderProps extends TutorsTabsProps {
	canManage: boolean
	onCreate: () => void
}

export function TutorsHeader({
	canManage,
	onCreate,
	...tabs
}: TutorsHeaderProps) {
	return (
		<>
			<PageHeader
				className="mb-6"
				title="Tuteurs"
				description="Les tuteurs entreprise et école qui suivent vos apprentis."
				actions={
					canManage ? (
						<Button type="button" size="sm" onClick={onCreate}>
							<Plus aria-hidden /> Nouveau tuteur
						</Button>
					) : null
				}
			/>
			<TutorsTabs {...tabs} />
		</>
	)
}

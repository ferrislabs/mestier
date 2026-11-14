import { Plus } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageHeader } from '#/components/ui/surface'
export interface TutorsHeaderProps {
	canManage: boolean
	onCreate: () => void
}

export function TutorsHeader({ canManage, onCreate }: TutorsHeaderProps) {
	return (
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
	)
}

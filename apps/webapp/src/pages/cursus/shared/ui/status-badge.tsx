import { Badge } from '#/components/ui/badge'

export function ClosedBadge({ closed }: { closed: boolean }) {
	return (
		<Badge variant={closed ? 'secondary' : 'outline'}>
			{closed ? 'Clôturé' : 'En cours'}
		</Badge>
	)
}

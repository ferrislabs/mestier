import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { cn } from '#/lib/utils'
import {
	isMissionStatus,
	MISSION_STATUS_LABELS,
	MISSION_STATUSES,
} from '#/pages/projects/lib/missions'
import type { Mission, MissionStatus } from '#/pages/projects/lib/model'

const DOTS: Record<MissionStatus, string> = {
	todo: 'bg-muted-foreground/50',
	in_progress: 'bg-primary',
	done: 'bg-state-progress',
}

export function MissionStatusSelect({
	mission,
	onChange,
}: {
	mission: Mission
	onChange: (status: MissionStatus) => void
}) {
	return (
		<Select
			value={mission.status}
			onValueChange={(value) => {
				if (isMissionStatus(value)) onChange(value)
			}}
		>
			<SelectTrigger
				size="sm"
				className="w-34 bg-card"
				aria-label={`Statut de la mission ${mission.title}`}
			>
				<SelectValue />
			</SelectTrigger>
			<SelectContent>
				{MISSION_STATUSES.map((status) => (
					<SelectItem key={status} value={status}>
						<span
							aria-hidden
							className={cn('size-2 rounded-full', DOTS[status])}
						/>
						{MISSION_STATUS_LABELS[status]}
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	)
}

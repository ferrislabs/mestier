import { ArrowDown, ArrowUp, Pencil, Trash2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import {
	type MissionFilter,
	moveAvailability,
} from '#/pages/projects/lib/mission-list'
import type { Mission, MissionStatus, Person } from '#/pages/projects/lib/model'
import { periodLabel } from '#/pages/projects/lib/project-view'
import { PersonAvatar } from '#/pages/projects/ui/avatars'
import { Dash } from '#/pages/projects/ui/dash'
import { MissionStatusSelect } from '#/pages/projects/ui/mission-status-select'

export function MissionsTable({
	missions,
	ordered,
	filter,
	people,
	onStatusChange,
	onMove,
	onEdit,
	onDelete,
}: {
	missions: readonly Mission[]
	ordered: readonly Mission[]
	filter: MissionFilter
	people: readonly Person[]
	onStatusChange: (mission: Mission, status: MissionStatus) => void
	onMove: (mission: Mission, delta: -1 | 1) => void
	onEdit: (mission: Mission) => void
	onDelete: (mission: Mission) => void
}) {
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="w-14 pl-4">Ordre</TableHead>
						<TableHead>Mission</TableHead>
						<TableHead>Statut</TableHead>
						<TableHead className="max-lg:hidden">Période</TableHead>
						<TableHead className="max-md:hidden">Responsable</TableHead>
						<TableHead className="pr-4 text-right">Actions</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{missions.map((mission) => {
						const index = ordered.findIndex((item) => item.id === mission.id)
						const moves = moveAvailability(index, ordered.length, filter)
						const responsible = people.find(
							(person) => person.id === mission.responsibleId,
						)
						const period = periodLabel(mission.startDate, mission.endDate)
						return (
							<TableRow key={mission.id}>
								<TableCell className="py-2.5 pl-4 text-muted-foreground tabular-nums">
									{index + 1}
								</TableCell>
								<TableCell className="max-w-80 whitespace-normal">
									<span className="block truncate font-medium">
										{mission.title}
									</span>
									{mission.description && (
										<span className="line-clamp-2 text-[12px] text-muted-foreground">
											{mission.description}
										</span>
									)}
								</TableCell>
								<TableCell>
									<MissionStatusSelect
										mission={mission}
										onChange={(status) => onStatusChange(mission, status)}
									/>
								</TableCell>
								<TableCell className="text-muted-foreground max-lg:hidden">
									{period ?? <Dash label="Période non renseignée" />}
								</TableCell>
								<TableCell className="max-md:hidden">
									<span className="flex items-center gap-2">
										<PersonAvatar person={responsible} />
										{responsible ? (
											<span className="block max-w-36 truncate">
												{responsible.name}
											</span>
										) : (
											<span className="text-muted-foreground">
												Sans responsable
											</span>
										)}
									</span>
								</TableCell>
								<TableCell className="pr-4">
									<div className="flex justify-end gap-0.5">
										<Button
											variant="ghost"
											size="icon-sm"
											disabled={!moves.up}
											aria-label={`Monter la mission ${mission.title}`}
											onClick={() => onMove(mission, -1)}
										>
											<ArrowUp aria-hidden />
										</Button>
										<Button
											variant="ghost"
											size="icon-sm"
											disabled={!moves.down}
											aria-label={`Descendre la mission ${mission.title}`}
											onClick={() => onMove(mission, 1)}
										>
											<ArrowDown aria-hidden />
										</Button>
										<Button
											variant="ghost"
											size="icon-sm"
											aria-label={`Modifier la mission ${mission.title}`}
											onClick={() => onEdit(mission)}
										>
											<Pencil aria-hidden />
										</Button>
										<Button
											variant="ghost"
											size="icon-sm"
											aria-label={`Supprimer la mission ${mission.title}`}
											onClick={() => onDelete(mission)}
										>
											<Trash2 aria-hidden />
										</Button>
									</div>
								</TableCell>
							</TableRow>
						)
					})}
				</TableBody>
			</Table>
		</div>
	)
}

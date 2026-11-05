import { Plus } from 'lucide-react'
import { useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { Progress } from '#/components/ui/progress'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import {
	filterMissions,
	isMissionFilter,
	MISSION_FILTERS,
	type MissionFilter,
	missionFilterLabel,
} from '#/pages/projects/lib/mission-list'
import { missionStats } from '#/pages/projects/lib/missions'
import type {
	Mission,
	MissionDraft,
	MissionStatus,
	Person,
	ProjectStatus,
} from '#/pages/projects/lib/model'
import { countLabel } from '#/pages/projects/lib/text'
import { MissionFormDialog } from '#/pages/projects/ui/mission-form-dialog'
import { MissionsTable } from '#/pages/projects/ui/missions-table'
import { Page } from '#/pages/projects/ui/page'
import { ProjectSubHeader } from '#/pages/projects/ui/project-header'

function ProgressTile({ missions }: { missions: readonly Mission[] }) {
	const stats = missionStats(missions)
	return (
		<div className="mb-5 rounded-lg border bg-card px-4 py-3">
			<div className="flex items-baseline justify-between gap-3">
				<span className="text-muted-foreground text-sm">Progression</span>
				<span className="font-semibold text-2xl tabular-nums">
					{stats.progress} %
				</span>
			</div>
			<Progress
				value={stats.progress}
				aria-label="Progression des missions"
				className="mt-2 h-2"
			/>
			<p className="mt-2 text-muted-foreground text-xs">
				{stats.total === 0
					? 'Aucune mission pour l’instant.'
					: `${countLabel(stats.done, 'terminée', 'terminées')} sur ${stats.total}, ${stats.inProgress} en cours, ${stats.todo} à faire.`}
			</p>
		</div>
	)
}

export interface MissionsPageProps {
	organizationSlug: string
	projectId: string
	projectName: string
	projectStatus: ProjectStatus
	ownerId: string | null
	people: readonly Person[]
	ordered: readonly Mission[]
	filter: MissionFilter
	onFilterChange: (filter: MissionFilter) => void
	onCreate: (draft: MissionDraft) => void
	onUpdate: (mission: Mission, draft: MissionDraft) => void
	onDelete: (mission: Mission) => void
	onStatusChange: (mission: Mission, status: MissionStatus) => void
	onMove: (mission: Mission, delta: -1 | 1) => void
}

export function MissionsPage({
	organizationSlug,
	projectId,
	projectName,
	projectStatus,
	ownerId,
	people,
	ordered,
	filter,
	onFilterChange,
	onCreate,
	onUpdate,
	onDelete,
	onStatusChange,
	onMove,
}: MissionsPageProps) {
	const [creating, setCreating] = useState(false)
	const [editing, setEditing] = useState<Mission | null>(null)
	const [editOpen, setEditOpen] = useState(false)
	const [deleting, setDeleting] = useState<Mission | null>(null)
	const [deleteOpen, setDeleteOpen] = useState(false)
	const listed = filterMissions(ordered, filter)

	return (
		<Page>
			<ProjectSubHeader
				organizationSlug={organizationSlug}
				projectId={projectId}
				name={projectName}
				status={projectStatus}
				title="Missions"
				actions={
					<Button size="sm" onClick={() => setCreating(true)}>
						<Plus aria-hidden /> Nouvelle mission
					</Button>
				}
			/>
			<ProgressTile missions={ordered} />
			{ordered.length > 0 && (
				<div className="mb-3 flex flex-wrap items-center gap-x-4 gap-y-2">
					<ToggleGroup
						type="single"
						variant="outline"
						size="sm"
						value={filter}
						onValueChange={(value) => {
							if (isMissionFilter(value)) onFilterChange(value)
						}}
						aria-label="Filtrer par statut"
					>
						{MISSION_FILTERS.map((option) => (
							<ToggleGroupItem key={option.value} value={option.value}>
								{option.label}
							</ToggleGroupItem>
						))}
					</ToggleGroup>
					{filter !== 'toutes' && (
						<p className="text-[12px] text-muted-foreground">
							L’ordre ne se change que dans la vue « Toutes ».
						</p>
					)}
				</div>
			)}
			{ordered.length === 0 ? (
				<EmptyState
					title="Aucune mission pour l’instant"
					hint="Découpe le projet en lots de travail pour suivre son avancement."
				>
					<Button size="sm" onClick={() => setCreating(true)}>
						<Plus aria-hidden /> Nouvelle mission
					</Button>
				</EmptyState>
			) : listed.length === 0 ? (
				<EmptyState
					title={`Aucune mission « ${missionFilterLabel(filter).toLowerCase()} »`}
					hint="Change de statut ou affiche toutes les missions du projet."
				>
					<Button
						variant="outline"
						size="sm"
						onClick={() => onFilterChange('toutes')}
					>
						Voir toutes les missions
					</Button>
				</EmptyState>
			) : (
				<MissionsTable
					missions={listed}
					ordered={ordered}
					filter={filter}
					people={people}
					onStatusChange={onStatusChange}
					onMove={onMove}
					onEdit={(mission) => {
						setEditing(mission)
						setEditOpen(true)
					}}
					onDelete={(mission) => {
						setDeleting(mission)
						setDeleteOpen(true)
					}}
				/>
			)}
			<MissionFormDialog
				open={creating}
				onOpenChange={setCreating}
				defaultResponsibleId={ownerId}
				people={people}
				onSubmit={onCreate}
			/>
			{editing && (
				<MissionFormDialog
					key={editing.id}
					open={editOpen}
					onOpenChange={setEditOpen}
					defaultResponsibleId={ownerId}
					mission={editing}
					people={people}
					onSubmit={(draft) => onUpdate(editing, draft)}
				/>
			)}
			<ConfirmDialog
				open={deleteOpen}
				onOpenChange={setDeleteOpen}
				title="Supprimer cette mission ?"
				description={`« ${deleting?.title ?? ''} » sera retirée du projet. Cette action est définitive.`}
				confirmLabel="Supprimer"
				destructive
				onConfirm={() => {
					if (deleting) onDelete(deleting)
					setDeleteOpen(false)
				}}
			/>
		</Page>
	)
}

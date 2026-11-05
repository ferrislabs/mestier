import { Link } from '@tanstack/react-router'
import {
	Archive,
	ArchiveRestore,
	ArrowRight,
	FileText,
	KanbanSquare,
	LayoutTemplate,
	Loader2,
	Pencil,
	Plus,
	Settings,
} from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { buildOrgPath } from '#/modules/org-path'
import { formatShortDate } from '#/pages/projects/lib/dates'
import { missionStats } from '#/pages/projects/lib/missions'
import type {
	Mission,
	MissionStatus,
	Person,
	ProjectExtras,
	ProjectStatus,
} from '#/pages/projects/lib/model'
import {
	type DoeSummary,
	doeSummaryParts,
	doeTileLabel,
	missionsRatio,
	periodLabel,
} from '#/pages/projects/lib/project-view'
import { countLabel } from '#/pages/projects/lib/text'
import { PersonAvatar } from '#/pages/projects/ui/avatars'
import { Dash } from '#/pages/projects/ui/dash'
import { MissionStatusSelect } from '#/pages/projects/ui/mission-status-select'
import { Page } from '#/pages/projects/ui/page'
import {
	ProjectBilling,
	type ProjectBillingProps,
} from '#/pages/projects/ui/project-billing'
import { ProjectHeader } from '#/pages/projects/ui/project-header'
import {
	DetailList,
	DetailRow,
	SectionCard,
} from '#/pages/projects/ui/section-card'
import { StatTile } from '#/pages/projects/ui/stat-tile'

const MISSION_PREVIEW = 6

export interface ProjectCustomerInfo {
	name: string
	email: string | null
	phone: string | null
}

export interface ProjectDetailUIProps extends ProjectBillingProps {
	status: ProjectStatus
	customer: ProjectCustomerInfo | null
	extras: ProjectExtras
	owner: Person | undefined
	people: readonly Person[]
	missions: readonly Mission[]
	doe: DoeSummary | null
	isDoeLoading: boolean
	canOpenDoe: boolean
	isBuildingTemplate: boolean
	error: string | null
	onEdit: () => void
	onToggleArchive: () => void
	onSaveAsTemplate: () => void
	onMissionStatusChange: (mission: Mission, status: MissionStatus) => void
}

function MissionPreview({
	organizationSlug,
	projectId,
	missions,
	people,
	onStatusChange,
}: {
	organizationSlug: string
	projectId: string
	missions: readonly Mission[]
	people: readonly Person[]
	onStatusChange: (mission: Mission, status: MissionStatus) => void
}) {
	const missionsPath = buildOrgPath(
		organizationSlug,
		'/planification/projects/$projectId/missions',
	)
	if (missions.length === 0)
		return (
			<div className="rounded-md border border-dashed px-4 py-6 text-center">
				<p className="text-[13px] text-muted-foreground">
					Aucune mission pour l’instant.
				</p>
				<Button variant="outline" size="sm" className="mt-3" asChild>
					<Link to={missionsPath} params={{ projectId }}>
						<Plus aria-hidden /> Ajouter une mission
					</Link>
				</Button>
			</div>
		)
	const shown = missions.slice(0, MISSION_PREVIEW)
	const rest = missions.length - shown.length
	return (
		<>
			<ul className="divide-y">
				{shown.map((mission) => {
					const responsible = people.find(
						(person) => person.id === mission.responsibleId,
					)
					return (
						<li
							key={mission.id}
							className="flex flex-wrap items-center gap-x-3 gap-y-2 py-2.5 first:pt-0"
						>
							<span className="min-w-0 flex-1">
								<span className="block truncate font-medium text-sm">
									{mission.title}
								</span>
								<span className="block truncate text-[12px] text-muted-foreground">
									{mission.endDate
										? `Fin prévue le ${formatShortDate(mission.endDate)}`
										: 'Sans échéance'}
								</span>
							</span>
							<span className="flex items-center gap-2">
								<PersonAvatar person={responsible} />
								<span className="sr-only">
									{responsible?.name ?? 'Sans responsable'}
								</span>
							</span>
							<MissionStatusSelect
								mission={mission}
								onChange={(status) => onStatusChange(mission, status)}
							/>
						</li>
					)
				})}
			</ul>
			{rest > 0 && (
				<p className="mt-2 text-[12px] text-muted-foreground">
					{rest === 1 ? '1 autre mission' : `${rest} autres missions`} dans la
					liste complète.
				</p>
			)}
		</>
	)
}

function DoeCard({
	organizationSlug,
	projectId,
	summary,
	isLoading,
	canOpenDoe,
}: {
	organizationSlug: string
	projectId: string
	summary: DoeSummary | null
	isLoading: boolean
	canOpenDoe: boolean
}) {
	return (
		<div className="grid gap-3">
			{summary ? (
				<p className="text-[13px]">{doeSummaryParts(summary).join(', ')}</p>
			) : (
				<p className="text-[13px] text-muted-foreground">
					{isLoading
						? 'Chargement du DOE…'
						: 'Ce projet n’a pas encore de DOE.'}
				</p>
			)}
			{canOpenDoe && !isLoading ? (
				<Button size="sm" className="w-fit" asChild>
					<Link
						to={buildOrgPath(
							organizationSlug,
							'/planification/projects/$projectId/doe',
						)}
						params={{ projectId }}
					>
						<FileText aria-hidden />
						{summary
							? `Ouvrir le DOE · indice ${summary.revision}`
							: 'Créer le DOE'}
					</Link>
				</Button>
			) : null}
		</div>
	)
}

function CustomerBlock({ customer }: { customer: ProjectCustomerInfo | null }) {
	if (!customer)
		return (
			<span className="text-muted-foreground">
				Projet interne, sans client.
			</span>
		)
	return (
		<span className="grid gap-0.5">
			<span className="font-medium">{customer.name}</span>
			{customer.phone && (
				<a
					href={`tel:${customer.phone.replace(/\s/g, '')}`}
					className="w-fit rounded-sm underline-offset-2 outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
				>
					{customer.phone}
				</a>
			)}
			{customer.email && (
				<a
					href={`mailto:${customer.email}`}
					className="w-fit rounded-sm underline-offset-2 outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
				>
					{customer.email}
				</a>
			)}
		</span>
	)
}

export function ProjectDetailUI({
	status,
	customer,
	extras,
	owner,
	people,
	missions,
	doe,
	isDoeLoading,
	canOpenDoe,
	isBuildingTemplate,
	error,
	onEdit,
	onToggleArchive,
	onSaveAsTemplate,
	onMissionStatusChange,
	...billing
}: ProjectDetailUIProps) {
	const { organizationSlug, project } = billing
	const stats = missionStats(missions)
	const archived = status === 'archived'
	const period = periodLabel(extras.startDate, extras.endDate)
	const projectPath = (suffix: string) =>
		buildOrgPath(
			organizationSlug,
			`/planification/projects/$projectId${suffix}`,
		)

	return (
		<Page>
			<ProjectHeader
				organizationSlug={organizationSlug}
				name={project.name}
				status={status}
				customerName={customer?.name ?? null}
				actions={
					<>
						<Button variant="outline" size="sm" asChild>
							<Link
								to={buildOrgPath(organizationSlug, '/planification/board')}
								search={{ project_id: project.id }}
							>
								<KanbanSquare aria-hidden /> Voir dans le tableau
							</Link>
						</Button>
						<Button variant="outline" size="sm" asChild>
							<Link
								to={projectPath('/settings')}
								params={{ projectId: project.id }}
							>
								<Settings aria-hidden /> Réglages
							</Link>
						</Button>
						<RequirePermission permission="MANAGE_PLANNING">
							{archived ? null : (
								<Button
									variant="outline"
									size="sm"
									disabled={isBuildingTemplate}
									onClick={onSaveAsTemplate}
								>
									{isBuildingTemplate ? (
										<Loader2 className="animate-spin" aria-hidden />
									) : (
										<LayoutTemplate aria-hidden />
									)}
									Enregistrer comme modèle
								</Button>
							)}
							<Button variant="outline" size="sm" onClick={onEdit}>
								<Pencil aria-hidden /> Modifier
							</Button>
							<Button variant="outline" size="sm" onClick={onToggleArchive}>
								{archived ? (
									<ArchiveRestore aria-hidden />
								) : (
									<Archive aria-hidden />
								)}
								{archived ? 'Réactiver' : 'Archiver'}
							</Button>
						</RequirePermission>
					</>
				}
			/>
			{error ? (
				<div
					role="alert"
					className="mb-5 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive"
				>
					{error}
				</div>
			) : null}
			<div className="mb-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
				<StatTile
					label="Missions"
					value={missionsRatio(stats)}
					hint={
						stats.total === 0
							? 'Aucune mission'
							: `${stats.progress} % terminées`
					}
				/>
				<StatTile
					label="DOE"
					value={doeTileLabel(doe)}
					hint={doe ? 'Dossier créé' : 'À créer'}
				/>
				<StatTile
					label="Période"
					value={period ?? 'Non planifiée'}
					valueClassName="text-lg leading-8"
					hint={period ? 'Début → fin' : 'Aucune date renseignée'}
				/>
				<StatTile
					label="Responsable"
					value={
						<span className="flex items-center gap-2">
							<PersonAvatar person={owner} size="md" />
							<span className="min-w-0 truncate">
								{owner?.name ?? 'Sans responsable'}
							</span>
						</span>
					}
					valueClassName="text-lg leading-8"
					hint={owner?.role || 'Aucun responsable désigné'}
				/>
			</div>
			<div className="grid items-start gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,22rem)]">
				<SectionCard
					title="Missions"
					hint={
						stats.total > 0
							? `${countLabel(stats.done, 'terminée', 'terminées')} sur ${stats.total}`
							: undefined
					}
					action={
						<Button variant="ghost" size="sm" asChild>
							<Link
								to={projectPath('/missions')}
								params={{ projectId: project.id }}
							>
								Voir les missions <ArrowRight aria-hidden />
							</Link>
						</Button>
					}
				>
					<MissionPreview
						organizationSlug={organizationSlug}
						projectId={project.id}
						missions={missions}
						people={people}
						onStatusChange={onMissionStatusChange}
					/>
				</SectionCard>
				<div className="grid gap-4">
					<SectionCard title="DOE">
						<DoeCard
							organizationSlug={organizationSlug}
							projectId={project.id}
							summary={doe}
							isLoading={isDoeLoading}
							canOpenDoe={canOpenDoe}
						/>
					</SectionCard>
					<SectionCard title="Informations">
						<DetailList>
							<DetailRow label="Client">
								<CustomerBlock customer={customer} />
							</DetailRow>
							<DetailRow label="Lieu d’intervention">
								{extras.place || <Dash />}
							</DetailRow>
							<DetailRow label="Description">
								{extras.description ? (
									<span className="whitespace-pre-line">
										{extras.description}
									</span>
								) : (
									<Dash label="Aucune description" />
								)}
							</DetailRow>
						</DetailList>
					</SectionCard>
				</div>
			</div>
			<ProjectBilling {...billing} />
		</Page>
	)
}
